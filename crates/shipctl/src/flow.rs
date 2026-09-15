use crate::adapters;
use crate::config::{self, LastRun, StepResult};
use anyhow::{bail, Result};
use serde::Serialize;
use std::path::Path;

#[derive(Debug, Serialize)]
pub struct FlowPlan {
    pub project: String,
    pub offline: bool,
    pub steps: Vec<FlowStep>,
}

#[derive(Debug, Serialize)]
pub struct FlowStep {
    pub id: String,
    pub adapter: String,
    pub args: Vec<String>,
    pub network: bool,
}

pub fn plan(
    project: &Path,
    skip_sign: bool,
    skip_deploy: bool,
    offline: bool,
) -> Result<FlowPlan> {
    let project = std::fs::canonicalize(project).unwrap_or_else(|_| project.to_path_buf());
    let intent = config::intent_for(&project)?;

    let mut steps = vec![FlowStep {
        id: "configure".into(),
        adapter: "shipctl".into(),
        args: vec!["configure".into()],
        network: false,
    }];
    if !skip_sign {
        steps.push(FlowStep {
            id: "sign".into(),
            adapter: "signet".into(),
            args: intent.sign_args.clone(),
            network: false,
        });
    }
    if !skip_deploy {
        if offline {
            bail!("flow --offline cannot include deploy; pass --skip-deploy");
        }
        steps.push(FlowStep {
            id: "deploy".into(),
            adapter: "orbit".into(),
            args: intent.deploy_args.clone(),
            network: true,
        });
    }
    Ok(FlowPlan {
        project: project.display().to_string(),
        offline,
        steps,
    })
}

pub fn execute(project: &Path, plan: &FlowPlan) -> Result<()> {
    let started_at = config::now_rfc3339();
    let mut results: Vec<StepResult> = Vec::new();

    // Always refresh / merge studio.json first.
    config::configure(project)?;
    results.push(StepResult {
        id: "configure".into(),
        ok: true,
        exit_code: 0,
        detail: "wrote .ship/studio.json".into(),
    });

    for step in &plan.steps {
        match step.id.as_str() {
            "configure" => {}
            "sign" => {
                let code = adapters::run_signet(project, &step.args)?;
                let ok = code == 0;
                results.push(StepResult {
                    id: "sign".into(),
                    ok,
                    exit_code: code,
                    detail: format!("signet {}", step.args.join(" ")),
                });
                if !ok {
                    write_failure(project, plan, &started_at, results, "signet step failed")?;
                    bail!("signet step failed with exit {code}");
                }
            }
            "deploy" => {
                let code = adapters::run_orbit(project, &step.args)?;
                let ok = code == 0;
                results.push(StepResult {
                    id: "deploy".into(),
                    ok,
                    exit_code: code,
                    detail: format!("orbit {}", step.args.join(" ")),
                });
                if !ok {
                    write_failure(project, plan, &started_at, results, "orbit step failed")?;
                    bail!("orbit step failed with exit {code}");
                }
            }
            other => bail!("unknown step {other}"),
        }
    }

    let finished_at = config::now_rfc3339();
    let ids: Vec<_> = results.iter().map(|s| s.id.as_str()).collect();
    config::write_last_run(
        project,
        &LastRun {
            finished: true,
            ok: true,
            started_at,
            finished_at,
            dry_run: false,
            offline: plan.offline,
            steps: results.clone(),
            message: format!("completed steps: {}", ids.join(" → ")),
            urls: crate::pulse::latest_live_urls(project),
        },
    )?;
    Ok(())
}

fn write_failure(
    project: &Path,
    plan: &FlowPlan,
    started_at: &str,
    results: Vec<StepResult>,
    message: &str,
) -> Result<()> {
    config::write_last_run(
        project,
        &LastRun {
            finished: true,
            ok: false,
            started_at: started_at.into(),
            finished_at: config::now_rfc3339(),
            dry_run: false,
            offline: plan.offline,
            steps: results,
            message: message.into(),
            urls: vec![],
        },
    )
}
