//! Local Ship bridge — Signet + Orbit adapters, offline-first.

mod adapters;
mod config;
mod flow;
mod mcp;

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
    /// Stdio MCP server (tools: doctor, configure, sign, deploy, flow, status).
    Mcp,
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
        Commands::Sign {
            project,
            offline,
            args,
        } => {
            let intent = config::intent_for(&project)?;
            let args = if args.is_empty() {
                if offline {
                    // Offline-safe local check (no release/publish).
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
