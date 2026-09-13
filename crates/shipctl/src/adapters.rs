use anyhow::{Context, Result};
use serde::Serialize;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use which::which;

#[derive(Debug, Serialize)]
pub struct DoctorReport {
    pub ok: bool,
    pub project: String,
    pub project_exists: bool,
    pub offline_bridge: bool,
    pub signet: ToolStatus,
    pub orbit: ToolStatus,
    pub studio: Option<serde_json::Value>,
    pub detected: crate::config::Detected,
    pub notes: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct ToolStatus {
    pub found: bool,
    pub path: Option<String>,
    pub version: Option<String>,
}

fn tool_status(names: &[&str], version_args: &[&str]) -> ToolStatus {
    for name in names {
        if let Ok(path) = which(name) {
            let version = Command::new(&path)
                .args(version_args)
                .output()
                .ok()
                .and_then(|out| {
                    let s = String::from_utf8_lossy(&out.stdout).trim().to_string();
                    if s.is_empty() {
                        let e = String::from_utf8_lossy(&out.stderr).trim().to_string();
                        if e.is_empty() {
                            None
                        } else {
                            Some(e.lines().next().unwrap_or("").to_string())
                        }
                    } else {
                        Some(s.lines().next().unwrap_or("").to_string())
                    }
                });
            return ToolStatus {
                found: true,
                path: Some(path.display().to_string()),
                version,
            };
        }
    }
    ToolStatus {
        found: false,
        path: None,
        version: None,
    }
}

fn env_bin(key: &str) -> Option<PathBuf> {
    std::env::var_os(key).map(PathBuf::from).filter(|p| p.is_file())
}

fn orbit_fallback() -> Option<PathBuf> {
    if let Some(p) = env_bin("ORBIT_PATH") {
        return Some(p);
    }
    if let Some(p) = env_bin("SHIP_ORBIT_FALLBACK") {
        return Some(p);
    }
    // Sibling checkout: <parent>/ship-studio + <parent>/ship/orbit.exe
    let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let sibling = manifest
        .join("../../..")
        .join("ship")
        .join("orbit.exe");
    if sibling.is_file() {
        return Some(sibling);
    }
    let sibling_unix = manifest.join("../../..").join("ship").join("orbit");
    if sibling_unix.is_file() {
        return Some(sibling_unix);
    }
    None
}

fn signet_fallback() -> Option<PathBuf> {
    env_bin("SIGNET_PATH")
}

pub fn doctor(project: &Path) -> Result<DoctorReport> {
    let project = std::fs::canonicalize(project).unwrap_or_else(|_| project.to_path_buf());
    let exists = project.is_dir();
    let mut signet = tool_status(&["signet", "signet.exe"], &["--version"]);
    if !signet.found {
        signet = tool_status(&["signet", "signet.exe"], &["version"]);
    }
    if !signet.found {
        if let Some(local) = signet_fallback() {
            let version = Command::new(&local)
                .arg("--version")
                .output()
                .ok()
                .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string());
            signet = ToolStatus {
                found: true,
                path: Some(local.display().to_string()),
                version,
            };
        }
    }

    let mut orbit = tool_status(&["orbit", "orbit.exe"], &["version"]);
    if !orbit.found {
        if let Some(local) = orbit_fallback() {
            let version = Command::new(&local)
                .arg("version")
                .output()
                .ok()
                .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string());
            orbit = ToolStatus {
                found: true,
                path: Some(local.display().to_string()),
                version,
            };
        }
    }

    let detected = crate::config::probe(&project);
    let studio = crate::config::read_studio(&project)
        .ok()
        .flatten()
        .and_then(|s| serde_json::to_value(s).ok());

    let mut notes = Vec::new();
    notes.push("Bridge is offline-first: it does not call vendor HTTPS itself.".into());
    if !signet.found {
        notes.push("signet not on PATH — install Signet or set SIGNET_PATH.".into());
    }
    if !orbit.found {
        notes.push("orbit not on PATH — build Orbit or set ORBIT_PATH.".into());
    }
    if !exists {
        notes.push("project path is not a directory.".into());
    }
    notes.extend(detected.hints.iter().cloned());

    let ok = exists && signet.found && orbit.found;
    Ok(DoctorReport {
        ok,
        project: project.display().to_string(),
        project_exists: exists,
        offline_bridge: true,
        signet,
        orbit,
        studio,
        detected,
        notes,
    })
}

fn resolve_bin(prefer: &[&str], fallback: Option<PathBuf>) -> Result<PathBuf> {
    for name in prefer {
        if let Ok(p) = which(name) {
            return Ok(p);
        }
    }
    if let Some(fb) = fallback {
        if fb.is_file() {
            return Ok(fb);
        }
    }
    anyhow::bail!("none of {:?} found on PATH (set SIGNET_PATH / ORBIT_PATH)", prefer)
}

pub fn run_signet(project: &Path, args: &[String]) -> Result<i32> {
    let bin = resolve_bin(&["signet", "signet.exe"], signet_fallback())?;
    run_in_project(&bin, project, args)
}

pub fn run_orbit(project: &Path, args: &[String]) -> Result<i32> {
    let bin = resolve_bin(&["orbit", "orbit.exe"], orbit_fallback())?;
    run_in_project(&bin, project, args)
}

fn run_in_project(bin: &Path, project: &Path, args: &[String]) -> Result<i32> {
    let status = Command::new(bin)
        .current_dir(project)
        .args(args)
        .stdin(Stdio::null())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()
        .with_context(|| format!("spawn {}", bin.display()))?;
    Ok(status.code().unwrap_or(1))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn doctor_runs_on_cwd() {
        let report = doctor(Path::new(".")).expect("doctor");
        assert!(report.offline_bridge);
        assert!(report.project_exists);
    }

    #[test]
    fn flow_plan_includes_real_sign_args() {
        let dir = tempfile_dir();
        let plan = crate::flow::plan(&dir, false, true, true).expect("plan");
        let sign = plan.steps.iter().find(|s| s.id == "sign").expect("sign");
        assert_ne!(sign.args, vec!["--help".to_string()]);
        assert!(!sign.args.is_empty());
    }

    #[test]
    fn probe_finds_nested_wrangler() {
        let dir = tempfile_dir();
        let api = dir.join("apps").join("api");
        std::fs::create_dir_all(&api).unwrap();
        std::fs::write(api.join("wrangler.toml"), "name = \"x\"\n").unwrap();
        let d = crate::config::probe(&dir);
        assert!(d.wrangler);
    }

    #[test]
    fn configure_suggests_cloudflare_deploy_args() {
        let dir = tempfile_dir();
        let api = dir.join("apps").join("api");
        std::fs::create_dir_all(&api).unwrap();
        std::fs::write(api.join("wrangler.toml"), "name = \"x\"\n").unwrap();
        let intent = crate::config::configure(&dir).expect("configure");
        assert_eq!(
            intent.deploy_args,
            vec![
                "deploy".to_string(),
                "--provider".to_string(),
                "cloudflare".to_string()
            ]
        );
    }

    fn tempfile_dir() -> PathBuf {
        let dir = std::env::temp_dir().join(format!(
            "shipctl-test-{}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }
}
