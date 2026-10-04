//! Hosted CLI deploy — H1 Cloudflare · H2 Vercel / Netlify · H4 failure taxonomy.
//! Streams vendor CLI on the operator machine; writes hosted last-run URLs.
//! Never calls vendor HTTPS with secrets; never creates API tokens.

use anyhow::{bail, Context, Result};
use serde::Serialize;
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::thread;
use which::which;

use crate::config;

#[derive(Debug, Clone, Serialize)]
pub struct HostPlan {
    pub provider: String,
    pub lane: String,
    pub cwd: PathBuf,
    pub asset_rel: String,
    pub project_name: String,
    pub argv: Vec<String>,
    /// Binary hint for logs (`wrangler` / `vercel` / `netlify`).
    pub cli: String,
}

#[derive(Debug, Serialize)]
pub struct HostAuthReport {
    pub ok: bool,
    pub provider: String,
    pub auth_required: bool,
    pub message: String,
}

#[derive(Debug, Serialize)]
pub struct HostDeployReport {
    pub ok: bool,
    pub provider: String,
    pub lane: String,
    pub urls: Vec<String>,
    pub auth_required: bool,
    pub message: String,
}

/// Probe whether the vendor CLI has an interactive login session (no deploy).
pub fn auth_check(provider: &str) -> Result<HostAuthReport> {
    let provider = normalize_provider(provider)?;
    let cli = match provider {
        "cloudflare" => "wrangler",
        "vercel" => "vercel",
        "netlify" => "netlify",
        _ => unreachable!(),
    };
    let bin = match resolve_cli(cli) {
        Ok(b) => b,
        Err(err) => {
            return Ok(HostAuthReport {
                ok: false,
                provider: provider.into(),
                auth_required: false,
                message: format!("{err} [{}]", HostFailClass::MissingCli.as_str()),
            });
        }
    };
    match probe_cli_auth(provider, &bin) {
        Ok(()) => Ok(HostAuthReport {
            ok: true,
            provider: provider.into(),
            auth_required: false,
            message: format!("{provider} CLI session ok"),
        }),
        Err(msg) => Ok(HostAuthReport {
            ok: false,
            provider: provider.into(),
            auth_required: true,
            message: format!("{msg} [{}]", HostFailClass::Auth.as_str()),
        }),
    }
}

fn probe_cli_auth(provider: &str, bin: &Path) -> std::result::Result<(), String> {
    let (args, ok_needles, fail_needles): (&[&str], &[&str], &[&str]) = match provider {
        "cloudflare" => (
            &["whoami"],
            &["logged in", "oauth", "email", "account"],
            &["not logged in", "please log in", "login required", "unauthorized"],
        ),
        "vercel" => (
            &["whoami"],
            &[],
            &["not logged in", "no existing credentials", "log in", "login required"],
        ),
        "netlify" => (
            &["api", "getCurrentUser"],
            &["\"email\"", "\"id\"", "\"full_name\""],
            &[
                "not logged in",
                "please log in",
                "login required",
                "netlify login",
                "unauthorized",
                "please login",
            ],
        ),
        _ => return Err(format!("unsupported provider `{provider}`")),
    };

    let mut child = Command::new(bin)
        .args(args)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| format!("failed to spawn {provider} auth probe: {e}"))?;

    let stdout = child.stdout.take();
    let stderr = child.stderr.take();
    let reader = thread::spawn(move || {
        let mut out = String::new();
        let mut err = String::new();
        if let Some(s) = stdout {
            for line in BufReader::new(s).lines().flatten() {
                out.push_str(&line);
                out.push('\n');
            }
        }
        if let Some(s) = stderr {
            for line in BufReader::new(s).lines().flatten() {
                err.push_str(&line);
                err.push('\n');
            }
        }
        (out, err)
    });

    let started = std::time::Instant::now();
    let timeout = std::time::Duration::from_secs(20);
    let status = loop {
        match child.try_wait() {
            Ok(Some(s)) => break s,
            Ok(None) if started.elapsed() > timeout => {
                let _ = child.kill();
                let _ = child.wait();
                return Err(format!(
                    "{provider} auth probe timed out — run Login CLI (`{} login`)",
                    match provider {
                        "vercel" => "vercel",
                        "netlify" => "netlify",
                        _ => "wrangler",
                    }
                ));
            }
            Ok(None) => thread::sleep(std::time::Duration::from_millis(50)),
            Err(e) => return Err(format!("auth probe wait failed: {e}")),
        }
    };

    let (out, err) = reader.join().unwrap_or_default();
    let text = format!("{out}\n{err}");
    let lower = text.to_ascii_lowercase();
    let login = match provider {
        "vercel" => "vercel",
        "netlify" => "netlify",
        _ => "wrangler",
    };
    if fail_needles.iter().any(|n| lower.contains(n)) || is_auth_failure_inner(&lower) {
        return Err(format!(
            "Not logged in to {provider}. Please run Login CLI (`{login} login`) or Sign in (web), then retry Deploy."
        ));
    }
    if !status.success() {
        return Err(format!(
            "Not logged in to {provider} (exit {}). Please run Login CLI (`{login} login`), then retry Deploy.",
            status.code().unwrap_or(-1)
        ));
    }
    if provider == "vercel" {
        let user = out.trim().to_string();
        if user.is_empty() || user.to_ascii_lowercase().contains("error") {
            return Err(
                "Not logged in to vercel. Please run Login CLI (`vercel login`), then retry Deploy."
                    .into(),
            );
        }
        return Ok(());
    }
    let _ = ok_needles;
    Ok(())
}

fn normalize_provider(raw: &str) -> Result<&'static str> {
    match raw.trim().to_ascii_lowercase().as_str() {
        "cloudflare" | "cf" | "wrangler" => Ok("cloudflare"),
        "vercel" => Ok("vercel"),
        "netlify" => Ok("netlify"),
        other => bail!(
            "hostdeploy unsupported provider `{other}` (cloudflare|vercel|netlify)"
        ),
    }
}

/// Collect https:// URLs; drop loopback / file.
pub fn extract_hosted_urls(text: &str) -> Vec<String> {
    let mut out = Vec::new();
    for raw in text.split_whitespace() {
        let t = raw.trim_matches(|c: char| {
            matches!(c, ',' | ')' | '(' | '[' | ']' | '"' | '\'' | ';' | '.')
        });
        if !t.starts_with("https://") {
            continue;
        }
        let lower = t.to_ascii_lowercase();
        if lower.contains("127.0.0.1")
            || lower.contains("localhost")
            || lower.contains("[::1]")
        {
            continue;
        }
        if !out.iter().any(|u| u == t) {
            out.push(t.to_string());
        }
    }
    for line in text.lines() {
        if let Some(idx) = line.find("https://") {
            let rest = &line[idx..];
            let end = rest
                .find(|c: char| c.is_whitespace() || matches!(c, ')' | ']' | '"' | '\''))
                .unwrap_or(rest.len());
            let t = rest[..end].trim_end_matches(|c: char| matches!(c, '.' | ',' | ';'));
            let lower = t.to_ascii_lowercase();
            if lower.contains("127.0.0.1")
                || lower.contains("localhost")
                || lower.contains("[::1]")
            {
                continue;
            }
            if t.starts_with("https://") && !out.iter().any(|u| u == t) {
                out.push(t.to_string());
            }
        }
    }
    out
}

/// Hosted CLI failure class — one primary recovery each (H4).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HostFailClass {
    Auth,
    MissingCli,
    Build,
    Network,
    Account,
    Detect,
    Unknown,
}

impl HostFailClass {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Auth => "auth",
            Self::MissingCli => "missing_cli",
            Self::Build => "build",
            Self::Network => "network",
            Self::Account => "account",
            Self::Detect => "detect",
            Self::Unknown => "unknown",
        }
    }
}

/// Map vendor CLI / hostdeploy stderr+stdout to a recovery class.
pub fn classify_host_failure(text: &str) -> HostFailClass {
    let l = text.to_ascii_lowercase();
    if l.contains("not on path")
        || (l.contains("not found")
            && (l.contains("wrangler") || l.contains("vercel") || l.contains("netlify")))
        || l.contains("program not found")
        || (l.contains("no such file")
            && (l.contains("wrangler") || l.contains("vercel") || l.contains("netlify")))
        || l.contains("is not recognized")
        || (l.contains("cannot find")
            && (l.contains("wrangler") || l.contains("vercel") || l.contains("netlify")))
    {
        return HostFailClass::MissingCli;
    }
    if l.contains("no wrangler.toml")
        || l.contains("no static index")
        || l.contains("no vercel.json")
        || l.contains("detect failed")
        || l.contains("hostdeploy cloudflare: no ")
        || l.contains("hostdeploy vercel: no ")
        || l.contains("hostdeploy netlify: no ")
    {
        return HostFailClass::Detect;
    }
    if is_auth_failure_inner(&l) {
        return HostFailClass::Auth;
    }
    if l.contains("econnrefused")
        || l.contains("etimedout")
        || l.contains("enotfound")
        || l.contains("network error")
        || l.contains("network request failed")
        || l.contains("could not resolve")
        || l.contains("getaddrinfo")
        || l.contains("tls handshake")
        || l.contains("connection reset")
        || l.contains("timed out")
        || (l.contains("timeout") && l.contains("fetch"))
    {
        return HostFailClass::Network;
    }
    // Missing host project / wrong account / no permission — operator creates on dashboard.
    if l.contains("permission denied")
        || l.contains("access denied")
        || l.contains("forbidden")
        || l.contains("status code 403")
        || l.contains("http 403")
        || l.contains("wrong account")
        || l.contains("not a member")
        || l.contains("insufficient permission")
        || l.contains("does not exist")
        || l.contains("project not found")
        || l.contains("couldn't find project")
        || l.contains("could not find project")
        || l.contains("no such project")
        || l.contains("pages project create")
        || l.contains("project doesn't exist")
        || (l.contains("project") && l.contains("not found") && !l.contains("module not found"))
    {
        return HostFailClass::Account;
    }
    if l.contains("build failed")
        || l.contains("compile error")
        || l.contains("syntax error")
        || l.contains("typescript error")
        || l.contains("failed to compile")
        || l.contains("error ts")
        || l.contains("module not found")
        || l.contains("cannot find module")
        || l.contains("npm err")
    {
        return HostFailClass::Build;
    }
    HostFailClass::Unknown
}

fn is_auth_failure_inner(l: &str) -> bool {
    l.contains("not logged in")
        || l.contains("please log in")
        || l.contains("login required")
        || l.contains("wrangler login")
        || l.contains("vercel login")
        || l.contains("netlify login")
        || l.contains("authentication error")
        || l.contains("unauthorized")
        || l.contains("missing credentials")
        || l.contains("auth required")
        || l.contains("re-authenticate")
        || l.contains("no existing credentials")
        || l.contains("not authenticated")
        || l.contains("please run `vercel login`")
        || l.contains("please run vercel login")
        || l.contains("run `netlify login`")
        || (l.contains("oauth token")
            && (l.contains("invalid") || l.contains("expired") || l.contains("missing")))
}

pub fn is_auth_failure(text: &str) -> bool {
    matches!(classify_host_failure(text), HostFailClass::Auth)
}

/// Primary recovery for Desktop / notes (action id · operator hint).
pub fn recovery_for(class: HostFailClass, provider: &str) -> (&'static str, String) {
    let p = provider.trim().to_ascii_lowercase();
    match class {
        HostFailClass::Auth => (
            "login_cli",
            format!(
                "Sign in with Login CLI (`{} login`) or Sign in (web). Studio never creates API tokens.",
                match p.as_str() {
                    "vercel" => "vercel",
                    "netlify" => "netlify",
                    _ => "wrangler",
                }
            ),
        ),
        HostFailClass::MissingCli => (
            "install_cli",
            format!(
                "Install the CLI on PATH (`{}`), then retry Deploy.",
                match p.as_str() {
                    "vercel" => "npm i -g vercel",
                    "netlify" => "npm i -g netlify-cli",
                    _ => "npm i -g wrangler",
                }
            ),
        ),
        HostFailClass::Build => (
            "preview_log",
            "Build failed — open Output, fix the project, then retry Deploy.".into(),
        ),
        HostFailClass::Network => (
            "retry_deploy",
            "Network error — check connectivity, then retry Deploy.".into(),
        ),
        HostFailClass::Account => (
            "open_dashboard",
            match p.as_str() {
                "vercel" => {
                    "Host project missing or wrong account — Open dashboard to create/select the project, then Retry Deploy.".into()
                }
                "netlify" => {
                    "Host project missing or wrong account — Open dashboard (or `netlify sites:create`), then Retry Deploy.".into()
                }
                _ => {
                    "Pages/Workers still blocked after create attempt — Open dashboard to confirm account/name, then Retry Deploy.".into()
                }
            },
        ),
        HostFailClass::Detect => (
            "open_dashboard",
            "No deployable surface detected — add host config or a static index, or Open dashboard.".into(),
        ),
        HostFailClass::Unknown => (
            "preview_log",
            "Deploy failed — see Output, then Open dashboard or Retry Deploy.".into(),
        ),
    }
}


fn rel_display(project: &Path, path: &Path) -> String {
    path.strip_prefix(project)
        .map(|p| p.display().to_string().replace('\\', "/"))
        .unwrap_or_else(|_| path.display().to_string().replace('\\', "/"))
}

fn sanitize_project_name(raw: &str) -> String {
    let mut s: String = raw
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '-' || c == '_' {
                c.to_ascii_lowercase()
            } else {
                '-'
            }
        })
        .collect();
    while s.contains("--") {
        s = s.replace("--", "-");
    }
    let s = s.trim_matches('-').to_string();
    if s.is_empty() {
        "ship-studio".into()
    } else {
        s.chars().take(58).collect()
    }
}

fn project_name(project: &Path) -> String {
    sanitize_project_name(
        project
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("ship-studio"),
    )
}

fn find_wrangler_toml(project: &Path) -> Option<PathBuf> {
    let candidates = [
        project.join("wrangler.toml"),
        project.join("wrangler.json"),
        project.join("wrangler.jsonc"),
        project.join("apps/api/wrangler.toml"),
        project.join("apps/worker/wrangler.toml"),
        project.join("apps/workers/wrangler.toml"),
    ];
    candidates.into_iter().find(|p| p.is_file())
}

fn find_pages_dir(project: &Path) -> Option<PathBuf> {
    let candidates = [
        "apps/website",
        "apps/docs",
        "public",
        "dist",
        "apps/web",
        ".",
    ];
    for rel in candidates {
        let dir = if rel == "." {
            project.to_path_buf()
        } else {
            project.join(rel)
        };
        if dir.join("index.html").is_file() {
            return Some(dir);
        }
    }
    None
}

fn empty_plan(provider: &str, project: &Path) -> HostPlan {
    HostPlan {
        provider: provider.into(),
        lane: "unknown".into(),
        cwd: project.to_path_buf(),
        asset_rel: String::new(),
        project_name: String::new(),
        argv: vec![],
        cli: provider.into(),
    }
}

pub fn detect_cloudflare(project: &Path) -> Result<HostPlan> {
    let project_name = project_name(project);

    if let Some(toml) = find_wrangler_toml(project) {
        let cwd = toml.parent().unwrap_or(project).to_path_buf();
        return Ok(HostPlan {
            provider: "cloudflare".into(),
            lane: "workers".into(),
            cwd,
            asset_rel: rel_display(project, &toml),
            project_name,
            argv: vec!["deploy".into()],
            cli: "wrangler".into(),
        });
    }

    let Some(dir) = find_pages_dir(project) else {
        bail!(
            "hostdeploy cloudflare: no wrangler.toml and no static index.html \
             (tried apps/website, public, dist). Add a Pages site or Workers config."
        );
    };
    let asset_rel = rel_display(project, &dir);
    Ok(HostPlan {
        provider: "cloudflare".into(),
        lane: "pages".into(),
        cwd: project.to_path_buf(),
        asset_rel: asset_rel.clone(),
        project_name: project_name.clone(),
        argv: vec![
            "pages".into(),
            "deploy".into(),
            asset_rel,
            "--project-name".into(),
            project_name,
            "--commit-dirty=true".into(),
        ],
        cli: "wrangler".into(),
    })
}

pub fn detect_vercel(project: &Path) -> Result<HostPlan> {
    let project_name = project_name(project);
    let has_marker = project.join("vercel.json").is_file()
        || project.join(".vercel").is_dir()
        || project.join("apps/website/vercel.json").is_file()
        || project.join("apps/web/vercel.json").is_file();
    let dir = find_pages_dir(project);
    if !has_marker && dir.is_none() {
        bail!(
            "hostdeploy vercel: no vercel.json / .vercel and no static index.html \
             (tried apps/website, public, dist)."
        );
    }
    let cwd = if project.join("apps/website/vercel.json").is_file() {
        project.join("apps/website")
    } else if project.join("apps/web/vercel.json").is_file() {
        project.join("apps/web")
    } else {
        project.to_path_buf()
    };
    let asset_rel = if has_marker {
        rel_display(project, &cwd)
    } else {
        rel_display(project, dir.as_ref().unwrap())
    };
    // --yes / --prod: non-interactive production deploy (operator already chose Deploy).
    Ok(HostPlan {
        provider: "vercel".into(),
        lane: "prod".into(),
        cwd,
        asset_rel,
        project_name,
        argv: vec!["deploy".into(), "--prod".into(), "--yes".into()],
        cli: "vercel".into(),
    })
}

pub fn detect_netlify(project: &Path) -> Result<HostPlan> {
    let project_name = project_name(project);
    let has_marker = project.join("netlify.toml").is_file()
        || project.join(".netlify").is_dir()
        || project.join("apps/website/netlify.toml").is_file();
    let Some(dir) = find_pages_dir(project) else {
        bail!(
            "hostdeploy netlify: no static index.html \
             (tried apps/website, public, dist). Add a publish directory."
        );
    };
    let _ = has_marker; // marker optional — dir is enough for first prod deploy
    let asset_rel = rel_display(project, &dir);
    Ok(HostPlan {
        provider: "netlify".into(),
        lane: "prod".into(),
        cwd: project.to_path_buf(),
        asset_rel: asset_rel.clone(),
        project_name,
        argv: vec![
            "deploy".into(),
            "--prod".into(),
            "--dir".into(),
            asset_rel,
            "--no-build".into(),
        ],
        cli: "netlify".into(),
    })
}

pub fn detect_provider(project: &Path, provider: &str) -> Result<HostPlan> {
    match normalize_provider(provider)? {
        "cloudflare" => detect_cloudflare(project),
        "vercel" => detect_vercel(project),
        "netlify" => detect_netlify(project),
        _ => unreachable!(),
    }
}

fn resolve_cli(cli: &str) -> Result<PathBuf> {
    let candidates: &[&str] = match cli {
        "wrangler" => &["wrangler", "wrangler.cmd", "wrangler.exe"],
        "vercel" => &["vercel", "vercel.cmd", "vercel.exe"],
        "netlify" => &["netlify", "netlify.cmd", "netlify.exe"],
        other => bail!("unknown CLI `{other}`"),
    };
    for name in candidates {
        if let Ok(p) = which(name) {
            return Ok(p);
        }
    }
    let hint = match cli {
        "wrangler" => "npm i -g wrangler",
        "vercel" => "npm i -g vercel",
        "netlify" => "npm i -g netlify-cli",
        _ => "install the vendor CLI",
    };
    bail!("{cli} not on PATH — install with `{hint}` or use Login CLI after install")
}

fn run_cli_stream(bin: &Path, plan: &HostPlan) -> Result<(i32, String)> {
    println!(
        "hostdeploy · {} · {} · {}",
        plan.provider, plan.lane, plan.asset_rel
    );
    println!("hostdeploy · cwd {}", plan.cwd.display());
    println!(
        "hostdeploy · {} {}",
        bin.display(),
        plan.argv.join(" ")
    );
    println!("hostdeploy · streaming {}…", plan.cli);

    let mut child = Command::new(bin)
        .current_dir(&plan.cwd)
        .args(&plan.argv)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .with_context(|| format!("spawn {}", bin.display()))?;

    let stdout = child.stdout.take().context("cli stdout")?;
    let stderr = child.stderr.take().context("cli stderr")?;
    let collected = std::sync::Arc::new(std::sync::Mutex::new(String::new()));
    let c_out = collected.clone();
    let c_err = collected.clone();

    let t_out = thread::spawn(move || {
        let reader = BufReader::new(stdout);
        for line in reader.lines().flatten() {
            println!("{line}");
            let _ = std::io::stdout().flush();
            if let Ok(mut buf) = c_out.lock() {
                buf.push_str(&line);
                buf.push('\n');
            }
        }
    });
    let t_err = thread::spawn(move || {
        let reader = BufReader::new(stderr);
        for line in reader.lines().flatten() {
            eprintln!("{line}");
            let _ = std::io::stderr().flush();
            if let Ok(mut buf) = c_err.lock() {
                buf.push_str(&line);
                buf.push('\n');
            }
        }
    });

    let status = child.wait().context("wait cli")?;
    let _ = t_out.join();
    let _ = t_err.join();
    let code = status.code().unwrap_or(1);
    let text = collected
        .lock()
        .map(|g| g.clone())
        .unwrap_or_default();
    Ok((code, text))
}

fn write_run(
    project: &Path,
    started: String,
    ok: bool,
    message: String,
    urls: Vec<String>,
    plan: &HostPlan,
    exit_code: i32,
    auth_required: bool,
) -> Result<()> {
    let finished = config::now_rfc3339();
    let deploy_ok = ok;
    let url_ok = ok && !urls.is_empty();
    config::write_last_run(
        project,
        &config::LastRun {
            finished: true,
            ok,
            started_at: started,
            finished_at: finished,
            dry_run: false,
            offline: false,
            steps: vec![
                config::StepResult {
                    id: "host.detect".into(),
                    ok: true,
                    exit_code: 0,
                    detail: format!("{} · {} · {}", plan.provider, plan.lane, plan.asset_rel),
                },
                config::StepResult {
                    id: "host.deploy".into(),
                    ok: deploy_ok,
                    exit_code,
                    detail: if auth_required {
                        "auth required — Login CLI or Sign in (web)".into()
                    } else {
                        format!("{} {}", plan.cli, plan.argv.join(" "))
                    },
                },
                config::StepResult {
                    id: "host.url".into(),
                    ok: url_ok,
                    exit_code: if url_ok { 0 } else { 1 },
                    detail: if urls.is_empty() {
                        format!("no https URL parsed from {} output", plan.cli)
                    } else {
                        urls.join(" · ")
                    },
                },
            ],
            message,
            urls,
            host_provider: Some(plan.provider.clone()),
        },
    )?;
    Ok(())
}

fn login_hint(provider: &str) -> &'static str {
    match provider {
        "cloudflare" => "Login CLI (`wrangler login`) or Sign in (web)",
        "vercel" => "Login CLI (`vercel login`) or Sign in (web)",
        "netlify" => "Login CLI (`netlify login`) or Sign in (web)",
        _ => "Login CLI or Sign in (web)",
    }
}

/// True when wrangler Pages deploy failed because the named project is absent.
pub fn is_missing_pages_project(text: &str) -> bool {
    let l = text.to_ascii_lowercase();
    if !(l.contains("does not exist")
        || l.contains("project not found")
        || l.contains("no such project")
        || l.contains("project doesn't exist"))
    {
        return false;
    }
    l.contains("pages") || l.contains("project name") || l.contains("pages/projects")
}

fn pages_create_plan(plan: &HostPlan) -> HostPlan {
    HostPlan {
        provider: plan.provider.clone(),
        lane: "pages-create".into(),
        cwd: plan.cwd.clone(),
        asset_rel: plan.asset_rel.clone(),
        project_name: plan.project_name.clone(),
        argv: vec![
            "pages".into(),
            "project".into(),
            "create".into(),
            plan.project_name.clone(),
            "--production-branch".into(),
            "main".into(),
        ],
        cli: plan.cli.clone(),
    }
}

fn create_ok_from_output(code: i32, text: &str) -> bool {
    if code == 0 {
        return true;
    }
    let l = text.to_ascii_lowercase();
    // Idempotent: project already there is fine — retry deploy.
    l.contains("already exists") || l.contains("already created")
}

fn apply_project_name(plan: &mut HostPlan, name: &str) {
    let name = sanitize_project_name(name);
    if name.is_empty() {
        return;
    }
    plan.project_name = name.clone();
    // Rewrite --project-name <x> in argv when present (Cloudflare Pages).
    let mut i = 0;
    while i + 1 < plan.argv.len() {
        if plan.argv[i] == "--project-name" {
            plan.argv[i + 1] = name.clone();
            return;
        }
        i += 1;
    }
    // Netlify: prefer --site <name>; create-if-missing uses --create-site on retry.
    if plan.provider == "netlify" {
        let mut i = 0;
        while i + 1 < plan.argv.len() {
            if plan.argv[i] == "--site" || plan.argv[i] == "--create-site" {
                plan.argv[i + 1] = name.clone();
                return;
            }
            i += 1;
        }
        plan.argv.push("--site".into());
        plan.argv.push(name);
    }
}

fn is_missing_netlify_site(text: &str) -> bool {
    let l = text.to_ascii_lowercase();
    l.contains("isn't linked")
        || l.contains("is not linked")
        || l.contains("not linked to a project")
        || l.contains("create & configure a new project")
        || l.contains("create-site")
        || (l.contains("site not found") && l.contains("netlify"))
        || l.contains("could not find the project")
        || l.contains("project not found")
}

fn netlify_create_and_deploy_plan(plan: &HostPlan) -> HostPlan {
    let mut argv = vec!["deploy".into(), "--prod".into()];
    // Keep --dir <path> from original plan.
    let mut i = 0;
    while i + 1 < plan.argv.len() {
        if plan.argv[i] == "--dir" {
            argv.push("--dir".into());
            argv.push(plan.argv[i + 1].clone());
            break;
        }
        i += 1;
    }
    if !argv.iter().any(|a| a == "--dir") {
        argv.push("--dir".into());
        argv.push(plan.asset_rel.clone());
    }
    argv.push("--no-build".into());
    argv.push("--create-site".into());
    argv.push(plan.project_name.clone());
    HostPlan {
        argv,
        ..plan.clone()
    }
}

pub fn run(project: &Path, provider: &str, name_override: Option<&str>) -> Result<HostDeployReport> {
    let provider = normalize_provider(provider)?;
    let project = project
        .canonicalize()
        .with_context(|| format!("canonicalize {}", project.display()))?;
    let started = config::now_rfc3339();
    println!("hostdeploy · starting {provider} CLI lane");
    println!("hostdeploy · project {}", project.display());
    println!("hostdeploy · detecting…");

    let mut plan = match detect_provider(&project, provider) {
        Ok(p) => p,
        Err(err) => {
            let message = err.to_string();
            println!("hostdeploy · detect failed · {message}");
            let empty = empty_plan(provider, &project);
            write_run(
                &project,
                started,
                false,
                message.clone(),
                vec![],
                &empty,
                1,
                false,
            )?;
            bail!("{message}");
        }
    };
    if let Some(n) = name_override {
        apply_project_name(&mut plan, n);
        println!("hostdeploy · project-name override · {}", plan.project_name);
    }

    // Unlinked Netlify folder + named site → create-site up front (avoids interactive link prompt).
    if plan.provider == "netlify"
        && !plan.project_name.is_empty()
        && !project.join(".netlify").is_dir()
    {
        plan = netlify_create_and_deploy_plan(&plan);
        println!(
            "hostdeploy · Netlify unlinked — using --create-site {}",
            plan.project_name
        );
    }

    println!(
        "hostdeploy · check · detect ok · {} · {}",
        plan.lane, plan.asset_rel
    );

    let bin = match resolve_cli(&plan.cli) {
        Ok(b) => b,
        Err(err) => {
            let message = err.to_string();
            println!("hostdeploy · {message}");
            write_run(
                &project,
                started,
                false,
                message.clone(),
                vec![],
                &plan,
                1,
                true,
            )?;
            let class = HostFailClass::MissingCli;
            let message = format!("{} [{}]", message, class.as_str());
            return Ok(HostDeployReport {
                ok: false,
                provider: plan.provider,
                lane: plan.lane,
                urls: vec![],
                auth_required: false,
                message,
            });
        }
    };

    // Auth gate — fail fast to Login CLI instead of streaming an interactive prompt.
    if let Err(msg) = probe_cli_auth(provider, &bin) {
        println!("hostdeploy · auth required · {msg}");
        write_run(
            &project,
            started,
            false,
            msg.clone(),
            vec![],
            &plan,
            1,
            false,
        )?;
        return Ok(HostDeployReport {
            ok: false,
            provider: plan.provider,
            lane: plan.lane,
            urls: vec![],
            auth_required: true,
            message: format!("{msg} [{}]", HostFailClass::Auth.as_str()),
        });
    }
    println!("hostdeploy · auth ok · {provider}");

    let (mut code, mut output) = run_cli_stream(&bin, &plan)?;

    // Cloudflare Pages: create-if-missing then one retry (no dashboard trip).
    if code != 0
        && plan.provider == "cloudflare"
        && plan.lane == "pages"
        && !plan.project_name.is_empty()
        && is_missing_pages_project(&output)
    {
        println!(
            "hostdeploy · Pages project `{}` missing — creating via wrangler (then retry Deploy)…",
            plan.project_name
        );
        let create = pages_create_plan(&plan);
        let (cc, cout) = run_cli_stream(&bin, &create)?;
        output.push('\n');
        output.push_str(&cout);
        if create_ok_from_output(cc, &cout) {
            println!("hostdeploy · create ok · retrying pages deploy…");
            let (c2, o2) = run_cli_stream(&bin, &plan)?;
            code = c2;
            output.push('\n');
            output.push_str(&o2);
        } else {
            println!("hostdeploy · create failed · exit {cc}");
        }
    }

    // Netlify: unlinked folder / missing site → create-site <name> then done (one shot).
    if code != 0
        && plan.provider == "netlify"
        && !plan.project_name.is_empty()
        && is_missing_netlify_site(&output)
    {
        println!(
            "hostdeploy · Netlify site `{}` missing/unlinked — create-site + deploy…",
            plan.project_name
        );
        let create = netlify_create_and_deploy_plan(&plan);
        let (c2, o2) = run_cli_stream(&bin, &create)?;
        code = c2;
        output.push('\n');
        output.push_str(&o2);
    }

    let urls = extract_hosted_urls(&output);
    let lower = output.to_ascii_lowercase();
    let mut class = classify_host_failure(&output);
    if code != 0
        && matches!(class, HostFailClass::Unknown)
        && lower.contains("login")
        && (lower.contains("wrangler") || lower.contains("vercel") || lower.contains("netlify"))
    {
        class = HostFailClass::Auth;
    }
    let auth_required = matches!(class, HostFailClass::Auth);

    // Cloudflare Workers may succeed without a printed URL; others need https.
    let ok = code == 0
        && (!urls.is_empty() || (plan.provider == "cloudflare" && plan.lane == "workers"));
    let message = if ok {
        format!(
            "hostdeploy {} {} ok · {}",
            plan.provider,
            plan.lane,
            urls.first().map(|s| s.as_str()).unwrap_or("(see dashboard)")
        )
    } else if auth_required {
        format!(
            "hostdeploy {}: auth required — {} [{}]",
            plan.provider,
            login_hint(provider),
            class.as_str()
        )
    } else if code == 0 && urls.is_empty() {
        format!(
            "hostdeploy {}: {} exited 0 but no https URL in output — Open dashboard to confirm [unknown]",
            plan.provider, plan.cli
        )
    } else {
        let (_action, hint) = recovery_for(class, provider);
        format!(
            "hostdeploy {}: {} exited {code} — {hint} [{}]",
            plan.provider, plan.cli, class.as_str()
        )
    };

    if ok {
        println!(
            "hostdeploy · check ok · {}",
            urls.first().map(|s| s.as_str()).unwrap_or("(see dashboard)")
        );
    } else {
        println!("hostdeploy · failed · {message}");
    }

    write_run(
        &project,
        started,
        ok,
        message.clone(),
        urls.clone(),
        &plan,
        code,
        auth_required,
    )?;

    let report = HostDeployReport {
        ok,
        provider: plan.provider,
        lane: plan.lane,
        urls,
        auth_required,
        message: message.clone(),
    };

    if !ok {
        bail!("{message}");
    }
    Ok(report)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn tmp() -> PathBuf {
        let dir = std::env::temp_dir().join(format!(
            "ship-hostdeploy-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn extract_hosted_urls_keeps_https_drops_loopback() {
        let text = r#"
Deployed to https://harbor.pages.dev
Also http://127.0.0.1:8787/ preview
Visit https://harbor.pages.dev/index.html
"#;
        let urls = extract_hosted_urls(text);
        assert!(urls.iter().any(|u| u.contains("harbor.pages.dev")));
        assert!(!urls.iter().any(|u| u.contains("127.0.0.1")));
    }

    #[test]
    fn auth_failure_detects_vendor_login() {
        assert!(is_auth_failure("Error: Not logged in. Run wrangler login."));
        assert!(is_auth_failure("Error: No existing credentials found. Please run `vercel login`."));
        assert!(is_auth_failure("Not logged in. Please run `netlify login`."));
        assert!(!is_auth_failure("Uploaded 3 files to pages.dev"));
    }

    #[test]
    fn classify_host_failure_taxonomy() {
        assert_eq!(
            classify_host_failure("Error: Not logged in. Run wrangler login."),
            HostFailClass::Auth
        );
        assert_eq!(
            classify_host_failure("wrangler not on PATH — install with `npm i -g wrangler`"),
            HostFailClass::MissingCli
        );
        assert_eq!(
            classify_host_failure("Error: getaddrinfo ENOTFOUND api.cloudflare.com"),
            HostFailClass::Network
        );
        assert_eq!(
            classify_host_failure("Failed to compile.\nModule not found: './x'"),
            HostFailClass::Build
        );
        assert_eq!(
            classify_host_failure("HTTP 403 Forbidden — insufficient permission"),
            HostFailClass::Account
        );
        assert_eq!(
            classify_host_failure(
                r#"ERROR The Pages project "harbor" does not exist. Maybe you intended to deploy a Worker? wrangler pages project create"#
            ),
            HostFailClass::Account
        );
        assert_eq!(
            classify_host_failure("Error: Project not found. Check the project name."),
            HostFailClass::Account
        );
        assert!(is_missing_pages_project(
            r#"ERROR The Pages project "harbor" does not exist."#
        ));
        assert!(!is_missing_pages_project("module not found: './x'"));
        assert!(create_ok_from_output(0, "ok"));
        assert!(create_ok_from_output(1, "Error: A project with that name already exists."));
        assert!(!create_ok_from_output(1, "authentication error"));
        assert_eq!(
            classify_host_failure(
                "hostdeploy cloudflare: no wrangler.toml and no static index.html"
            ),
            HostFailClass::Detect
        );
        let (action, hint) = recovery_for(HostFailClass::Auth, "cloudflare");
        assert_eq!(action, "login_cli");
        assert!(hint.to_lowercase().contains("login"));
    }

    #[test]
    fn detect_pages_from_website_index() {
        let dir = tmp();
        fs::create_dir_all(dir.join("apps/website")).unwrap();
        fs::write(dir.join("apps/website/index.html"), "<h1>ok</h1>").unwrap();
        let plan = detect_cloudflare(&dir).unwrap();
        assert_eq!(plan.lane, "pages");
        assert!(plan.argv.contains(&"pages".into()));
        assert!(plan.asset_rel.contains("apps/website"));
    }

    #[test]
    fn detect_workers_from_wrangler_toml() {
        let dir = tmp();
        fs::create_dir_all(dir.join("apps/api")).unwrap();
        fs::write(dir.join("apps/api/wrangler.toml"), "name = \"api\"\n").unwrap();
        let plan = detect_cloudflare(&dir).unwrap();
        assert_eq!(plan.lane, "workers");
        assert_eq!(plan.argv, vec!["deploy".to_string()]);
    }

    #[test]
    fn detect_vercel_from_static_or_marker() {
        let dir = tmp();
        fs::create_dir_all(dir.join("apps/website")).unwrap();
        fs::write(dir.join("apps/website/index.html"), "<h1>ok</h1>").unwrap();
        let plan = detect_vercel(&dir).unwrap();
        assert_eq!(plan.provider, "vercel");
        assert!(plan.argv.iter().any(|a| a == "--prod"));
        assert!(plan.argv.iter().any(|a| a == "--yes"));
    }

    #[test]
    fn detect_netlify_from_static_dir() {
        let dir = tmp();
        fs::create_dir_all(dir.join("public")).unwrap();
        fs::write(dir.join("public/index.html"), "<h1>ok</h1>").unwrap();
        let plan = detect_netlify(&dir).unwrap();
        assert_eq!(plan.provider, "netlify");
        assert!(plan.argv.iter().any(|a| a == "--prod"));
        assert!(plan.argv.iter().any(|a| a == "public"));
    }

    #[test]
    fn apply_project_name_adds_netlify_site_flag() {
        let dir = tmp();
        fs::create_dir_all(dir.join("dist")).unwrap();
        fs::write(dir.join("dist/index.html"), "<h1>ok</h1>").unwrap();
        let mut plan = detect_netlify(&dir).unwrap();
        apply_project_name(&mut plan, "Ship Studio Site!");
        assert_eq!(plan.project_name, "ship-studio-site");
        let site_idx = plan
            .argv
            .iter()
            .position(|a| a == "--site")
            .expect("--site");
        assert_eq!(plan.argv.get(site_idx + 1).map(String::as_str), Some("ship-studio-site"));
        let create = netlify_create_and_deploy_plan(&plan);
        assert!(create.argv.iter().any(|a| a == "--create-site"));
        assert!(create.argv.iter().any(|a| a == "ship-studio-site"));
        assert!(is_missing_netlify_site(
            "This folder isn't linked to a project yet\nTo create and deploy in one go, use: netlify deploy --create-site"
        ));
    }

    #[test]
    fn auth_check_reports_json_shape_when_cli_missing() {
        // Force missing CLI by probing a fake provider path via normalize only —
        // when wrangler/vercel/netlify exist this still returns a HostAuthReport.
        let report = auth_check("netlify").expect("auth_check returns");
        assert_eq!(report.provider, "netlify");
        assert!(!report.message.is_empty());
        // Either logged in (ok) or auth_required / missing_cli.
        if !report.ok {
            assert!(
                report.auth_required
                    || report.message.contains("[missing_cli]")
                    || report.message.contains("[auth]"),
                "unexpected message: {}",
                report.message
            );
        }
    }
}
