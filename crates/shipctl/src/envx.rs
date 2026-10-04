//! ENV / token portal — names, files, create URLs, put CLIs. Never values.

use crate::human;
use crate::portal::{self, ProviderId};
use crate::secrets;
use anyhow::Result;
use serde::Serialize;
use std::path::Path;

#[derive(Debug, Serialize)]
pub struct EnvAction {
    pub id: String,
    pub kind: String,
    pub title: String,
    pub detail: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub entry_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub put_cli: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub provider: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct EnvPortal {
    pub schema: String,
    pub project: String,
    pub actions: Vec<EnvAction>,
    pub notes: Vec<String>,
}

pub fn plan_for(project: &Path) -> Result<EnvPortal> {
    let secrets_plan = secrets::plan_for(project, None)?;
    let portal = portal::plan_for(project, None)?;
    let put_queue = human::put_queue_public(&secrets_plan.hints);
    let mut actions = Vec::new();

    actions.push(EnvAction {
        id: "configure.local".into(),
        kind: "configure".into(),
        title: "Configure — local env files".into(),
        detail: "Edit .dev.vars / .env locally. Names only in Ship Studio.".into(),
        entry_url: None,
        put_cli: None,
        provider: None,
        name: None,
    });

    for step in &portal.steps {
        if step.kind == "oauth" || step.kind == "token_page" || step.kind == "dashboard" {
            actions.push(EnvAction {
                id: format!("create.{}", step.id),
                kind: "create".into(),
                title: format!("Create — {}", step.title),
                detail: step.detail.clone(),
                entry_url: step.entry_url.clone(),
                put_cli: step.cli.clone(),
                provider: Some(step.provider.clone()),
                name: None,
            });
        }
    }

    for h in &put_queue {
        actions.push(EnvAction {
            id: format!("retrieve.{}", h.name),
            kind: "retrieve".into(),
            title: format!("Retrieve / put — {}", h.name),
            detail: format!(
                "Copy on the official page, then put via `{}`.",
                h.put_cli.join(" ")
            ),
            entry_url: h.entry_url.clone(),
            put_cli: Some(h.put_cli.clone()),
            provider: Some(h.provider.clone()),
            name: Some(h.name.clone()),
        });
    }

    Ok(EnvPortal {
        schema: "ship-studio/env/v1".into(),
        project: project.display().to_string(),
        notes: vec![
            "Create tokens on official dashboards. Retrieve = copy + Put on Cloudflare/Vercel/Netlify.".into(),
            "No Put rows yet? Add empty NAME= lines to .env / .dev.vars or wrangler # Secrets: comments.".into(),
            "Ship Studio never stores secret values in .ship/.".into(),
        ],
        actions,
    })
}

pub fn put(project: &Path, provider: &str, name: &str) -> Result<i32> {
    let id = ProviderId::parse(provider)?;
    secrets::put_secret(project, id, name)
}

/// Agent-safe Put launch: recipe + optional external TTY. Never accepts a secret value.
#[derive(Debug, Serialize)]
pub struct PutLaunch {
    pub ok: bool,
    pub project: String,
    pub provider: String,
    pub name: String,
    /// Exact shipctl argv the human (or spawned terminal) should run.
    pub argv: Vec<String>,
    /// Shell-friendly one-liner (quoted for copy-paste).
    pub recipe: String,
    /// Underlying host CLI after shipctl starts (e.g. wrangler secret put NAME).
    pub host_cli: Vec<String>,
    pub spawned: bool,
    pub hint: String,
}

pub fn put_launch(
    project: &Path,
    provider: &str,
    name: &str,
    spawn: bool,
) -> Result<PutLaunch> {
    let id = ProviderId::parse(provider)?;
    let name = name.trim();
    if name.is_empty() || name == "<NAME>" {
        anyhow::bail!("pass a real secret name — never a value");
    }
    match id {
        ProviderId::Cloudflare | ProviderId::Vercel | ProviderId::Netlify => {}
        _ => anyhow::bail!(
            "{} has no interactive Put CLI — open the vendor dashboard, then put on cloudflare|vercel|netlify",
            id.label()
        ),
    }

    let project = std::fs::canonicalize(project).unwrap_or_else(|_| project.to_path_buf());
    let shipctl = std::env::current_exe()
        .ok()
        .and_then(|p| p.to_str().map(|s| s.to_string()))
        .unwrap_or_else(|| "shipctl".into());

    let argv = vec![
        shipctl.clone(),
        "env".into(),
        "--project".into(),
        project.display().to_string(),
        "--provider".into(),
        id.as_str().into(),
        "--put".into(),
        name.into(),
    ];
    let recipe = shell_join(&argv);
    let host_cli = secrets::put_cli_public(id, name);

    let mut spawned = false;
    if spawn {
        spawned = spawn_put_terminal(&recipe, &project)?;
    }

    Ok(PutLaunch {
        ok: true,
        project: project.display().to_string(),
        provider: id.as_str().into(),
        name: name.into(),
        argv,
        recipe,
        host_cli,
        spawned,
        hint: if spawned {
            "Terminal opened — paste the secret when the host CLI prompts. Never paste into chat or Studio."
                .into()
        } else {
            "Run the recipe in a visible terminal (or call again with spawn:true). Paste only when prompted — never into chat or Studio."
                .into()
        },
    })
}

fn shell_join(argv: &[String]) -> String {
    argv.iter()
        .map(|a| {
            let needs_quote = a.is_empty()
                || a.chars()
                    .any(|c| c.is_whitespace() || "\"&|<>^".contains(c));
            if needs_quote {
                format!("\"{}\"", a.replace('"', "\\\""))
            } else {
                a.clone()
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

fn spawn_put_terminal(recipe: &str, project: &Path) -> Result<bool> {
    use std::process::{Command, Stdio};

    #[cfg(target_os = "windows")]
    {
        let title = "Ship Studio env put";
        // /K keeps the window open after put so the human sees errors.
        let status = Command::new("cmd")
            .args(["/C", "start", title, "cmd", "/K", recipe])
            .current_dir(project)
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn();
        return Ok(status.is_ok());
    }

    #[cfg(target_os = "macos")]
    {
        let escaped = recipe.replace('\\', "\\\\").replace('"', "\\\"");
        let script = format!(
            "tell application \"Terminal\" to do script \"cd {cwd} && {cmd}\"",
            cwd = project.display().to_string().replace('"', "\\\""),
            cmd = escaped
        );
        let status = Command::new("osascript")
            .args(["-e", &script])
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn();
        return Ok(status.is_ok());
    }

    #[cfg(not(any(target_os = "windows", target_os = "macos")))]
    {
        let _ = (recipe, project);
        // No reliable default terminal; recipe-only.
        Ok(false)
    }
}
