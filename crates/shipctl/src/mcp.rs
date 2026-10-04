//! Minimal stdio MCP server for Ship tools.

use crate::adapters;
use crate::assist;
use crate::config;
use crate::envx;
use crate::flow;
use crate::guide;
use crate::human;
use crate::launch;
use crate::portal;
use crate::publish;
use crate::pulse;
use crate::scopes;
use crate::secrets;
use crate::ship;
use crate::signpath;
use crate::vault_km;
use anyhow::{Context, Result};
use serde_json::{json, Value};
use std::io::{self, BufRead, IsTerminal, Write};
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
            "Human portal sprint: open Polar/GitHub/dashboards. put:true requires an interactive TTY — prefer ship_env_put (spawn) or Desktop Put when running under MCP.",
            false,
        ),
        tool(
            "ship_launch",
            "Guided launch status (open→verify→next). Use CLI for open/verify/confirm/next mutations.",
            false,
        ),
        tool(
            "ship_publish",
            "Publish portal status (minute wizard, read-only). Mutations: ship_publish_open / ship_publish_verify / ship_publish_confirm / ship_publish_next.",
            false,
        ),
        tool_publish_open(),
        tool_publish_verify(),
        tool_publish_confirm(),
        tool_publish_next(),
        tool_publish_watch(),
        tool_env_put(),
        tool(
            "ship_scopes",
            "Detect Web/API/Desktop deploy scopes (directories + providers)",
            false,
        ),
        tool(
            "ship_env",
            "ENV/token portal: configure, retrieve, create URLs (no secret values)",
            false,
        ),
        tool(
            "ship_sign_paths",
            "Self-sign (Signet) vs official vendor signing wizards",
            false,
        ),
        tool(
            "ship_assist",
            "Full-stack deploy assist checklist",
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
        tool_hostdeploy(),
        tool("ship_status", "Read .ship/last-run.json", false),
        tool(
            "ship_pulse",
            "Project pulse: git, publish/launch progress, deploy signals, next action (local only)",
            false,
        ),
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
        "description": "Encrypted kmvault (.km) list/export — Clavis-compatible. Prefer Desktop vault export. Do NOT pass secret values or passphrases in tool args (they land in agent logs); use SHIP_VAULT_PASSPHRASE in the environment and export from a human TTY when possible. list/show titles only when needed.",
        "inputSchema": {
            "type": "object",
            "properties": {
                "action": {
                    "type": "string",
                    "description": "export | list | show — prefer list; avoid export with value in MCP"
                },
                "out": { "type": "string", "description": "Output .km path (export)" },
                "file": { "type": "string", "description": "Existing .km path (list/show)" },
                "title": { "type": "string", "description": "Entry title (export one / show)" },
                "value": {
                    "type": "string",
                    "description": "DISCOURAGED in MCP — secret lands in agent context. Prefer Desktop Put + vault export."
                },
                "url": { "type": "string" },
                "name": { "type": "string", "description": "Vault display name" },
                "passphrase": {
                    "type": "string",
                    "description": "DISCOURAGED — prefer SHIP_VAULT_PASSPHRASE env; never echo in chat"
                },
                "entries": {
                    "type": "array",
                    "description": "DISCOURAGED in MCP export: [{title,value,url?}]",
                    "items": { "type": "object" }
                }
            },
            "required": ["action"]
        }
    })
}

fn tool_publish_open() -> Value {
    json!({
        "name": "ship_publish_open",
        "description": "Open/Run the current publish step (browser entry_url and/or local CLI). Does not Confirm. Never stores secrets. OAuth login CLIs may prompt — prefer a human-visible TTY.",
        "inputSchema": {
            "type": "object",
            "properties": publish_common_props(json!({})),
            "required": ["project"]
        }
    })
}

fn tool_publish_verify() -> Value {
    json!({
        "name": "ship_publish_verify",
        "description": "Local Verify for the current publish step (disk / local CLI / operator CLI). Never vendor HTTPS. Does not mark Human/OAuth/deploy Done — use ship_publish_confirm after the human attests.",
        "inputSchema": {
            "type": "object",
            "properties": publish_common_props(json!({})),
            "required": ["project"]
        }
    })
}

fn tool_publish_confirm() -> Value {
    json!({
        "name": "ship_publish_confirm",
        "description": "Mark the current publish step done (operator attestation). Agents must only call this after the human finished the official page or CLI. Never live npm/cargo/docker/store upload.",
        "inputSchema": {
            "type": "object",
            "properties": publish_common_props(json!({})),
            "required": ["project"]
        }
    })
}

fn tool_publish_next() -> Value {
    json!({
        "name": "ship_publish_next",
        "description": "Advance to the next pending publish step (requires current done, or force=true). Does not skip vendor UI.",
        "inputSchema": {
            "type": "object",
            "properties": publish_common_props(json!({
                "force": {
                    "type": "boolean",
                    "description": "Advance even if current step is not done (default false)"
                }
            })),
            "required": ["project"]
        }
    })
}

fn publish_common_props(mut extra: Value) -> Value {
    extra["project"] = json!({ "type": "string", "description": "Absolute project path" });
    extra["mode"] = json!({
        "type": "string",
        "description": "general (default) | advanced"
    });
    extra["intent"] = json!({
        "type": "string",
        "description": "Optional ship intent override (local | public)"
    });
    extra
}

fn tool_env_put() -> Value {
    json!({
        "name": "ship_env_put",
        "description": "Launch (or print) interactive host Put for a secret NAME on cloudflare|vercel|netlify. Never accepts a secret value. Prefer spawn:true so a visible terminal prompts the human.",
        "inputSchema": {
            "type": "object",
            "properties": {
                "project": { "type": "string", "description": "Absolute project path" },
                "provider": {
                    "type": "string",
                    "description": "cloudflare | vercel | netlify"
                },
                "name": {
                    "type": "string",
                    "description": "Secret env NAME only — never the value"
                },
                "spawn": {
                    "type": "boolean",
                    "description": "Open an external terminal with the Put CLI (default true)"
                }
            },
            "required": ["project", "provider", "name"]
        }
    })
}

fn tool_publish_watch() -> Value {
    json!({
        "name": "ship_publish_watch",
        "description": "One local Verify probe for the current publish step (never vendor HTTPS). Agents should poll. Optional auto_confirm when verify ok (never live-publishes).",
        "inputSchema": {
            "type": "object",
            "properties": {
                "project": { "type": "string", "description": "Absolute project path" },
                "auto_confirm": {
                    "type": "boolean",
                    "description": "When verify ok, Confirm then Next (default false)"
                }
            },
            "required": ["project"]
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

fn tool_hostdeploy() -> Value {
    json!({
        "name": "ship_hostdeploy",
        "description": "Stream local vendor CLI deploy (wrangler/vercel/netlify) via shipctl hostdeploy. Writes hosted last-run URLs. Prefer Desktop Deployment Deploy when auth prompts may appear. Never stores secrets; does not fill Create Token. On auth fail tell human: Login CLI or Sign in (web).",
        "inputSchema": {
            "type": "object",
            "properties": {
                "project": { "type": "string", "description": "Absolute project path" },
                "provider": {
                    "type": "string",
                    "description": "cloudflare | vercel | netlify (default cloudflare)"
                },
                "name": {
                    "type": "string",
                    "description": "Optional Pages/site project name override"
                }
            },
            "required": ["project"]
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
        props["put"] = json!({
            "type": "boolean",
            "description": "Interactive put queue — REQUIRES a real TTY. Under MCP use ship_env_put instead (default false)."
        });
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
            if put && !std::io::stdin().is_terminal() {
                anyhow::bail!(
                    "ship_human put:true needs an interactive TTY (MCP has none). Use ship_env_put {{ provider, name, spawn:true }} or Desktop Put — never paste secrets into tool args."
                );
            }
            serde_json::to_value(human::run_with_options(&project, open, put, open_all)?)?
        }
        "ship_launch" => {
            let state = launch::load_or_build(&project)?;
            serde_json::to_value(launch::view(&state))?
        }
        "ship_publish" => {
            let (mode, intent) = publish_args(&args);
            let state = publish::load_or_build_with_options(&project, mode, intent)?;
            serde_json::to_value(publish::view(&state))?
        }
        "ship_publish_open" => {
            let (mode, intent) = publish_args(&args);
            let _ = publish::load_or_build_with_options(&project, mode, intent)?;
            serde_json::to_value(publish::open_current(&project)?)?
        }
        "ship_publish_verify" => {
            let (mode, intent) = publish_args(&args);
            let _ = publish::load_or_build_with_options(&project, mode, intent)?;
            let (ok, msg, view) = publish::verify_current(&project)?;
            json!({
                "ok": ok,
                "message": msg,
                "publish": view
            })
        }
        "ship_publish_confirm" => {
            let (mode, intent) = publish_args(&args);
            let _ = publish::load_or_build_with_options(&project, mode, intent)?;
            serde_json::to_value(publish::confirm_current(&project)?)?
        }
        "ship_publish_next" => {
            let (mode, intent) = publish_args(&args);
            let force = args
                .get("force")
                .and_then(|v| v.as_bool())
                .unwrap_or(false);
            let _ = publish::load_or_build_with_options(&project, mode, intent)?;
            match publish::next(&project, force) {
                Ok(view) => serde_json::to_value(view)?,
                Err(err) => {
                    let state =
                        publish::load_or_build_with_options(&project, mode, intent)?;
                    json!({
                        "ok": false,
                        "message": format!("{err:#}"),
                        "publish": publish::view(&state)
                    })
                }
            }
        }
        "ship_publish_watch" => {
            let auto_confirm = args
                .get("auto_confirm")
                .and_then(|v| v.as_bool())
                .unwrap_or(false);
            let _ = publish::load_or_build(&project)?;
            publish::watch_probe(&project, auto_confirm)?
        }
        "ship_scopes" => serde_json::to_value(scopes::plan_for(&project))?,
        "ship_env" => serde_json::to_value(envx::plan_for(&project)?)?,
        "ship_env_put" => {
            if args.get("value").is_some() {
                anyhow::bail!(
                    "ship_env_put never accepts a secret value — pass name only; human pastes in the terminal"
                );
            }
            let provider = args
                .get("provider")
                .and_then(|p| p.as_str())
                .context("ship_env_put requires provider")?;
            let name = args
                .get("name")
                .and_then(|n| n.as_str())
                .context("ship_env_put requires name")?;
            let spawn = args
                .get("spawn")
                .and_then(|v| v.as_bool())
                .unwrap_or(true);
            serde_json::to_value(envx::put_launch(&project, provider, name, spawn)?)?
        }
        "ship_sign_paths" => serde_json::to_value(signpath::plan_for(&project))?,
        "ship_assist" => serde_json::to_value(assist::plan_for(&project)?)?,
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
        "ship_hostdeploy" => {
            if offline {
                anyhow::bail!("ship_hostdeploy refuses offline");
            }
            let provider = args
                .get("provider")
                .and_then(|p| p.as_str())
                .unwrap_or("cloudflare");
            let name = args.get("name").and_then(|n| n.as_str());
            // Prefer Desktop Deploy when TTY auth may appear; still runnable for agents.
            match crate::hostdeploy::run(&project, provider, name) {
                Ok(report) => serde_json::to_value(report)?,
                Err(err) => {
                    let last = config::read_last_run(&project).unwrap_or(json!({}));
                    json!({
                        "ok": false,
                        "provider": provider,
                        "error": err.to_string(),
                        "last_run": last,
                        "hint": "Classify failure from last_run.message ([auth]/missing_cli]/…). Auth → human Login CLI / Sign in (web). Prefer Desktop Deployment Deploy for interactive prompts."
                    })
                }
            }
        }
        "ship_status" => config::read_last_run(&project)?,
        "ship_pulse" => serde_json::to_value(pulse::for_project(&project)?)?,
        other => anyhow::bail!("unknown tool: {other}"),
    };

    Ok(json!({
        "content": [{ "type": "text", "text": serde_json::to_string_pretty(&body)? }]
    }))
}

fn publish_args(args: &Value) -> (publish::StudioMode, Option<config::ShipIntent>) {
    let mode = args
        .get("mode")
        .and_then(|m| m.as_str())
        .map(publish::StudioMode::parse)
        .unwrap_or_default();
    let intent = args
        .get("intent")
        .and_then(|i| i.as_str())
        .map(config::ShipIntent::parse);
    (mode, intent)
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tools_include_ship_hostdeploy() {
        let tools = tools();
        let names: Vec<_> = tools
            .iter()
            .filter_map(|t| t.get("name").and_then(|n| n.as_str()))
            .collect();
        assert!(names.contains(&"ship_hostdeploy"), "{names:?}");
        assert!(names.contains(&"ship_pulse"));
    }

    #[test]
    fn tools_include_publish_g1_mutations() {
        let tools = tools();
        let names: Vec<_> = tools
            .iter()
            .filter_map(|t| t.get("name").and_then(|n| n.as_str()))
            .collect();
        for n in [
            "ship_publish_open",
            "ship_publish_verify",
            "ship_publish_confirm",
            "ship_publish_next",
            "ship_env_put",
        ] {
            assert!(names.contains(&n), "{n} missing in {names:?}");
        }
    }

    #[test]
    fn publish_verify_mcp_returns_ok_payload() {
        let dir = std::env::temp_dir().join(format!(
            "shipctl-mcp-g1-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_nanos())
                .unwrap_or(0)
        ));
        let _ = std::fs::create_dir_all(&dir);
        std::fs::write(dir.join("package.json"), r#"{"name":"mcp-g1"}"#).unwrap();
        let params = json!({
            "name": "ship_publish_verify",
            "arguments": { "project": dir.to_string_lossy() }
        });
        let wrapped = call_tool(params).expect("verify tool");
        let text = wrapped["content"][0]["text"].as_str().expect("text");
        let body: Value = serde_json::from_str(text).expect("json");
        assert!(body.get("ok").and_then(|v| v.as_bool()).is_some(), "{body}");
        assert!(body.get("publish").is_some(), "{body}");
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn env_put_rejects_value_arg() {
        let params = json!({
            "name": "ship_env_put",
            "arguments": {
                "project": ".",
                "provider": "cloudflare",
                "name": "FOO",
                "value": "secret"
            }
        });
        let err = call_tool(params).expect_err("must reject value");
        assert!(err.to_string().contains("never accepts"), "{err}");
    }

    #[test]
    fn env_put_recipe_without_spawn() {
        let dir = std::env::temp_dir().join(format!(
            "shipctl-mcp-g2-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_nanos())
                .unwrap_or(0)
        ));
        let _ = std::fs::create_dir_all(&dir);
        std::fs::write(dir.join("wrangler.toml"), "name = \"g2\"\n").unwrap();
        let params = json!({
            "name": "ship_env_put",
            "arguments": {
                "project": dir.to_string_lossy(),
                "provider": "cloudflare",
                "name": "RESEND_API_KEY",
                "spawn": false
            }
        });
        let wrapped = call_tool(params).expect("env put");
        let text = wrapped["content"][0]["text"].as_str().expect("text");
        let body: Value = serde_json::from_str(text).expect("json");
        assert_eq!(body["ok"], true);
        assert_eq!(body["spawned"], false);
        assert_eq!(body["name"], "RESEND_API_KEY");
        let recipe = body["recipe"].as_str().unwrap_or("");
        assert!(recipe.contains("--put"), "{recipe}");
        assert!(recipe.contains("RESEND_API_KEY"), "{recipe}");
        let host = body["host_cli"].as_array().expect("host_cli");
        assert_eq!(host[0], "wrangler");
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn human_put_bails_without_tty() {
        // MCP stdio is not an interactive put TTY in unit tests.
        if std::io::stdin().is_terminal() {
            // Rare in CI; skip assertion rather than hang.
            return;
        }
        let params = json!({
            "name": "ship_human",
            "arguments": { "project": ".", "put": true, "open": false }
        });
        let err = call_tool(params).expect_err("put without TTY must fail");
        let msg = err.to_string();
        assert!(msg.contains("TTY") || msg.contains("ship_env_put"), "{msg}");
    }
}
