use serde::Serialize;
use std::fs;
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::Mutex;
use std::thread;
use tauri::{AppHandle, Emitter, State};

/// Tracks the active shipctl process for cancel.
struct ActiveRun {
    pid: AtomicU32,
    cancel_requested: AtomicBool,
    /// Serializes start so two runs cannot overlap.
    gate: Mutex<()>,
}

impl ActiveRun {
    fn new() -> Self {
        Self {
            pid: AtomicU32::new(0),
            cancel_requested: AtomicBool::new(false),
            gate: Mutex::new(()),
        }
    }
}

#[derive(Debug, Serialize)]
struct CmdResult {
    ok: bool,
    code: i32,
    stdout: String,
    stderr: String,
    shipctl: String,
    cancelled: bool,
}

#[derive(Debug, Clone, Serialize)]
struct StreamLine {
    stream: String,
    text: String,
}

#[derive(Debug, Serialize)]
struct ShipState {
    project: String,
    has_ship_dir: bool,
    studio: Option<serde_json::Value>,
    last_run: Option<serde_json::Value>,
}

fn resolve_shipctl() -> Result<PathBuf, String> {
    if let Ok(p) = std::env::var("SHIPCTL_PATH") {
        let path = PathBuf::from(p);
        if path.is_file() {
            return Ok(path);
        }
    }

    // Portable: shipctl next to this desktop exe (after stage-desktop).
    if let Ok(exe) = std::env::current_exe() {
        if let Some(dir) = exe.parent() {
            for name in ["shipctl.exe", "shipctl"] {
                let cand = dir.join(name);
                if cand.is_file() {
                    return Ok(cand);
                }
            }
        }
    }

    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let candidates = [
        manifest_dir
            .join("../../..")
            .join("target")
            .join("release")
            .join("shipctl.exe"),
        manifest_dir
            .join("../../..")
            .join("target")
            .join("debug")
            .join("shipctl.exe"),
        manifest_dir
            .join("../../..")
            .join("target")
            .join("release")
            .join("shipctl"),
        manifest_dir
            .join("../../..")
            .join("target")
            .join("debug")
            .join("shipctl"),
    ];
    for c in candidates {
        if c.is_file() {
            return Ok(c);
        }
    }
    which::which("shipctl")
        .or_else(|_| which::which("shipctl.exe"))
        .map_err(|_| {
            "shipctl not found. Build shipctl --release, run scripts/stage-desktop, or set SHIPCTL_PATH"
                .into()
        })
}

#[tauri::command]
fn resolve_shipctl_path() -> Result<String, String> {
    Ok(resolve_shipctl()?.display().to_string())
}

fn ship_dir(project: &Path) -> PathBuf {
    project.join(".ship")
}

fn read_json_file(path: &Path) -> Result<serde_json::Value, String> {
    let raw = fs::read_to_string(path).map_err(|e| format!("read {}: {e}", path.display()))?;
    serde_json::from_str(&raw).map_err(|e| format!("parse {}: {e}", path.display()))
}

fn emit_line(app: &AppHandle, stream: &str, text: &str) {
    let _ = app.emit(
        "shipctl-line",
        StreamLine {
            stream: stream.into(),
            text: text.into(),
        },
    );
}

fn kill_process_tree(pid: u32) {
    if pid == 0 {
        return;
    }
    #[cfg(windows)]
    {
        let _ = Command::new("taskkill")
            .args(["/PID", &pid.to_string(), "/T", "/F"])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status();
    }
    #[cfg(unix)]
    {
        let _ = Command::new("kill")
            .args(["-TERM", &format!("-{pid}")])
            .status();
        let _ = Command::new("kill")
            .args(["-TERM", &pid.to_string()])
            .status();
    }
}

#[tauri::command]
fn pick_project() -> Result<Option<String>, String> {
    let folder = rfd::FileDialog::new()
        .set_title("Open project folder (one window = one repo)")
        .pick_folder();
    Ok(folder.map(|p| p.display().to_string()))
}

#[tauri::command]
fn pick_vault_save(default_name: Option<String>) -> Result<Option<String>, String> {
    let name = default_name.unwrap_or_else(|| "ship-secrets.km".into());
    let file = rfd::FileDialog::new()
        .set_title("Save encrypted vault (.km)")
        .add_filter("Clavis vault", &["km"])
        .set_file_name(&name)
        .save_file();
    Ok(file.map(|p| p.display().to_string()))
}

/// Open an interactive terminal for `shipctl launch open` (OAuth / sign / deploy run).
#[tauri::command]
fn open_launch_open_terminal(project: String) -> Result<(), String> {
    let shipctl = resolve_shipctl()?;
    let project_path = PathBuf::from(&project);
    if !project_path.is_dir() {
        return Err(format!("not a directory: {project}"));
    }

    #[cfg(windows)]
    {
        let wt = Command::new("wt")
            .args([
                "-d",
                &project,
                shipctl.to_str().unwrap_or("shipctl"),
                "launch",
                "--project",
                &project,
                "open",
            ])
            .spawn();
        if wt.is_ok() {
            return Ok(());
        }
        Command::new("cmd")
            .args([
                "/C",
                "start",
                "Ship Studio launch",
                shipctl.to_str().unwrap_or("shipctl"),
                "launch",
                "--project",
                &project,
                "open",
            ])
            .spawn()
            .map_err(|e| format!("spawn terminal: {e}"))?;
        return Ok(());
    }

    #[cfg(not(windows))]
    {
        let _ = (&shipctl, &project_path);
        Err("open_launch_open_terminal is implemented for Windows in this build".into())
    }
}

/// Open an interactive terminal for `shipctl human --no-open --put` (paste loop).
#[tauri::command]
fn open_human_put_terminal(project: String) -> Result<(), String> {
    let shipctl = resolve_shipctl()?;
    let project_path = PathBuf::from(&project);
    if !project_path.is_dir() {
        return Err(format!("not a directory: {project}"));
    }

    #[cfg(windows)]
    {
        // Prefer Windows Terminal; fall back to cmd.
        let wt = Command::new("wt")
            .args([
                "-d",
                &project,
                shipctl.to_str().unwrap_or("shipctl"),
                "human",
                "--project",
                &project,
                "--no-open",
                "--put",
            ])
            .spawn();
        if wt.is_ok() {
            return Ok(());
        }
        Command::new("cmd")
            .args([
                "/C",
                "start",
                "Ship Studio paste",
                shipctl.to_str().unwrap_or("shipctl"),
                "human",
                "--project",
                &project,
                "--no-open",
                "--put",
            ])
            .spawn()
            .map_err(|e| format!("spawn terminal: {e}"))?;
        return Ok(());
    }

    #[cfg(not(windows))]
    {
        let _ = (&shipctl, &project_path);
        Err("open_human_put_terminal is implemented for Windows in this build".into())
    }
}

/// Run shipctl with extra env (used for SHIP_VAULT_PASSPHRASE; values not logged).
#[tauri::command]
fn run_shipctl_env(
    app: AppHandle,
    active: State<'_, ActiveRun>,
    project: String,
    args: Vec<String>,
    env: std::collections::HashMap<String, String>,
) -> Result<CmdResult, String> {
    let (shipctl, mut child) = {
        let _gate = active
            .gate
            .lock()
            .map_err(|_| "lock poisoned".to_string())?;
        if active.pid.load(Ordering::SeqCst) != 0 {
            return Err("a command is already running — Cancel first".into());
        }
        active.cancel_requested.store(false, Ordering::SeqCst);

        let shipctl = resolve_shipctl()?;
        let project_path = Path::new(&project);
        if !project_path.is_dir() {
            return Err(format!("not a directory: {project}"));
        }

        // Never echo env keys that look like secrets/passphrases.
        emit_line(
            &app,
            "meta",
            &format!("$ {} {}", shipctl.display(), args.join(" ")),
        );

        let mut cmd = Command::new(&shipctl);
        cmd.current_dir(project_path)
            .args(&args)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        for (k, v) in &env {
            cmd.env(k, v);
        }

        let child = cmd
            .spawn()
            .map_err(|e| format!("spawn {}: {e}", shipctl.display()))?;

        active.pid.store(child.id(), Ordering::SeqCst);
        (shipctl, child)
    };

    let stdout = child
        .stdout
        .take()
        .ok_or_else(|| "missing stdout pipe".to_string())?;
    let stderr = child
        .stderr
        .take()
        .ok_or_else(|| "missing stderr pipe".to_string())?;

    let app_out = app.clone();
    let out_handle = thread::spawn(move || {
        let mut buf = String::new();
        let reader = BufReader::new(stdout);
        for line in reader.lines() {
            let line = line.unwrap_or_default();
            emit_line(&app_out, "stdout", &line);
            buf.push_str(&line);
            buf.push('\n');
        }
        buf
    });

    let app_err = app.clone();
    let err_handle = thread::spawn(move || {
        let mut buf = String::new();
        let reader = BufReader::new(stderr);
        for line in reader.lines() {
            let line = line.unwrap_or_default();
            emit_line(&app_err, "stderr", &line);
            buf.push_str(&line);
            buf.push('\n');
        }
        buf
    });

    let status = child
        .wait()
        .map_err(|e| format!("wait {}: {e}", shipctl.display()))?;

    let cancelled = active.cancel_requested.swap(false, Ordering::SeqCst);
    active.pid.store(0, Ordering::SeqCst);

    let stdout = out_handle.join().unwrap_or_default();
    let stderr = err_handle.join().unwrap_or_default();
    let code = status.code().unwrap_or(1);

    let meta = if cancelled {
        format!("exit {code} (cancelled)")
    } else {
        format!("exit {code}")
    };
    emit_line(&app, "meta", &meta);

    Ok(CmdResult {
        ok: status.success() && !cancelled,
        code,
        stdout,
        stderr,
        shipctl: shipctl.display().to_string(),
        cancelled,
    })
}

#[tauri::command]
fn cancel_shipctl(app: AppHandle, active: State<'_, ActiveRun>) -> Result<bool, String> {
    let pid = active.pid.load(Ordering::SeqCst);
    if pid == 0 {
        return Ok(false);
    }
    active.cancel_requested.store(true, Ordering::SeqCst);
    emit_line(&app, "meta", &format!("cancelling pid {pid}…"));
    kill_process_tree(pid);
    emit_line(&app, "meta", "cancel signal sent");
    Ok(true)
}

#[tauri::command]
fn run_shipctl(
    app: AppHandle,
    active: State<'_, ActiveRun>,
    project: String,
    args: Vec<String>,
) -> Result<CmdResult, String> {
    let (shipctl, mut child) = {
        let _gate = active
            .gate
            .lock()
            .map_err(|_| "lock poisoned".to_string())?;
        if active.pid.load(Ordering::SeqCst) != 0 {
            return Err("a command is already running — Cancel first".into());
        }
        active.cancel_requested.store(false, Ordering::SeqCst);

        let shipctl = resolve_shipctl()?;
        let project_path = Path::new(&project);
        if !project_path.is_dir() {
            return Err(format!("not a directory: {project}"));
        }

        emit_line(
            &app,
            "meta",
            &format!("$ {} {}", shipctl.display(), args.join(" ")),
        );

        let child = Command::new(&shipctl)
            .current_dir(project_path)
            .args(&args)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| format!("spawn {}: {e}", shipctl.display()))?;

        active.pid.store(child.id(), Ordering::SeqCst);
        (shipctl, child)
    };

    let stdout = child
        .stdout
        .take()
        .ok_or_else(|| "missing stdout pipe".to_string())?;
    let stderr = child
        .stderr
        .take()
        .ok_or_else(|| "missing stderr pipe".to_string())?;

    let app_out = app.clone();
    let out_handle = thread::spawn(move || {
        let mut buf = String::new();
        let reader = BufReader::new(stdout);
        for line in reader.lines() {
            let line = line.unwrap_or_default();
            emit_line(&app_out, "stdout", &line);
            buf.push_str(&line);
            buf.push('\n');
        }
        buf
    });

    let app_err = app.clone();
    let err_handle = thread::spawn(move || {
        let mut buf = String::new();
        let reader = BufReader::new(stderr);
        for line in reader.lines() {
            let line = line.unwrap_or_default();
            emit_line(&app_err, "stderr", &line);
            buf.push_str(&line);
            buf.push('\n');
        }
        buf
    });

    let status = child
        .wait()
        .map_err(|e| format!("wait {}: {e}", shipctl.display()))?;

    let cancelled = active.cancel_requested.swap(false, Ordering::SeqCst);
    active.pid.store(0, Ordering::SeqCst);

    let stdout = out_handle.join().unwrap_or_default();
    let stderr = err_handle.join().unwrap_or_default();
    let code = status.code().unwrap_or(1);

    let meta = if cancelled {
        format!("exit {code} (cancelled)")
    } else {
        format!("exit {code}")
    };
    emit_line(&app, "meta", &meta);

    Ok(CmdResult {
        ok: status.success() && !cancelled,
        code,
        stdout,
        stderr,
        shipctl: shipctl.display().to_string(),
        cancelled,
    })
}

#[tauri::command]
fn load_ship_state(project: String) -> Result<ShipState, String> {
    let project_path = Path::new(&project);
    if !project_path.is_dir() {
        return Err(format!("not a directory: {project}"));
    }
    let dir = ship_dir(project_path);
    let studio_path = dir.join("studio.json");
    let last_path = dir.join("last-run.json");
    Ok(ShipState {
        project,
        has_ship_dir: dir.is_dir(),
        studio: if studio_path.is_file() {
            Some(read_json_file(&studio_path)?)
        } else {
            None
        },
        last_run: if last_path.is_file() {
            Some(read_json_file(&last_path)?)
        } else {
            None
        },
    })
}

#[tauri::command]
fn save_ritual_args(
    project: String,
    sign_args: Vec<String>,
    deploy_args: Vec<String>,
) -> Result<serde_json::Value, String> {
    let project_path = Path::new(&project);
    if !project_path.is_dir() {
        return Err(format!("not a directory: {project}"));
    }
    let dir = ship_dir(project_path);
    fs::create_dir_all(&dir).map_err(|e| format!("mkdir .ship: {e}"))?;
    let path = dir.join("studio.json");

    let mut studio = if path.is_file() {
        let raw = fs::read_to_string(&path).map_err(|e| e.to_string())?;
        serde_json::from_str::<serde_json::Value>(&raw).map_err(|e| e.to_string())?
    } else {
        serde_json::json!({
            "schema": "ship-studio/v0",
            "project": project,
            "workflow": ["doctor", "configure", "sign (signet)", "deploy (orbit)"],
            "adapters": { "signet": "signet", "orbit": "orbit" },
            "offline_bridge": true,
            "notes": []
        })
    };

    let obj = studio
        .as_object_mut()
        .ok_or_else(|| "studio.json is not an object".to_string())?;
    obj.insert(
        "sign_args".into(),
        serde_json::to_value(&sign_args).map_err(|e| e.to_string())?,
    );
    obj.insert(
        "deploy_args".into(),
        serde_json::to_value(&deploy_args).map_err(|e| e.to_string())?,
    );
    obj.insert("project".into(), serde_json::Value::String(project.clone()));

    let pretty = serde_json::to_string_pretty(&studio).map_err(|e| e.to_string())?;
    fs::write(&path, pretty).map_err(|e| format!("write {}: {e}", path.display()))?;
    Ok(studio)
}

#[tauri::command]
fn write_temp_json(path: String, json: String) -> Result<String, String> {
    let p = PathBuf::from(&path);
    if let Some(parent) = p.parent() {
        fs::create_dir_all(parent).map_err(|e| format!("mkdir: {e}"))?;
    }
    fs::write(&p, json.as_bytes()).map_err(|e| format!("write {}: {e}", p.display()))?;
    Ok(p.display().to_string())
}

/// Write vault entry JSON to OS temp (not under the project). Returns absolute path.
#[tauri::command]
fn write_vault_entries_temp(json: String) -> Result<String, String> {
    let mut p = std::env::temp_dir();
    p.push(format!(
        "ship-vault-export-{}-{}.json",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_millis())
            .unwrap_or(0)
    ));
    fs::write(&p, json.as_bytes()).map_err(|e| format!("write {}: {e}", p.display()))?;
    Ok(p.display().to_string())
}

#[tauri::command]
fn delete_path(path: String) -> Result<(), String> {
    let p = PathBuf::from(&path);
    if p.is_file() {
        fs::remove_file(&p).map_err(|e| format!("delete {}: {e}", p.display()))?;
    }
    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(ActiveRun::new())
        .invoke_handler(tauri::generate_handler![
            pick_project,
            pick_vault_save,
            open_human_put_terminal,
            open_launch_open_terminal,
            run_shipctl,
            run_shipctl_env,
            cancel_shipctl,
            load_ship_state,
            save_ritual_args,
            resolve_shipctl_path,
            write_temp_json,
            write_vault_entries_temp,
            delete_path
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
