//! Local Ship bridge — Signet + Orbit adapters, offline-first.

mod adapters;
mod config;
mod flow;
mod guide;
mod human;
mod mcp;
mod portal;
mod secrets;
mod ship;
mod tui;
mod vault_km;

use anyhow::{bail, Context, Result};
use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(name = "shipctl", version, about = "Local Ship workflow bridge (Signet + Orbit)")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Check local tooling and project readiness (offline).
    Doctor {
        #[arg(long, default_value = ".")]
        project: PathBuf,
    },
    /// Rapid env intent: write .ship/studio.json and print next steps.
    Configure {
        #[arg(long, default_value = ".")]
        project: PathBuf,
    },
    /// Provider portal: entry URLs + OAuth/dashboard (Cloudflare/Vercel/Netlify/GitHub/Polar).
    Portal {
        #[arg(long, default_value = ".")]
        project: PathBuf,
        /// Filter to one provider (cloudflare|vercel|netlify|github|polar).
        #[arg(long)]
        provider: Option<String>,
        /// Open token/credential pages in the system browser.
        #[arg(long, default_value_t = false)]
        open: bool,
        /// Run CLI OAuth login for planned providers (interactive).
        #[arg(long, default_value_t = false)]
        login: bool,
    },
    /// Unified offline shipping checklist (doctor + portal + secrets + flow plan).
    Guide {
        #[arg(long, default_value = ".")]
        project: PathBuf,
        /// Open all unique provider/marketplace entry URLs in the browser.
        #[arg(long, default_value_t = false)]
        open: bool,
    },
    /// One-shot offline prep: guide → configure → flow dry-run (optional --open).
    Ship {
        #[arg(long, default_value = ".")]
        project: PathBuf,
        /// Open provider/marketplace entry URLs after prep.
        #[arg(long, default_value_t = false)]
        open: bool,
    },
    /// Human portal sprint: open dashboards in priority order, then paste secrets.
    Human {
        #[arg(long, default_value = ".")]
        project: PathBuf,
        /// Open paste-source pages (Polar/GitHub…) — default true.
        #[arg(long, default_value_t = true)]
        open: bool,
        /// Skip opening browsers.
        #[arg(long, default_value_t = false)]
        no_open: bool,
        /// Also open every guide entry URL (CF/Vercel dashboards), not only paste sources.
        #[arg(long, default_value_t = false)]
        open_all: bool,
        /// Interactively put each queued secret (paste in terminal).
        #[arg(long, default_value_t = false)]
        put: bool,
    },
    /// Interactive TUI shipping portal (CLI companion to the desktop app).
    Tui {
        #[arg(long, default_value = ".")]
        project: PathBuf,
    },
    /// Paste-secret assist: list hinted names / put via provider CLI (never stores values).
    Secrets {
        #[arg(long, default_value = ".")]
        project: PathBuf,
        #[arg(long)]
        provider: Option<String>,
        /// Open token/credential pages once.
        #[arg(long, default_value_t = false)]
        open: bool,
        /// Interactively put this secret name (requires --provider).
        #[arg(long)]
        put: Option<String>,
    },
    /// Encrypted vault.km export (Clavis / Keys Manager compatible).
    Vault {
        #[command(subcommand)]
        action: VaultCmd,
    },
    /// Run Signet in the project (pass-through args after --).
    Sign {
        #[arg(long, default_value = ".")]
        project: PathBuf,
        #[arg(long)]
        offline: bool,
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },
    /// Run Orbit in the project (default: ship).
    Deploy {
        #[arg(long, default_value = ".")]
        project: PathBuf,
        #[arg(long)]
        offline: bool,
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },
    /// configure → sign → deploy (or print plan with --dry-run).
    Flow {
        #[arg(long, default_value = ".")]
        project: PathBuf,
        #[arg(long)]
        dry_run: bool,
        #[arg(long)]
        offline: bool,
        #[arg(long, default_value_t = false)]
        skip_sign: bool,
        #[arg(long, default_value_t = false)]
        skip_deploy: bool,
    },
    /// Show last .ship/last-run.json if any.
    Status {
        #[arg(long, default_value = ".")]
        project: PathBuf,
    },
    /// Stdio MCP server (tools: doctor, configure, portal, sign, deploy, flow, status).
    Mcp,
}

#[derive(Subcommand, Debug)]
enum VaultCmd {
    /// Create a new encrypted .km vault (passphrase via TTY or SHIP_VAULT_PASSPHRASE).
    Export {
        #[arg(long, default_value = ".")]
        project: PathBuf,
        /// Output path (e.g. ./ship-secrets.km).
        #[arg(long)]
        out: PathBuf,
        /// Vault display name inside the file.
        #[arg(long, default_value = "Ship Studio secrets")]
        name: String,
        /// Pre-fill entry titles from project secret hints (values still prompted).
        #[arg(long, default_value_t = false)]
        from_hints: bool,
        /// Non-interactive: single entry title (requires --value-env).
        #[arg(long)]
        title: Option<String>,
        /// Non-interactive: read secret from this env var (with --title).
        #[arg(long)]
        value_env: Option<String>,
        #[arg(long, default_value = "")]
        url: String,
        /// Non-interactive: JSON array of {title,value,url?,notes?} (passphrase via env/TTY).
        #[arg(long)]
        entries_file: Option<PathBuf>,
    },
    /// Append one secret to an existing vault.
    Add {
        #[arg(long)]
        file: PathBuf,
        #[arg(long)]
        title: String,
        #[arg(long, default_value = "")]
        url: String,
        /// Read secret from this env var instead of prompting.
        #[arg(long)]
        value_env: Option<String>,
    },
    /// List entry titles (unlocks; never prints values).
    List {
        #[arg(long)]
        file: PathBuf,
    },
    /// Print one entry value to stdout (operator terminal only).
    Show {
        #[arg(long)]
        file: PathBuf,
        #[arg(long)]
        title: String,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Commands::Doctor { project } => {
            let report = adapters::doctor(&project)?;
            println!("{}", serde_json::to_string_pretty(&report)?);
            if !report.ok {
                bail!("doctor found problems");
            }
        }
        Commands::Configure { project } => {
            let intent = config::configure(&project)?;
            println!("{}", serde_json::to_string_pretty(&intent)?);
        }
        Commands::Portal {
            project,
            provider,
            open,
            login,
        } => {
            let filter = provider
                .as_deref()
                .map(portal::ProviderId::parse)
                .transpose()?;
            let plan = portal::plan_for(&project, filter)?;
            if open {
                let opened = portal::open_urls(&plan)?;
                eprintln!("opened {} url(s)", opened.len());
            }
            let login_results = if login {
                Some(portal::run_logins(&project, &plan)?)
            } else {
                None
            };
            let mut out = serde_json::to_value(&plan)?;
            if let Some(lr) = login_results {
                out["login_results"] = serde_json::Value::Array(lr);
            }
            println!("{}", serde_json::to_string_pretty(&out)?);
        }
        Commands::Guide { project, open } => {
            let plan = guide::plan_for(&project)?;
            // Keep desktop/status in sync with latest checklist.
            let dir = config::ship_dir(&project);
            let _ = std::fs::create_dir_all(&dir);
            let _ = std::fs::write(
                dir.join("last-guide.json"),
                serde_json::to_string_pretty(&plan)?,
            );
            if open {
                let opened = guide::open_entries(&plan)?;
                eprintln!("opened {} entry url(s)", opened.len());
            }
            println!("{}", serde_json::to_string_pretty(&plan)?);
        }
        Commands::Ship { project, open } => {
            let prep = ship::run(&project, open)?;
            println!("{}", serde_json::to_string_pretty(&prep)?);
        }
        Commands::Human {
            project,
            open,
            no_open,
            open_all,
            put,
        } => {
            let do_open = open && !no_open;
            let sprint = human::run_with_options(&project, do_open, put, open_all)?;
            println!("{}", serde_json::to_string_pretty(&sprint)?);
        }
        Commands::Tui { project } => tui::run(&project)?,
        Commands::Secrets {
            project,
            provider,
            open,
            put,
        } => {
            let filter = provider
                .as_deref()
                .map(portal::ProviderId::parse)
                .transpose()?;
            if let Some(name) = put {
                let id = filter.context("--put requires --provider")?;
                let code = secrets::put_secret(&project, id, &name)?;
                if code != 0 {
                    bail!("secrets put exited {code}");
                }
            } else {
                let plan = secrets::plan_for(&project, filter)?;
                if open {
                    let opened = secrets::open_entry_urls(&plan)?;
                    eprintln!("opened {} url(s)", opened.len());
                }
                println!("{}", serde_json::to_string_pretty(&plan)?);
            }
        }
        Commands::Vault { action } => match action {
            VaultCmd::Export {
                project,
                out,
                name,
                from_hints,
                title,
                value_env,
                url,
                entries_file,
            } => {
                let path = if let Some(entries_path) = entries_file {
                    let entries = vault_km::load_entries_file(&entries_path)?;
                    vault_km::export_entries(&out, &name, &entries)?
                } else if let (Some(title), Some(env_name)) = (title, value_env) {
                    let mut value =
                        std::env::var(&env_name).with_context(|| format!("env {env_name}"))?;
                    if value.is_empty() {
                        bail!("env {env_name} is empty");
                    }
                    let path = vault_km::export_one(&out, &name, &title, &value, &url)?;
                    value.clear();
                    path
                } else {
                    let hints: Vec<String> = if from_hints {
                        secrets::plan_for(&project, None)?
                            .hints
                            .into_iter()
                            .map(|h| h.name)
                            .filter(|n| !n.is_empty() && !n.contains('<'))
                            .collect()
                    } else {
                        Vec::new()
                    };
                    vault_km::export_interactive(&out, &name, &hints)?
                };
                println!(
                    "{}",
                    serde_json::json!({
                        "ok": true,
                        "path": path,
                        "format": "kmvault/v1",
                        "compatible_with": "Clavis / Keys Manager",
                        "hint": "Open the .km file in Clavis, or shipctl vault list --file …"
                    })
                );
            }
            VaultCmd::Add {
                file,
                title,
                url,
                value_env,
            } => {
                let mut value = if let Some(env) = value_env {
                    std::env::var(&env).with_context(|| format!("env {env}"))?
                } else {
                    eprint!("Secret value: ");
                    let _ = std::io::Write::flush(&mut std::io::stderr());
                    rpassword::read_password().context("read secret")?
                };
                if value.is_empty() {
                    bail!("empty secret value");
                }
                vault_km::add_secret(
                    &file,
                    &title,
                    &value,
                    &url,
                    "Added via shipctl vault add",
                )?;
                value.clear();
                println!(
                    "{}",
                    serde_json::json!({ "ok": true, "file": file, "title": title })
                );
            }
            VaultCmd::List { file } => {
                let titles = vault_km::list_titles(&file)?;
                println!(
                    "{}",
                    serde_json::to_string_pretty(&serde_json::json!({
                        "file": file,
                        "titles": titles,
                    }))?
                );
            }
            VaultCmd::Show { file, title } => {
                let mut value = vault_km::show_value(&file, &title)?;
                print!("{value}");
                value.clear();
            }
        },
        Commands::Sign {
            project,
            offline,
            args,
        } => {
            let intent = config::intent_for(&project)?;
            let args = if args.is_empty() {
                if offline {
                    vec!["doctor".into(), "--json".into()]
                } else {
                    intent.sign_args
                }
            } else if offline
                && args
                    .iter()
                    .any(|a| a == "release" || a == "graduate" || a == "ship")
            {
                bail!("sign --offline refuses network-ish subcommands; use doctor/scan/inspect");
            } else {
                args
            };
            let code = adapters::run_signet(&project, &args)?;
            if code != 0 {
                bail!("signet exited {code}");
            }
        }
        Commands::Deploy {
            project,
            offline,
            args,
        } => {
            if offline {
                bail!("deploy requires network via orbit; refuse --offline");
            }
            let intent = config::intent_for(&project)?;
            let args = if args.is_empty() {
                intent.deploy_args
            } else {
                args
            };
            let code = adapters::run_orbit(&project, &args)?;
            if code != 0 {
                bail!("orbit exited {code}");
            }
        }
        Commands::Flow {
            project,
            dry_run,
            offline,
            skip_sign,
            skip_deploy,
        } => {
            let plan = flow::plan(&project, skip_sign, skip_deploy, offline)?;
            if dry_run {
                println!("{}", serde_json::to_string_pretty(&plan)?);
                return Ok(());
            }
            flow::execute(&project, &plan).context("flow failed")?;
        }
        Commands::Status { project } => {
            let status = config::read_last_run(&project)?;
            println!("{}", serde_json::to_string_pretty(&status)?);
        }
        Commands::Mcp => mcp::serve()?,
    }
    Ok(())
}
