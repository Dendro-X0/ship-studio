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
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub portal_providers: Vec<String>,
    pub secret_hint_count: usize,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub provider_clis: Vec<ProviderCliStatus>,
    pub notes: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct ProviderCliStatus {
    pub provider: String,
    pub bin: String,
    pub found: bool,
    pub fix: String,
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

    let portal_providers = crate::portal::detected_providers(&detected)
        .into_iter()
        .map(|p| p.as_str().to_string())
        .collect::<Vec<_>>();
    let provider_clis = provider_cli_status(&detected);
    let secret_hint_count = crate::secrets::plan_for(&project, None)
        .map(|p| {
            p.hints
                .iter()
                .filter(|h| h.name != "<NAME>")
                .count()
        })
        .unwrap_or(0);

    let mut notes = Vec::new();
    notes.push("Bridge is offline-first: it does not call vendor HTTPS itself.".into());
    if !signet.found {
        let wants_signet = detected.tauri || detected.signet_toml;
        if wants_signet {
            notes.push("signet not on PATH — install Signet or set SIGNET_PATH.".into());
        } else {
            notes.push("signet not on PATH — optional for this layout (no Tauri/signet.toml).".into());
        }
    }
    if !orbit.found {
        let needs_host = detected.wrangler || detected.vercel || detected.netlify;
        if needs_host {
            notes.push("orbit not on PATH — build Orbit or set ORBIT_PATH (or use wrangler/vercel/netlify CLI).".into());
        } else {
            notes.push("orbit not on PATH — optional for this layout (no Web/API host detected).".into());
        }
    }
    if !exists {
        notes.push("project path is not a directory.".into());
    }
    if !portal_providers.is_empty() {
        notes.push(format!(
            "Portal providers: {} — run `shipctl portal` or TUI wizard.",
            portal_providers.join(", ")
        ));
    }
    for cli in &provider_clis {
        if !cli.found {
            notes.push(format!(
                "{} CLI `{}` missing — {}",
                cli.provider, cli.bin, cli.fix
            ));
        }
    }
    if secret_hint_count > 0 {
        notes.push(format!(
            "{secret_hint_count} secret hint(s) — run `shipctl secrets` to paste via provider CLI."
        ));
    }
    notes.push("Full checklist: `shipctl guide` · one-shot prep: `shipctl ship`".into());
    notes.extend(detected.hints.iter().cloned());

    // Final-mile readiness (shipping hub band #11).
    if signet.found {
        notes.push("Cut ready: Signet on PATH — Publish can Run build / release / graduate.".into());
    }
    if orbit.found {
        notes.push("Cut ready: Orbit on PATH — Publish can Run Web/API deploy.".into());
    }
    if detected.container {
        let docker_ok = which("docker").is_ok() || which("docker.exe").is_ok();
        if docker_ok {
            notes.push(
                "Cut ready: docker on PATH — Advanced Publish can Run container.build (push stays Confirm)."
                    .into(),
            );
        } else {
            notes.push(
                "Container layout detected but docker missing — install Docker so container.build can Run."
                    .into(),
            );
        }
    }
    if detected.ci_release {
        let gh_ok = which("gh").is_ok() || which("gh.exe").is_ok();
        if gh_ok {
            notes.push(
                "Cut ready: gh on PATH — Advanced ci.release / release.github can Run `gh` list commands (read-only)."
                    .into(),
            );
        } else {
            notes.push(
                "Release workflow(s) detected but gh missing — install GitHub CLI for ci.release Run."
                    .into(),
            );
        }
    } else if detected.github {
        let gh_ok = which("gh").is_ok() || which("gh.exe").is_ok();
        if gh_ok {
            notes.push(
                "Cut ready: gh on PATH — Advanced release.github can Run `gh release list` (read-only)."
                    .into(),
            );
        }
    }
    if detected.npm_publish {
        let npm_ok = which("npm").is_ok() || which("npm.cmd").is_ok() || which("npm.exe").is_ok();
        if npm_ok {
            notes.push(
                "Cut ready: npm on PATH — Advanced listing.npm can Run `npm publish --dry-run`."
                    .into(),
            );
        }
    }
    if detected.crates_publish {
        let cargo_ok = which("cargo").is_ok() || which("cargo.exe").is_ok();
        if cargo_ok {
            notes.push(
                "Cut ready: cargo on PATH — Advanced listing.crates can Run `cargo publish --dry-run`."
                    .into(),
            );
        }
    }
    if detected.graduate_sign {
        notes.push(
            "Graduate opted in — Advanced sign.graduate runs `signet graduate notes` (then apply/ov-sign/notarize)."
                .into(),
        );
    }
    if (detected.tauri || detected.signet_toml)
        && !(detected.wrangler || detected.vercel || detected.netlify)
    {
        notes.push(
            "Desktop-only layout — final-mile cut is Signet release (+ marketing), not Orbit deploy."
                .into(),
        );
    }
    if detected.fly || detected.railway || detected.render || detected.digitalocean || detected.heroku || detected.amplify {
        let mut m = Vec::new();
        if detected.fly {
            m.push("Fly");
        }
        if detected.railway {
            m.push("Railway");
        }
        if detected.render {
            m.push("Render");
        }
        if detected.digitalocean {
            m.push("DigitalOcean");
        }
        if detected.heroku {
            m.push("Heroku");
        }
        if detected.amplify {
            m.push("Amplify");
        }
        notes.push(format!(
            "Alt hosts ({}) — Advanced host.* opens dashboards; deploy stays on their CLI/UI (optional flyctl/railway/doctl/heroku; not required for doctor ok).",
            m.join(" · ")
        ));
    }
    if detected.mobile
        && (detected.firebase || detected.appwrite || detected.convex || detected.supabase)
    {
        notes.push(
            "Mobile BaaS — Advanced baas.provision opens Firebase/Appwrite/Convex/Supabase console."
                .into(),
        );
    }
    if detected.stripe || detected.paddle {
        let mut m = Vec::new();
        if detected.stripe {
            m.push("Stripe");
        }
        if detected.paddle {
            m.push("Paddle");
        }
        notes.push(format!(
            "Commerce ({}) — Advanced listing.* opens SKU dashboards; no Payment Link creation from Studio.",
            m.join(" · ")
        ));
    }
    notes.push(
        "Progress nudge: `shipctl publish watch` (CLI) · Desktop Watch · TUI `w` · MCP `ship_publish_watch` — local Verify only."
            .into(),
    );

    let wants_signet = detected.tauri || detected.signet_toml;
    let needs_host = detected.wrangler || detected.vercel || detected.netlify;
    let provider_cli_ok = provider_clis.iter().any(|c| {
        c.found
            && matches!(
                c.provider.as_str(),
                "cloudflare" | "vercel" | "netlify"
            )
    });
    let ok = doctor_tools_ok(exists, wants_signet, needs_host, signet.found, orbit.found, provider_cli_ok);
    Ok(DoctorReport {
        ok,
        project: project.display().to_string(),
        project_exists: exists,
        offline_bridge: true,
        signet,
        orbit,
        studio,
        detected,
        portal_providers,
        secret_hint_count,
        provider_clis,
        notes,
    })
}

/// Whether Doctor should report ok for this layout (band #15).
pub fn doctor_tools_ok(
    exists: bool,
    wants_signet: bool,
    needs_host: bool,
    signet_found: bool,
    orbit_found: bool,
    provider_cli_ok: bool,
) -> bool {
    if !exists {
        return false;
    }
    if wants_signet && !signet_found {
        return false;
    }
    if needs_host && !(orbit_found || provider_cli_ok) {
        return false;
    }
    true
}

fn provider_cli_status(detected: &crate::config::Detected) -> Vec<ProviderCliStatus> {
    let mut out = Vec::new();
    let checks: &[(&str, bool, &str, &str)] = &[
        (
            "cloudflare",
            detected.wrangler,
            "wrangler",
            "npm i -g wrangler (or use project pnpm exec wrangler)",
        ),
        (
            "vercel",
            detected.vercel,
            "vercel",
            "npm i -g vercel",
        ),
        (
            "netlify",
            detected.netlify,
            "netlify",
            "npm i -g netlify-cli",
        ),
        (
            "github",
            detected.github,
            "gh",
            "install GitHub CLI: https://cli.github.com/",
        ),
        (
            "container",
            detected.container,
            "docker",
            "install Docker Desktop / engine so Publish can Run `docker build`",
        ),
        (
            "fly",
            detected.fly,
            "flyctl",
            "optional: install flyctl — Studio only opens the Fly dashboard",
        ),
        (
            "railway",
            detected.railway,
            "railway",
            "optional: install Railway CLI — Studio only opens the Railway dashboard",
        ),
        (
            "digitalocean",
            detected.digitalocean,
            "doctl",
            "optional: install doctl — Studio only opens the DigitalOcean dashboard",
        ),
        (
            "heroku",
            detected.heroku,
            "heroku",
            "optional: install Heroku CLI — Studio only opens the Heroku dashboard",
        ),
    ];
    for (provider, needed, bin, fix) in checks {
        if !*needed {
            continue;
        }
        let mut found = which(bin).is_ok() || which(format!("{bin}.exe")).is_ok();
        // Fly ships as flyctl or fly.
        if !found && *provider == "fly" {
            found = which("fly").is_ok() || which("fly.exe").is_ok();
        }
        out.push(ProviderCliStatus {
            provider: (*provider).into(),
            bin: (*bin).into(),
            found,
            fix: (*fix).into(),
        });
    }
    out
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
    fn doctor_tools_ok_matrix() {
        // Library-only: no Signet/host tools required.
        assert!(doctor_tools_ok(true, false, false, false, false, false));
        assert!(!doctor_tools_ok(false, false, false, false, false, false));
        // Desktop Signet: needs Signet, not Orbit.
        assert!(doctor_tools_ok(true, true, false, true, false, false));
        assert!(!doctor_tools_ok(true, true, false, false, true, false));
        // Workers host: Orbit or provider CLI.
        assert!(doctor_tools_ok(true, false, true, false, true, false));
        assert!(doctor_tools_ok(true, false, true, false, false, true));
        assert!(!doctor_tools_ok(true, false, true, false, false, false));
        // Signet + host: both sides.
        assert!(doctor_tools_ok(true, true, true, true, true, false));
        assert!(doctor_tools_ok(true, true, true, true, false, true));
        assert!(!doctor_tools_ok(true, true, true, true, false, false));
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

    #[test]
    fn configure_suggests_netlify_deploy_args() {
        let dir = tempfile_dir();
        std::fs::write(dir.join("netlify.toml"), "[build]\n").unwrap();
        let intent = crate::config::configure(&dir).expect("configure");
        assert_eq!(
            intent.deploy_args,
            vec![
                "deploy".to_string(),
                "--provider".to_string(),
                "netlify".to_string()
            ]
        );
        assert!(intent.detected.netlify);
    }

    #[test]
    fn doctor_notes_hosts_baas_commerce_and_watch() {
        let dir = tempfile_dir();
        std::fs::create_dir_all(dir.join("android")).unwrap();
        std::fs::write(dir.join("fly.toml"), "app = \"demo\"\n").unwrap();
        std::fs::write(dir.join("firebase.json"), "{}\n").unwrap();
        std::fs::write(dir.join("build.gradle"), "// android\n").unwrap();
        std::fs::write(dir.join(".env"), "STRIPE_SECRET_KEY=\nPADDLE_API_KEY=\n").unwrap();
        let report = doctor(&dir).expect("doctor");
        assert!(report.ok, "alt-host without flyctl must still be doctor ok");
        assert!(report
            .notes
            .iter()
            .any(|n| n.contains("host.*") || n.contains("Fly")));
        assert!(report.notes.iter().any(|n| n.contains("baas.provision")));
        assert!(report
            .notes
            .iter()
            .any(|n| n.contains("Stripe") || n.contains("listing.*")));
        assert!(report.notes.iter().any(|n| n.contains("publish watch")));
        assert!(report
            .provider_clis
            .iter()
            .any(|c| c.provider == "fly" && c.bin == "flyctl"));
    }

    fn tempfile_dir() -> PathBuf {
        use std::sync::atomic::{AtomicU64, Ordering};
        static N: AtomicU64 = AtomicU64::new(0);
        let dir = std::env::temp_dir().join(format!(
            "shipctl-test-{}-{}-{}",
            std::process::id(),
            N.fetch_add(1, Ordering::Relaxed),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_nanos())
                .unwrap_or(0)
        ));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }
}
