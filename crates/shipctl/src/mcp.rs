//! Minimal stdio MCP server for Ship tools.

use crate::adapters;
use crate::config;
use crate::flow;
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
