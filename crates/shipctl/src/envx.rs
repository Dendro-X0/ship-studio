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
            "Create tokens on official dashboards. Retrieve = copy + put CLI.".into(),
            "Ship Studio never stores secret values in .ship/.".into(),
        ],
        actions,
    })
}

pub fn put(project: &Path, provider: &str, name: &str) -> Result<i32> {
    let id = ProviderId::parse(provider)?;
    secrets::put_secret(project, id, name)
}
