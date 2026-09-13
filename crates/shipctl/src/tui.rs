//! Interactive TUI shipping portal (thin shell over doctor/portal/configure/flow).

use crate::adapters;
use crate::config;
use crate::flow;
use crate::portal::{self, PortalPlan, PortalStep, ProviderId};
use crate::secrets::{self, SecretHint, SecretsPlan};
use anyhow::{bail, Context, Result};
use crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers};
use crossterm::execute;
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use ratatui::backend::CrosstermBackend;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, List, ListItem, ListState, Paragraph, Wrap};
use ratatui::Terminal;
use std::io::{self, IsTerminal, Stdout};
use std::path::{Path, PathBuf};
use std::time::Duration;

#[derive(Clone, Copy, PartialEq, Eq)]
enum Screen {
    Home,
    Providers,
    Portal,
    Secrets,
    Wizard,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum WizardPhase {
    Doctor,
    Providers,
    Portal,
    Secrets,
    Configure,
    DryRun,
    Done,
}

struct App {
    project: PathBuf,
    screen: Screen,
    home_idx: usize,
    portal_idx: usize,
    provider_idx: usize,
    secrets_idx: usize,
    selected: Vec<bool>,
    plan: Option<PortalPlan>,
    secrets_plan: Option<SecretsPlan>,
    wizard: Option<WizardPhase>,
    status: String,
    log: Vec<String>,
}

const HOME_ITEMS: &[&str] = &[
    "Ship wizard (guided)",
    "Ship (one-shot offline prep)",
    "Human portal (open → paste)",
    "Guide (JSON checklist)",
    "Doctor",
    "Pick providers → Portal",
    "Portal (auto-detect)",
    "Secrets (paste assist)",
    "Configure (.ship/studio.json)",
    "Flow dry-run",
    "Flow (network deploy)",
    "Status",
    "Quit",
];

const WIZARD_LABELS: &[(WizardPhase, &str)] = &[
    (WizardPhase::Doctor, "1 · Doctor"),
    (WizardPhase::Providers, "2 · Pick providers"),
    (WizardPhase::Portal, "3 · Open entries / login"),
    (WizardPhase::Secrets, "4 · Paste secrets"),
    (WizardPhase::Configure, "5 · Configure"),
    (WizardPhase::DryRun, "6 · Flow dry-run"),
    (WizardPhase::Done, "7 · Done"),
];

pub fn run(project: &Path) -> Result<()> {
    if !io::stdin().is_terminal() || !io::stdout().is_terminal() {
        bail!("TUI requires an interactive terminal; use shipctl portal / doctor / flow instead");
    }

    let project = std::fs::canonicalize(project).unwrap_or_else(|_| project.to_path_buf());
    let detected = config::probe(&project);
    let detected_ids = portal::detected_providers(&detected);
    let all = ProviderId::all();
    let mut selected = vec![false; all.len()];
    for (i, id) in all.iter().enumerate() {
        selected[i] = detected_ids.contains(id);
    }
    if !selected.iter().any(|s| *s) {
        selected = vec![true; all.len()];
    }

    let mut app = App {
        project: project.clone(),
        screen: Screen::Home,
        home_idx: 0,
        portal_idx: 0,
        provider_idx: 0,
        secrets_idx: 0,
        selected,
        plan: None,
        secrets_plan: None,
        wizard: None,
        status: format!("project: {}", project.display()),
        log: vec![
            "Ship Studio TUI — wizard navigates entries; OAuth/env stay manual.".into(),
            "↑↓ select · Enter · Space toggle providers · q quit · Esc back".into(),
        ],
    };

    enable_raw_mode().context("enable raw mode")?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen).context("enter alt screen")?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend).context("terminal")?;

    let result = event_loop(&mut terminal, &mut app);

    disable_raw_mode().ok();
    execute!(terminal.backend_mut(), LeaveAlternateScreen).ok();
    terminal.show_cursor().ok();

    result
}

fn event_loop(terminal: &mut Terminal<CrosstermBackend<Stdout>>, app: &mut App) -> Result<()> {
    loop {
        terminal.draw(|f| draw(f, app))?;

        if !event::poll(Duration::from_millis(200))? {
            continue;
        }
        let Event::Key(key) = event::read()? else {
            continue;
        };
        if key.kind != KeyEventKind::Press {
            continue;
        }

        match app.screen {
            Screen::Home => match key.code {
                KeyCode::Char('q') => return Ok(()),
                KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                    return Ok(());
                }
                KeyCode::Up | KeyCode::Char('k') => {
                    if app.home_idx > 0 {
                        app.home_idx -= 1;
                    }
                }
                KeyCode::Down | KeyCode::Char('j') => {
                    if app.home_idx + 1 < HOME_ITEMS.len() {
                        app.home_idx += 1;
                    }
                }
                KeyCode::Enter => {
                    if !home_action(app)? {
                        return Ok(());
                    }
                }
                KeyCode::Char('d') => run_doctor(app),
                KeyCode::Char('p') => open_portal_auto(app),
                KeyCode::Char('w') => start_wizard(app),
                KeyCode::Char('g') => show_guide(app),
                KeyCode::Char('G') => {
                    show_guide(app);
                    if let Ok(plan) = crate::guide::plan_for(&app.project) {
                        match crate::guide::open_entries(&plan) {
                            Ok(u) => app.push(format!("opened {} entry url(s)", u.len())),
                            Err(e) => app.push(format!("open failed: {e:#}")),
                        }
                    }
                }
                _ => {}
            },
            Screen::Providers => match key.code {
                KeyCode::Esc | KeyCode::Char('b') => {
                    if app.wizard.is_some() {
                        app.screen = Screen::Wizard;
                    } else {
                        app.screen = Screen::Home;
                    }
                    app.status = "back".into();
                }
                KeyCode::Char('q') => return Ok(()),
                KeyCode::Up | KeyCode::Char('k') => {
                    if app.provider_idx > 0 {
                        app.provider_idx -= 1;
                    }
                }
                KeyCode::Down | KeyCode::Char('j') => {
                    if app.provider_idx + 1 < app.selected.len() {
                        app.provider_idx += 1;
                    }
                }
                KeyCode::Char(' ') => {
                    app.selected[app.provider_idx] = !app.selected[app.provider_idx];
                }
                KeyCode::Enter => {
                    if selected_providers(app).is_empty() {
                        app.push("select at least one provider (Space)");
                        continue;
                    }
                    load_portal_from_selection(app);
                    if app.wizard.is_some() {
                        app.wizard = Some(WizardPhase::Portal);
                        app.screen = Screen::Portal;
                    }
                }
                _ => {}
            },
            Screen::Portal => match key.code {
                KeyCode::Esc | KeyCode::Char('b') => {
                    if app.wizard.is_some() {
                        app.wizard = Some(WizardPhase::Providers);
                        app.screen = Screen::Providers;
                    } else {
                        app.screen = Screen::Home;
                    }
                    app.status = "back".into();
                }
                KeyCode::Char('q') => return Ok(()),
                KeyCode::Up | KeyCode::Char('k') => {
                    if app.portal_idx > 0 {
                        app.portal_idx -= 1;
                    }
                }
                KeyCode::Down | KeyCode::Char('j') => {
                    let n = app.plan.as_ref().map(|p| p.steps.len()).unwrap_or(0);
                    if n > 0 && app.portal_idx + 1 < n {
                        app.portal_idx += 1;
                    }
                }
                KeyCode::Enter | KeyCode::Char('o') => open_selected_step(app),
                KeyCode::Char('l') => login_selected_provider(app),
                KeyCode::Char('a') => open_all_urls(app),
                KeyCode::Char('n') if app.wizard.is_some() => {
                    app.wizard = Some(WizardPhase::Secrets);
                    open_secrets(app);
                }
                _ => {}
            },
            Screen::Secrets => match key.code {
                KeyCode::Esc | KeyCode::Char('b') => {
                    if app.wizard.is_some() {
                        app.wizard = Some(WizardPhase::Portal);
                        app.screen = Screen::Portal;
                    } else {
                        app.screen = Screen::Home;
                    }
                    app.status = "back".into();
                }
                KeyCode::Char('q') => return Ok(()),
                KeyCode::Up | KeyCode::Char('k') => {
                    if app.secrets_idx > 0 {
                        app.secrets_idx -= 1;
                    }
                }
                KeyCode::Down | KeyCode::Char('j') => {
                    let n = app
                        .secrets_plan
                        .as_ref()
                        .map(|p| p.hints.len())
                        .unwrap_or(0);
                    if n > 0 && app.secrets_idx + 1 < n {
                        app.secrets_idx += 1;
                    }
                }
                KeyCode::Enter | KeyCode::Char('p') => put_selected_secret(app),
                KeyCode::Char('o') => open_selected_secret_url(app),
                KeyCode::Char('v') => export_vault_from_hints(app),
                KeyCode::Char('a') => {
                    if let Some(plan) = &app.secrets_plan {
                        match secrets::open_entry_urls(plan) {
                            Ok(u) => app.push(format!("opened {} url(s)", u.len())),
                            Err(e) => app.push(format!("open failed: {e:#}")),
                        }
                    }
                }
                KeyCode::Char('n') if app.wizard.is_some() => {
                    app.wizard = Some(WizardPhase::Configure);
                    app.screen = Screen::Wizard;
                    app.status = "wizard → configure (Enter)".into();
                }
                _ => {}
            },
            Screen::Wizard => match key.code {
                KeyCode::Esc | KeyCode::Char('b') => {
                    app.wizard = None;
                    app.screen = Screen::Home;
                    app.status = "wizard cancelled".into();
                }
                KeyCode::Char('q') => return Ok(()),
                KeyCode::Enter | KeyCode::Char('n') => wizard_advance(app)?,
                _ => {}
            },
        }
    }
}

impl App {
    fn push(&mut self, line: impl Into<String>) {
        self.log.push(line.into());
        if self.log.len() > 40 {
            self.log.drain(0..self.log.len() - 40);
        }
    }
}

fn selected_providers(app: &App) -> Vec<ProviderId> {
    ProviderId::all()
        .into_iter()
        .enumerate()
        .filter(|(i, _)| app.selected[*i])
        .map(|(_, id)| id)
        .collect()
}

fn home_action(app: &mut App) -> Result<bool> {
    match app.home_idx {
        0 => {
            start_wizard(app);
            Ok(true)
        }
        1 => {
            do_ship_prep(app, false);
            Ok(true)
        }
        2 => {
            do_human(app, true, false);
            Ok(true)
        }
        3 => {
            show_guide(app);
            Ok(true)
        }
        4 => {
            run_doctor(app);
            Ok(true)
        }
        5 => {
            app.wizard = None;
            app.screen = Screen::Providers;
            app.status = "Space toggle · Enter open portal".into();
            Ok(true)
        }
        6 => {
            open_portal_auto(app);
            Ok(true)
        }
        7 => {
            open_secrets(app);
            Ok(true)
        }
        8 => {
            do_configure(app);
            Ok(true)
        }
        9 => {
            do_dry_run(app);
            Ok(true)
        }
        10 => {
            do_flow(app);
            Ok(true)
        }
        11 => {
            do_status(app);
            Ok(true)
        }
        12 => Ok(false),
        _ => Ok(true),
    }
}

fn do_human(app: &mut App, open: bool, put: bool) {
    if put {
        let _ = disable_raw_mode();
        let _ = execute!(io::stdout(), LeaveAlternateScreen);
    }
    let result = crate::human::run(&app.project, open, put);
    if put {
        let _ = execute!(io::stdout(), EnterAlternateScreen);
        let _ = enable_raw_mode();
    }
    match result {
        Ok(sprint) => {
            app.push(format!(
                "human · opened={} · put_queue={}",
                sprint.opened.len(),
                sprint.put_queue.len()
            ));
            for c in &sprint.checklist {
                app.push(format!("  · {c}"));
            }
            app.status = "human portal — run --put in a terminal to paste".into();
        }
        Err(e) => {
            app.push(format!("human failed: {e:#}"));
            app.status = "human failed".into();
        }
    }
}

fn do_ship_prep(app: &mut App, open: bool) {
    match crate::ship::run(&app.project, open) {
        Ok(prep) => {
            app.push(format!(
                "ship prep · providers={} · secrets={} · opened={}",
                prep.guide.providers.join(","),
                prep.guide.secret_hint_count,
                prep.opened.len()
            ));
            for n in &prep.next {
                app.push(format!("  → {n}"));
            }
            app.status = "ship prep ok — see next in log".into();
        }
        Err(e) => {
            app.push(format!("ship prep failed: {e:#}"));
            app.status = "ship prep failed".into();
        }
    }
}

fn show_guide(app: &mut App) {
    match crate::guide::plan_for(&app.project) {
        Ok(plan) => {
            app.push(format!(
                "guide · doctor_ok={} · providers={} · secrets={}",
                plan.doctor_ok,
                plan.providers.join(","),
                plan.secret_hint_count
            ));
            for s in &plan.steps {
                app.push(format!("  · {} — {}", s.id, s.title));
            }
            app.status = "guide loaded (see log)".into();
        }
        Err(e) => {
            app.push(format!("guide failed: {e:#}"));
            app.status = "guide failed".into();
        }
    }
}

fn start_wizard(app: &mut App) {
    app.wizard = Some(WizardPhase::Doctor);
    app.screen = Screen::Wizard;
    app.status = "Ship wizard — Enter to run each phase".into();
    app.push("wizard started");
}

fn wizard_advance(app: &mut App) -> Result<()> {
    let phase = app.wizard.unwrap_or(WizardPhase::Doctor);
    match phase {
        WizardPhase::Doctor => {
            run_doctor(app);
            app.wizard = Some(WizardPhase::Providers);
            app.screen = Screen::Providers;
            app.status = "pick providers · Space · Enter".into();
        }
        WizardPhase::Providers => {
            app.screen = Screen::Providers;
        }
        WizardPhase::Portal => {
            app.screen = Screen::Portal;
            app.status = "open/login entries · then n next".into();
        }
        WizardPhase::Secrets => {
            open_secrets(app);
        }
        WizardPhase::Configure => {
            do_configure(app);
            app.wizard = Some(WizardPhase::DryRun);
            app.status = "Enter for flow dry-run".into();
        }
        WizardPhase::DryRun => {
            do_dry_run(app);
            app.wizard = Some(WizardPhase::Done);
            app.status = "wizard complete — Enter to finish".into();
        }
        WizardPhase::Done => {
            app.wizard = None;
            app.screen = Screen::Home;
            app.status = "wizard done".into();
            app.push("wizard finished");
        }
    }
    Ok(())
}

fn do_configure(app: &mut App) {
    match config::configure(&app.project) {
        Ok(intent) => {
            app.push(format!("configured · deploy_args={:?}", intent.deploy_args));
            app.status = "configure ok".into();
        }
        Err(e) => {
            app.push(format!("configure failed: {e:#}"));
            app.status = "configure failed".into();
        }
    }
}

fn do_dry_run(app: &mut App) {
    match flow::plan(&app.project, false, true, true) {
        Ok(plan) => {
            app.push(format!(
                "dry-run · {} steps · offline={}",
                plan.steps.len(),
                plan.offline
            ));
            for s in &plan.steps {
                app.push(format!("  · {} {:?}", s.id, s.args));
            }
            app.status = "flow dry-run ok".into();
        }
        Err(e) => {
            app.push(format!("plan failed: {e:#}"));
            app.status = "dry-run failed".into();
        }
    }
}

fn do_flow(app: &mut App) {
    let _ = disable_raw_mode();
    let _ = execute!(io::stdout(), LeaveAlternateScreen);
    let plan = flow::plan(&app.project, false, false, false);
    let exec = plan.and_then(|p| flow::execute(&app.project, &p));
    let _ = execute!(io::stdout(), EnterAlternateScreen);
    let _ = enable_raw_mode();
    match exec {
        Ok(()) => {
            app.push("flow finished ok");
            app.status = "flow ok".into();
        }
        Err(e) => {
            app.push(format!("flow failed: {e:#}"));
            app.status = "flow failed".into();
        }
    }
}

fn do_status(app: &mut App) {
    match config::read_last_run(&app.project) {
        Ok(v) => {
            app.push(v.to_string());
            app.status = "status loaded".into();
        }
        Err(e) => app.push(format!("status failed: {e:#}")),
    }
}

fn run_doctor(app: &mut App) {
    match adapters::doctor(&app.project) {
        Ok(report) => {
            app.push(format!(
                "doctor ok={} · signet={} · orbit={}",
                report.ok, report.signet.found, report.orbit.found
            ));
            for n in report.notes.iter().take(8) {
                app.push(format!("  · {n}"));
            }
            app.status = if report.ok {
                "doctor healthy".into()
            } else {
                "doctor issues".into()
            };
        }
        Err(e) => {
            app.push(format!("doctor failed: {e:#}"));
            app.status = "doctor failed".into();
        }
    }
}

fn open_secrets(app: &mut App) {
    match secrets::plan_for(&app.project, None) {
        Ok(plan) => {
            app.push(format!("secrets · {} hint(s)", plan.hints.len()));
            app.secrets_idx = 0;
            app.secrets_plan = Some(plan);
            app.screen = Screen::Secrets;
            app.status = "secrets — Enter put · o open · v vault.km · a all · n next".into();
        }
        Err(e) => {
            app.push(format!("secrets failed: {e:#}"));
            app.status = "secrets failed".into();
        }
    }
}

fn selected_secret(app: &App) -> Option<&SecretHint> {
    app.secrets_plan.as_ref()?.hints.get(app.secrets_idx)
}

fn open_selected_secret_url(app: &mut App) {
    if let Some(url) = selected_secret(app).and_then(|h| h.entry_url.clone()) {
        match portal::open_url(&url) {
            Ok(()) => app.push(format!("opened {url}")),
            Err(e) => app.push(format!("open failed: {e:#}")),
        }
    }
}

fn put_selected_secret(app: &mut App) {
    let Some(hint) = selected_secret(app) else {
        return;
    };
    let name = hint.name.clone();
    let provider = hint.provider.clone();
    let id = match ProviderId::parse(&provider) {
        Ok(id) => id,
        Err(e) => {
            app.push(format!("{e:#}"));
            return;
        }
    };
    if name == "<NAME>" {
        app.push("template hint — set a real name via CLI: shipctl secrets put --name …");
        return;
    }
    app.push(format!("putting {provider}/{name} — paste in the child CLI…"));
    let _ = disable_raw_mode();
    let _ = execute!(io::stdout(), LeaveAlternateScreen);
    let code = secrets::put_secret(&app.project, id, &name);
    let _ = execute!(io::stdout(), EnterAlternateScreen);
    let _ = enable_raw_mode();
    match code {
        Ok(0) => {
            app.push(format!("put ok · {name}"));
            app.status = "secret put ok".into();
        }
        Ok(c) => {
            app.push(format!("put exited {c}"));
            app.status = "secret put failed".into();
        }
        Err(e) => {
            app.push(format!("put failed: {e:#}"));
            app.status = "secret put failed".into();
        }
    }
}

fn export_vault_from_hints(app: &mut App) {
    let hints: Vec<String> = app
        .secrets_plan
        .as_ref()
        .map(|p| {
            p.hints
                .iter()
                .map(|h| h.name.clone())
                .filter(|n| !n.is_empty() && !n.contains('<'))
                .collect()
        })
        .unwrap_or_default();
    let default_out = app.project.join("ship-secrets.km");
    app.push(format!(
        "vault export → {} (Clavis-compatible .km)",
        default_out.display()
    ));
    let _ = disable_raw_mode();
    let _ = execute!(io::stdout(), LeaveAlternateScreen);
    eprintln!("\n=== Encrypted vault export (kmvault) ===\n");
    let result = crate::vault_km::export_interactive(
        &default_out,
        "Ship Studio secrets",
        &hints,
    );
    let _ = execute!(io::stdout(), EnterAlternateScreen);
    let _ = enable_raw_mode();
    match result {
        Ok(path) => {
            app.push(format!("vault written · {}", path.display()));
            app.status = "vault export ok".into();
        }
        Err(e) => {
            app.push(format!("vault export failed: {e:#}"));
            app.status = "vault export failed".into();
        }
    }
}

fn open_portal_auto(app: &mut App) {
    app.wizard = None;
    match portal::plan_for(&app.project, None) {
        Ok(plan) => apply_plan(app, plan),
        Err(e) => {
            app.push(format!("portal failed: {e:#}"));
            app.status = "portal failed".into();
        }
    }
}

fn load_portal_from_selection(app: &mut App) {
    let ids = selected_providers(app);
    match portal::plan_for_providers(&app.project, &ids) {
        Ok(plan) => apply_plan(app, plan),
        Err(e) => {
            app.push(format!("portal failed: {e:#}"));
            app.status = "portal failed".into();
        }
    }
}

fn apply_plan(app: &mut App, plan: PortalPlan) {
    app.push(format!("portal · providers={}", plan.providers.join(",")));
    app.portal_idx = 0;
    app.plan = Some(plan);
    app.screen = Screen::Portal;
    app.status = "portal — Enter open · l login · a all · n next (wizard)".into();
}

fn selected_step(app: &App) -> Option<&PortalStep> {
    app.plan.as_ref()?.steps.get(app.portal_idx)
}

fn open_selected_step(app: &mut App) {
    let url = selected_step(app).and_then(|s| s.entry_url.clone());
    if let Some(url) = url {
        match portal::open_url(&url) {
            Ok(()) => {
                app.push(format!("opened {url}"));
                app.status = "opened entry url".into();
            }
            Err(e) => app.push(format!("open failed: {e:#}")),
        }
    } else if let Some(step) = selected_step(app) {
        if step.kind == "oauth" {
            login_selected_provider(app);
        } else {
            app.push(format!("{} — no URL (use l for OAuth CLI)", step.title));
        }
    }
}

fn open_all_urls(app: &mut App) {
    if let Some(plan) = &app.plan {
        match portal::open_urls(plan) {
            Ok(urls) => {
                app.push(format!("opened {} url(s)", urls.len()));
                app.status = "opened token pages".into();
            }
            Err(e) => app.push(format!("open failed: {e:#}")),
        }
    }
}

fn login_selected_provider(app: &mut App) {
    let Some(step) = selected_step(app) else {
        return;
    };
    let provider = step.provider.clone();
    let filter = match ProviderId::parse(&provider) {
        Ok(id) => id,
        Err(e) => {
            app.push(format!("{e:#}"));
            return;
        }
    };
    let plan = match portal::plan_for(&app.project, Some(filter)) {
        Ok(p) => p,
        Err(e) => {
            app.push(format!("{e:#}"));
            return;
        }
    };

    let _ = disable_raw_mode();
    let _ = execute!(io::stdout(), LeaveAlternateScreen);
    let results = portal::run_logins(&app.project, &plan);
    let _ = execute!(io::stdout(), EnterAlternateScreen);
    let _ = enable_raw_mode();

    match results {
        Ok(rows) => {
            for r in rows {
                app.push(r.to_string());
            }
            app.status = format!("{provider} login finished");
        }
        Err(e) => {
            app.push(format!("login failed: {e:#}"));
            app.status = "login failed".into();
        }
    }
}

fn draw(f: &mut ratatui::Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(8),
            Constraint::Length(10),
            Constraint::Length(3),
        ])
        .split(f.area());

    let title = Paragraph::new(Line::from(vec![
        Span::styled(
            " Ship Studio ",
            Style::default()
                .fg(Color::Black)
                .bg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw("  CLI · TUI · Desktop shipping portal"),
    ]))
    .block(Block::default().borders(Borders::ALL).title("shipctl tui"));
    f.render_widget(title, chunks[0]);

    match app.screen {
        Screen::Home => draw_home(f, app, chunks[1]),
        Screen::Providers => draw_providers(f, app, chunks[1]),
        Screen::Portal => draw_portal(f, app, chunks[1]),
        Screen::Secrets => draw_secrets(f, app, chunks[1]),
        Screen::Wizard => draw_wizard(f, app, chunks[1]),
    }

    let log_lines: Vec<Line> = app
        .log
        .iter()
        .rev()
        .take(8)
        .rev()
        .map(|l| Line::from(l.as_str()))
        .collect();
    let log = Paragraph::new(log_lines)
        .wrap(Wrap { trim: true })
        .block(Block::default().borders(Borders::ALL).title("log"));
    f.render_widget(log, chunks[2]);

    let help = match app.screen {
        Screen::Home => "↑↓ · Enter · w wizard · d doctor · p portal · q quit",
        Screen::Providers => "↑↓ · Space toggle · Enter continue · Esc back",
        Screen::Portal => "↑↓ · Enter/o open · l login · a all · n next · Esc back",
        Screen::Secrets => "↑↓ · Enter put · o open URL · v vault.km · a all · n next · Esc back",
        Screen::Wizard => "Enter / n advance · Esc cancel wizard",
    };
    let status = Paragraph::new(format!("{}\n{}", app.status, help))
        .block(Block::default().borders(Borders::ALL).title("status"));
    f.render_widget(status, chunks[3]);
}

fn draw_home(f: &mut ratatui::Frame, app: &App, area: Rect) {
    let items: Vec<ListItem> = HOME_ITEMS
        .iter()
        .enumerate()
        .map(|(i, label)| {
            let style = if i == app.home_idx {
                Style::default()
                    .fg(Color::Black)
                    .bg(Color::Cyan)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };
            ListItem::new(*label).style(style)
        })
        .collect();
    let mut state = ListState::default();
    state.select(Some(app.home_idx));
    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(format!("home · {}", app.project.display())),
        )
        .highlight_symbol("› ");
    f.render_stateful_widget(list, area, &mut state);
}

fn draw_providers(f: &mut ratatui::Frame, app: &App, area: Rect) {
    let items: Vec<ListItem> = ProviderId::all()
        .iter()
        .enumerate()
        .map(|(i, id)| {
            let mark = if app.selected[i] { "[x]" } else { "[ ]" };
            let line = format!("{mark} {} ({})", id.label(), id.as_str());
            let style = if i == app.provider_idx {
                Style::default()
                    .fg(Color::Black)
                    .bg(Color::Green)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };
            ListItem::new(line).style(style)
        })
        .collect();
    let mut state = ListState::default();
    state.select(Some(app.provider_idx));
    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("providers · Space toggle · Enter → portal"),
        )
        .highlight_symbol("› ");
    f.render_stateful_widget(list, area, &mut state);
}

fn draw_wizard(f: &mut ratatui::Frame, app: &App, area: Rect) {
    let current = app.wizard.unwrap_or(WizardPhase::Doctor);
    let items: Vec<ListItem> = WIZARD_LABELS
        .iter()
        .map(|(phase, label)| {
            let style = if *phase == current {
                Style::default()
                    .fg(Color::Black)
                    .bg(Color::Magenta)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::DarkGray)
            };
            ListItem::new(*label).style(style)
        })
        .collect();
    let list = List::new(items).block(
        Block::default()
            .borders(Borders::ALL)
            .title("ship wizard — Enter runs highlighted phase"),
    );
    f.render_widget(list, area);
}

fn draw_secrets(f: &mut ratatui::Frame, app: &App, area: Rect) {
    let Some(plan) = &app.secrets_plan else {
        f.render_widget(
            Paragraph::new("no secrets plan").block(Block::default().borders(Borders::ALL)),
            area,
        );
        return;
    };
    let items: Vec<ListItem> = plan
        .hints
        .iter()
        .enumerate()
        .map(|(i, h)| {
            let line = format!(
                "{} / {} · {}",
                h.provider,
                h.name,
                h.put_cli.join(" ")
            );
            let style = if i == app.secrets_idx {
                Style::default()
                    .fg(Color::Black)
                    .bg(Color::Red)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };
            ListItem::new(line).style(style)
        })
        .collect();
    let mut state = ListState::default();
    state.select(Some(app.secrets_idx));
    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("secrets · paste in child CLI (values never stored)"),
        )
        .highlight_symbol("› ");
    f.render_stateful_widget(list, area, &mut state);
}

fn draw_portal(f: &mut ratatui::Frame, app: &App, area: Rect) {
    let Some(plan) = &app.plan else {
        f.render_widget(
            Paragraph::new("no portal plan").block(Block::default().borders(Borders::ALL)),
            area,
        );
        return;
    };
    let items: Vec<ListItem> = plan
        .steps
        .iter()
        .enumerate()
        .map(|(i, step)| {
            let marker = if step.entry_url.is_some() {
                "URL"
            } else if step.cli.is_some() {
                "CLI"
            } else {
                "···"
            };
            let line = format!(
                "[{}] {} · {} — {}",
                marker, step.provider, step.kind, step.title
            );
            let style = if i == app.portal_idx {
                Style::default()
                    .fg(Color::Black)
                    .bg(Color::Yellow)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };
            ListItem::new(line).style(style)
        })
        .collect();
    let mut state = ListState::default();
    state.select(Some(app.portal_idx));
    let title = format!("portal · {}", plan.providers.join(", "));
    let list = List::new(items)
        .block(Block::default().borders(Borders::ALL).title(title))
        .highlight_symbol("› ");
    f.render_stateful_widget(list, area, &mut state);
}
