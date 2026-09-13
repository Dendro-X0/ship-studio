//! Minimal stdio MCP server for Ship tools.

use crate::adapters;
use crate::config;
use crate::flow;
use crate::guide;
use crate::human;
use crate::portal;
use crate::secrets;
use crate::ship;
use crate::vault_km;
use anyhow::{Context, Result};
use serde_json::{json, Value};
use std::io::{self, BufRead, Write};
use std::path::PathBuf;

pub fn serve() -> Result<()> {
    let stdin = io::stdin();
    let mut stdout = io::stdout();
    for line in stdin.lock().lines() {
        let line = line?;
        if line.trim().is_empty() {
            continue;
        }
        let req: Value = serde_json::from_str(&line).context("parse mcp json")?;
        let id = req.get("id").cloned().unwrap_or(Value::Null);
        let method = req.get("method").and_then(|m| m.as_str()).unwrap_or("");
        let res = match method {
            "initialize" => ok(
                id,
                json!({
                    "protocolVersion": "2024-11-05",
                    "capabilities": { "tools": {} },
                    "serverInfo": { "name": "shipctl", "version": env!("CARGO_PKG_VERSION") }
                }),
            ),
            "notifications/initialized" | "initialized" => continue,
            "tools/list" => ok(id, json!({ "tools": tools() })),
            "tools/call" => {
                let params = req.get("params").cloned().unwrap_or(json!({}));
                match call_tool(params) {
                    Ok(v) => ok(id, v),
                    Err(e) => err(id, e.to_string()),
                }
            }
            "ping" => ok(id, json!({})),
            _ => err(id, format!("method not found: {method}")),
        };
        writeln!(stdout, "{}", serde_json::to_string(&res)?)?;
        stdout.flush()?;
    }
    Ok(())
}

fn tools() -> Vec<Value> {
    vec![
        tool(
            "ship_doctor",
            "Check Signet/Orbit and project path (offline)",
            false,
        ),
        tool(
            "ship_configure",
            "Write .ship/studio.json env/workflow intent",
            false,
        ),
        tool_portal(),
        tool_secrets(),
        tool_vault(),
        tool(
            "ship_guide",
            "Unified offline shipping checklist (doctor+portal+secrets+flow); optional open entry URLs",
            false,
        ),
        tool(
            "ship_ship",
            "One-shot offline prep: guide → configure → flow dry-run; optional open entry URLs",
            false,
        ),
        tool(
            "ship_human",
            "Human portal sprint: open Polar/GitHub/dashboards; optional interactive secret put queue",
            false,
        ),
        tool(
            "ship_flow_dry_run",
            "Print configure→sign→deploy plan",
            true,
        ),
        tool(
            "ship_flow",
            "Execute configure→sign→deploy (set skip_deploy when offline)",
            true,
        ),
        tool(
            "ship_sign",
            "Run signet with studio.json sign_args (or override args)",
            false,
        ),
        tool(
            "ship_deploy",
            "Run orbit with studio.json deploy_args (or override args)",
            false,
        ),
        tool("ship_status", "Read .ship/last-run.json", false),
    ]
}

fn tool_secrets() -> Value {
    json!({
        "name": "ship_secrets",
        "description": "Paste-secret assist: hinted secret names + put CLI (never stores values)",
        "inputSchema": {
            "type": "object",
            "properties": {
                "project": { "type": "string" },
                "provider": { "type": "string" },
                "open": { "type": "boolean" }
            }
        }
    })
}

fn tool_vault() -> Value {
    json!({
        "name": "ship_vault",
        "description": "Encrypted kmvault (.km) export/list/show — Clavis-compatible. Passphrase via SHIP_VAULT_PASSPHRASE or argument (local only).",
        "inputSchema": {
            "type": "object",
            "properties": {
                "action": {
                    "type": "string",
                    "description": "export | list | show"
                },
                "out": { "type": "string", "description": "Output .km path (export)" },
                "file": { "type": "string", "description": "Existing .km path (list/show)" },
                "title": { "type": "string", "description": "Entry title (export one / show)" },
                "value": { "type": "string", "description": "Secret value (export one; prefer not logging)" },
                "url": { "type": "string" },
                "name": { "type": "string", "description": "Vault display name" },
                "passphrase": { "type": "string", "description": "Optional; else SHIP_VAULT_PASSPHRASE" },
                "entries": {
                    "type": "array",
                    "description": "export: [{title,value,url?}]",
                    "items": { "type": "object" }
                }
            },
            "required": ["action"]
        }
    })
}

fn tool_portal() -> Value {
    json!({
        "name": "ship_portal",
        "description": "Provider portal plan: Cloudflare/Vercel/Netlify/GitHub entry URLs and OAuth CLI steps",
        "inputSchema": {
            "type": "object",
            "properties": {
                "project": { "type": "string", "description": "Absolute project path" },
                "provider": {
                    "type": "string",
                    "description": "Optional filter: cloudflare|vercel|netlify|github"
                },
                "open": {
                    "type": "boolean",
                    "description": "Open token pages in the system browser"
                }
            }
        }
    })
}

fn tool(name: &str, description: &str, flow_flags: bool) -> Value {
    let mut props = json!({
        "project": { "type": "string", "description": "Absolute project path" }
    });
    if flow_flags {
        props["skip_sign"] = json!({ "type": "boolean" });
        props["skip_deploy"] = json!({ "type": "boolean" });
        props["offline"] = json!({ "type": "boolean" });
    }
    if name == "ship_sign" || name == "ship_deploy" {
        props["args"] = json!({
            "type": "array",
            "items": { "type": "string" },
            "description": "Optional override args"
        });
    }
    if name == "ship_guide" || name == "ship_ship" {
        props["open"] = json!({
            "type": "boolean",
            "description": "Open all unique entry URLs in the browser"
        });
    }
    if name == "ship_human" {
        props["open"] = json!({ "type": "boolean", "description": "Open paste-source URLs (default true)" });
        props["open_all"] = json!({ "type": "boolean", "description": "Also open full guide entry URLs" });
        props["put"] = json!({ "type": "boolean", "description": "Interactively put queued secrets" });
    }
    json!({
        "name": name,
        "description": description,
        "inputSchema": {
            "type": "object",
            "properties": props
        }
    })
}

fn call_tool(params: Value) -> Result<Value> {
    let name = params
        .get("name")
        .and_then(|n| n.as_str())
        .unwrap_or("");
    let args = params.get("arguments").cloned().unwrap_or(json!({}));
    let project = args
        .get("project")
        .and_then(|p| p.as_str())
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."));
    let skip_sign = args
        .get("skip_sign")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);
    let skip_deploy = args
        .get("skip_deploy")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);
    let offline = args
        .get("offline")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);

    let body = match name {
        "ship_doctor" => serde_json::to_value(adapters::doctor(&project)?)?,
        "ship_configure" => serde_json::to_value(config::configure(&project)?)?,
        "ship_portal" => {
            let filter = args
                .get("provider")
                .and_then(|p| p.as_str())
                .map(portal::ProviderId::parse)
                .transpose()?;
            let plan = portal::plan_for(&project, filter)?;
            let open = args
                .get("open")
                .and_then(|v| v.as_bool())
                .unwrap_or(false);
            let mut out = serde_json::to_value(&plan)?;
            if open {
                let opened = portal::open_urls(&plan)?;
                out["opened"] = json!(opened);
            }
            out
        }
        "ship_secrets" => {
            let filter = args
                .get("provider")
                .and_then(|p| p.as_str())
                .map(portal::ProviderId::parse)
                .transpose()?;
            let plan = secrets::plan_for(&project, filter)?;
            let open = args
                .get("open")
                .and_then(|v| v.as_bool())
                .unwrap_or(false);
            let mut out = serde_json::to_value(&plan)?;
            if open {
                let opened = secrets::open_entry_urls(&plan)?;
                out["opened"] = json!(opened);
            }
            out
        }
        "ship_vault" => {
            let action = args
                .get("action")
                .and_then(|a| a.as_str())
                .unwrap_or("list");
            if let Some(pass) = args.get("passphrase").and_then(|p| p.as_str()) {
                if !pass.is_empty() {
                    std::env::set_var("SHIP_VAULT_PASSPHRASE", pass);
                }
            }
            match action {
                "export" => {
                    let out_path = args
                        .get("out")
                        .and_then(|p| p.as_str())
                        .map(PathBuf::from)
                        .context("ship_vault export requires out")?;
                    let vault_name = args
                        .get("name")
                        .and_then(|n| n.as_str())
                        .unwrap_or("Ship Studio secrets");
                    let path = if let Some(arr) = args.get("entries").and_then(|e| e.as_array()) {
                        let mut entries = Vec::new();
                        for item in arr {
                            let title = item
                                .get("title")
                                .or_else(|| item.get("name"))
                                .and_then(|x| x.as_str())
                                .unwrap_or("")
                                .to_string();
                            let value = item
                                .get("value")
                                .or_else(|| item.get("password"))
                                .and_then(|x| x.as_str())
                                .unwrap_or("")
                                .to_string();
                            if title.is_empty() || value.is_empty() {
                                continue;
                            }
                            let url = item
                                .get("url")
                                .and_then(|x| x.as_str())
                                .unwrap_or("")
                                .to_string();
                            let notes = item
                                .get("notes")
                                .and_then(|x| x.as_str())
                                .unwrap_or("Exported via MCP ship_vault")
                                .to_string();
                            entries.push((title, value, url, notes));
                        }
                        vault_km::export_entries(&out_path, vault_name, &entries)?
                    } else {
                        let title = args
                            .get("title")
                            .and_then(|t| t.as_str())
                            .context("ship_vault export needs entries[] or title+value")?;
                        let value = args
                            .get("value")
                            .and_then(|v| v.as_str())
                            .context("ship_vault export needs value")?;
                        let url = args.get("url").and_then(|u| u.as_str()).unwrap_or("");
                        vault_km::export_one(&out_path, vault_name, title, value, url)?
                    };
                    json!({
                        "ok": true,
                        "path": path,
                        "format": "kmvault/v1",
                        "compatible_with": "Clavis / Keys Manager"
                    })
                }
                "list" => {
                    let file = args
                        .get("file")
                        .and_then(|p| p.as_str())
                        .map(PathBuf::from)
                        .context("ship_vault list requires file")?;
                    let titles = vault_km::list_titles(&file)?;
                    json!({ "file": file, "titles": titles })
                }
                "show" => {
                    let file = args
                        .get("file")
                        .and_then(|p| p.as_str())
                        .map(PathBuf::from)
                        .context("ship_vault show requires file")?;
                    let title = args
                        .get("title")
                        .and_then(|t| t.as_str())
                        .context("ship_vault show requires title")?;
                    let value = vault_km::show_value(&file, title)?;
                    json!({ "title": title, "value": value })
                }
                other => anyhow::bail!("ship_vault unknown action: {other}"),
            }
        }
        "ship_guide" => {
            let plan = guide::plan_for(&project)?;
            let open = args
                .get("open")
                .and_then(|v| v.as_bool())
                .unwrap_or(false);
            let mut out = serde_json::to_value(&plan)?;
            if open {
                let opened = guide::open_entries(&plan)?;
                out["opened"] = json!(opened);
            }
            out
        }
        "ship_ship" => {
            let open = args
                .get("open")
                .and_then(|v| v.as_bool())
                .unwrap_or(false);
            serde_json::to_value(ship::run(&project, open)?)?
        }
        "ship_human" => {
            let open = args
                .get("open")
                .and_then(|v| v.as_bool())
                .unwrap_or(true);
            let put = args
                .get("put")
                .and_then(|v| v.as_bool())
                .unwrap_or(false);
            let open_all = args
                .get("open_all")
                .and_then(|v| v.as_bool())
                .unwrap_or(false);
            serde_json::to_value(human::run_with_options(&project, open, put, open_all)?)?
        }
        "ship_flow_dry_run" => {
            serde_json::to_value(flow::plan(&project, skip_sign, skip_deploy, offline)?)?
        }
        "ship_flow" => {
            let plan = flow::plan(&project, skip_sign, skip_deploy, offline)?;
            flow::execute(&project, &plan)?;
            config::read_last_run(&project)?
        }
        "ship_sign" => {
            let intent = config::intent_for(&project)?;
            let sign_args = override_args(&args, &intent.sign_args);
            let code = adapters::run_signet(&project, &sign_args)?;
            json!({ "ok": code == 0, "exit_code": code, "args": sign_args })
        }
        "ship_deploy" => {
            if offline {
                anyhow::bail!("ship_deploy refuses offline");
            }
            let intent = config::intent_for(&project)?;
            let deploy_args = override_args(&args, &intent.deploy_args);
            let code = adapters::run_orbit(&project, &deploy_args)?;
            json!({ "ok": code == 0, "exit_code": code, "args": deploy_args })
        }
        "ship_status" => config::read_last_run(&project)?,
        other => anyhow::bail!("unknown tool: {other}"),
    };

    Ok(json!({
        "content": [{ "type": "text", "text": serde_json::to_string_pretty(&body)? }]
    }))
}

fn override_args(args: &Value, defaults: &[String]) -> Vec<String> {
    args.get("args")
        .and_then(|a| a.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str().map(|s| s.to_string()))
                .collect()
        })
        .filter(|v: &Vec<String>| !v.is_empty())
        .unwrap_or_else(|| defaults.to_vec())
}

fn ok(id: Value, result: Value) -> Value {
    json!({ "jsonrpc": "2.0", "id": id, "result": result })
}

fn err(id: Value, message: String) -> Value {
    json!({
        "jsonrpc": "2.0",
        "id": id,
        "error": { "code": -32000, "message": message }
    })
}
