//! itch.io butler push — recipe + optional visible TTY. Never holds credentials.

use anyhow::{bail, Result};
use serde::Serialize;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use which::which;

const BUTLER_DOCS: &str = "https://itch.io/docs/butler/";

#[derive(Debug, Serialize)]
pub struct ButlerLaunch {
    pub ok: bool,
    pub project: String,
    pub target: String,
    pub dir: String,
    pub argv: Vec<String>,
    pub recipe: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub butler_path: Option<String>,
    pub spawned: bool,
    pub hint: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub docs_url: Option<String>,
}

/// Agent-safe butler push: print recipe and/or spawn a terminal. Never accepts secrets.
pub fn push_launch(
    project: &Path,
    target: &str,
    dir: Option<&str>,
    spawn: bool,
) -> Result<ButlerLaunch> {
    let project = std::fs::canonicalize(project).unwrap_or_else(|_| project.to_path_buf());
    let target = target.trim();
    if target.is_empty() {
        bail!("pass itch target as user/game:channel (e.g. you/mygame:windows)");
    }
    if !target.contains('/') {
        bail!("itch target must look like user/game or user/game:channel");
    }

    let dir_path = resolve_push_dir(&project, dir)?;
    let butler = match which("butler") {
        Ok(p) => p,
        Err(_) => {
            return Ok(ButlerLaunch {
                ok: false,
                project: project.display().to_string(),
                target: target.into(),
                dir: dir_path.display().to_string(),
                argv: vec![],
                recipe: String::new(),
                butler_path: None,
                spawned: false,
                hint: format!(
                    "butler not on PATH — install from {BUTLER_DOCS} then re-run `shipctl butler push` or ship_butler_push"
                ),
                docs_url: Some(BUTLER_DOCS.into()),
            });
        }
    };

    let butler_s = butler.display().to_string();
    let argv = vec![
        butler_s.clone(),
        "push".into(),
        dir_path.display().to_string(),
        target.into(),
    ];
    let recipe = shell_join(&argv);

    let mut spawned = false;
    if spawn {
        spawned = spawn_terminal("Ship Studio butler push", &recipe, &project)?;
    }

    Ok(ButlerLaunch {
        ok: true,
        project: project.display().to_string(),
        target: target.into(),
        dir: dir_path.display().to_string(),
        argv,
        recipe,
        butler_path: Some(butler_s),
        spawned,
        hint: if spawned {
            "Terminal opened — complete butler login/push there. Studio never stores itch credentials."
                .into()
        } else {
            "Run the recipe in a visible terminal (or call with spawn:true). Confirm submit.itch when the build is live."
                .into()
        },
        docs_url: Some(BUTLER_DOCS.into()),
    })
}

fn resolve_push_dir(project: &Path, dir: Option<&str>) -> Result<PathBuf> {
    if let Some(d) = dir.map(str::trim).filter(|s| !s.is_empty()) {
        let p = PathBuf::from(d);
        let abs = if p.is_absolute() {
            p
        } else {
            project.join(p)
        };
        if !abs.is_dir() {
            bail!("push dir is not a directory: {}", abs.display());
        }
        return Ok(std::fs::canonicalize(&abs).unwrap_or(abs));
    }

    let dist = project.join("dist");
    let build = project.join("build");
    let dist_ok = dist.is_dir();
    let build_ok = build.is_dir();
    if dist_ok && build_ok {
        bail!("both dist/ and build/ exist — pass dir explicitly (e.g. dist or build)");
    }
    if dist_ok {
        return Ok(std::fs::canonicalize(&dist).unwrap_or(dist));
    }
    if build_ok {
        return Ok(std::fs::canonicalize(&build).unwrap_or(build));
    }
    Ok(project.to_path_buf())
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

fn spawn_terminal(title: &str, recipe: &str, cwd: &Path) -> Result<bool> {
    #[cfg(target_os = "windows")]
    {
        let status = Command::new("cmd")
            .args(["/C", "start", title, "cmd", "/K", recipe])
            .current_dir(cwd)
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
            cwd = cwd.display().to_string().replace('"', "\\\""),
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
        let _ = (title, recipe, cwd);
        Ok(false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn rejects_empty_target() {
        let dir = std::env::temp_dir().join(format!(
            "shipctl-butler-empty-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_nanos())
                .unwrap_or(0)
        ));
        let _ = fs::create_dir_all(&dir);
        let err = push_launch(&dir, "", None, false).unwrap_err();
        assert!(err.to_string().contains("target"), "{err}");
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn ambiguous_dist_and_build_requires_dir() {
        let dir = std::env::temp_dir().join(format!(
            "shipctl-butler-ambig-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_nanos())
                .unwrap_or(0)
        ));
        let _ = fs::create_dir_all(dir.join("dist"));
        let _ = fs::create_dir_all(dir.join("build"));
        let err = push_launch(&dir, "me/game:windows", None, false).unwrap_err();
        assert!(err.to_string().contains("explicitly"), "{err}");
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn missing_butler_returns_ok_false() {
        // Force a PATH without butler by using a temp empty path — which::which still searches
        // real PATH. If butler is installed locally, ok may be true; assert shape either way
        // only when we get ok:false OR ok:true with recipe.
        let dir = std::env::temp_dir().join(format!(
            "shipctl-butler-miss-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_nanos())
                .unwrap_or(0)
        ));
        let _ = fs::create_dir_all(&dir);
        let launch = push_launch(&dir, "me/game:windows", Some(dir.to_str().unwrap()), false)
            .expect("launch");
        if launch.ok {
            assert!(launch.recipe.contains("push"), "{launch:?}");
            assert!(launch.recipe.contains("me/game:windows"), "{launch:?}");
            assert_eq!(launch.spawned, false);
        } else {
            assert!(launch.hint.contains("PATH"), "{launch:?}");
            assert_eq!(launch.docs_url.as_deref(), Some(BUTLER_DOCS));
        }
        let _ = fs::remove_dir_all(&dir);
    }
}
