//! Paste-secret assist — hint discovery + interactive put (never store values).

use crate::config;
use crate::portal::{self, ProviderId};
use anyhow::{bail, Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecretHint {
    pub provider: String,
    pub name: String,
    pub source: String,
    pub put_cli: Vec<String>,
    pub work_dir: String,
    pub entry_url: Option<String>,
    pub detail: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecretsPlan {
    pub schema: String,
    pub project: String,
    pub hints: Vec<SecretHint>,
    pub notes: Vec<String>,
}

pub fn plan_for(project: &Path, filter: Option<ProviderId>) -> Result<SecretsPlan> {
    let project = fs::canonicalize(project).unwrap_or_else(|_| project.to_path_buf());
    let detected = config::probe(&project);
    let mut providers = portal::detected_providers(&detected);
    if let Some(f) = filter {
        providers.retain(|p| *p == f);
        if providers.is_empty() {
            providers.push(f);
        }
    }
    if providers.is_empty() {
        providers = ProviderId::all().to_vec();
    }

    let mut hints = Vec::new();
    for id in &providers {
        hints.extend(hints_for_provider(&project, *id)?);
    }
    dedupe_hints(&mut hints);

    Ok(SecretsPlan {
        schema: "ship-studio/secrets/v1".into(),
        project: project.display().to_string(),
        hints,
        notes: vec![
            "Paste values into the provider CLI — shipctl never stores secret values.".into(),
            "Run: shipctl secrets put --project . --provider cloudflare --name <NAME>".into(),
            "Or use TUI → Secrets → Enter to put the selected hint.".into(),
            "Optional backup: shipctl vault export --out ship-secrets.km --from-hints".into(),
        ],
    })
}

fn hints_for_provider(project: &Path, id: ProviderId) -> Result<Vec<SecretHint>> {
    let cat = portal::catalog_entry(id);
    let mut names: Vec<(String, String)> = Vec::new();

    match id {
        ProviderId::Cloudflare => {
            if let Some(w) = find_wrangler_file(project) {
                for n in parse_wrangler_secret_names(&w)? {
                    names.push((n, format!("wrangler comment · {}", w.display())));
                }
            }
            for (n, src) in empty_env_keys(project, &[".dev.vars"]) {
                if !names.iter().any(|(x, _)| x == &n) {
                    names.push((n, src));
                }
            }
        }
        ProviderId::Vercel | ProviderId::Netlify => {
            // Do not treat Cloudflare `.dev.vars` as Vercel/Netlify env.
            for (n, src) in empty_env_keys(project, &[".env", ".env.local"]) {
                names.push((n, src));
            }
        }
        ProviderId::Github => {
            names.push(("GITHUB_TOKEN".into(), "catalog".into()));
        }
        ProviderId::Polar => {
            names.push(("POLAR_CHECKOUT_URL".into(), "catalog".into()));
            names.push(("POLAR_WEBHOOK_SECRET".into(), "catalog".into()));
            for (n, src) in empty_env_keys(project, &[".env", ".env.local", ".dev.vars"]) {
                if n.starts_with("POLAR_") && !names.iter().any(|(x, _)| x == &n) {
                    names.push((n, src));
                }
            }
        }
    }

    let work = match id {
        ProviderId::Cloudflare => find_wrangler_dir(project).unwrap_or_else(|| project.to_path_buf()),
        _ => project.to_path_buf(),
    };

    let mut out = Vec::new();
    for (name, source) in names {
        let put_cli = put_cli_for(id, &name);
        let source_url = portal::source_url_for_secret_name(&name);
        let once = portal::once_hint_for_secret_name(&name);
        let detail = match id {
            ProviderId::Github => {
                format!("{once} Then put on the deploy target (e.g. wrangler secret put GITHUB_TOKEN).")
            }
            ProviderId::Polar => {
                format!("{once} Then put on Cloudflare/Vercel with secrets put.")
            }
            _ => format!(
                "Put destination: {}. Value source: {source_url}. {once} Source: {source}",
                id.as_str()
            ),
        };
        out.push(SecretHint {
            provider: id.as_str().into(),
            name,
            source,
            put_cli,
            work_dir: work.display().to_string(),
            entry_url: Some(source_url.into()),
            detail,
        });
    }

    if out.is_empty() && matches!(id, ProviderId::Cloudflare | ProviderId::Vercel | ProviderId::Netlify)
    {
        out.push(SecretHint {
            provider: id.as_str().into(),
            name: "<NAME>".into(),
            source: "template".into(),
            put_cli: put_cli_for(id, "<NAME>"),
            work_dir: work.display().to_string(),
            entry_url: Some(cat.create_url.into()),
            detail: format!(
                "No names discovered — replace <NAME> or add a `# Secrets:` block in wrangler / fill .dev.vars keys. {} {}",
                cat.env_hint, cat.once_hint
            ),
        });
    }

    Ok(out)
}

fn put_cli_for(id: ProviderId, name: &str) -> Vec<String> {
    match id {
        ProviderId::Cloudflare => vec!["wrangler".into(), "secret".into(), "put".into(), name.into()],
        ProviderId::Vercel => vec!["vercel".into(), "env".into(), "add".into(), name.into()],
        ProviderId::Netlify => vec!["netlify".into(), "env:set".into(), name.into()],
        ProviderId::Github => vec!["gh".into(), "auth".into(), "login".into()],
        ProviderId::Polar => vec![
            "shipctl".into(),
            "portal".into(),
            "--provider".into(),
            "polar".into(),
            "--open".into(),
        ],
    }
}

/// Interactive put — value is entered in the child CLI, not captured by shipctl.
pub fn put_secret(project: &Path, provider: ProviderId, name: &str) -> Result<i32> {
    if name.is_empty() || name == "<NAME>" {
        bail!("pass a real secret name with --name");
    }
    if provider == ProviderId::Github {
        bail!("GitHub has no deploy secret put here — open the token page, then put on cloudflare/vercel/netlify");
    }
    if provider == ProviderId::Polar {
        bail!("Polar has no secret put CLI — open polar.sh dashboard, then `shipctl secrets put --provider cloudflare --name POLAR_…`");
    }
    let project = fs::canonicalize(project).unwrap_or_else(|_| project.to_path_buf());
    let work = match provider {
        ProviderId::Cloudflare => find_wrangler_dir(&project).unwrap_or(project.clone()),
        _ => project.clone(),
    };
    let args = put_cli_for(provider, name);
    let bin = &args[0];
    let rest = &args[1..];
    let status = Command::new(bin)
        .args(rest)
        .current_dir(&work)
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()
        .with_context(|| format!("run {} in {}", bin, work.display()))?;
    Ok(status.code().unwrap_or(-1))
}

pub fn open_entry_urls(plan: &SecretsPlan) -> Result<Vec<String>> {
    let mut opened = Vec::new();
    let mut seen = std::collections::HashSet::new();
    for h in &plan.hints {
        let Some(url) = h.entry_url.as_deref() else {
            continue;
        };
        if !seen.insert(url.to_string()) {
            continue;
        }
        portal::open_url(url)?;
        opened.push(url.to_string());
    }
    Ok(opened)
}

fn find_wrangler_file(project: &Path) -> Option<PathBuf> {
    for name in ["wrangler.toml", "wrangler.json", "wrangler.jsonc"] {
        let p = project.join(name);
        if p.is_file() {
            return Some(p);
        }
    }
    // nested one / apps/*
    if let Ok(entries) = fs::read_dir(project) {
        for ent in entries.flatten() {
            let p = ent.path();
            if !p.is_dir() {
                continue;
            }
            let n = ent.file_name().to_string_lossy().to_lowercase();
            if n == "node_modules" || n == ".git" || n == "target" || n == "dist" {
                continue;
            }
            for name in ["wrangler.toml", "wrangler.json", "wrangler.jsonc"] {
                let c = p.join(name);
                if c.is_file() {
                    return Some(c);
                }
            }
            if n == "apps" {
                if let Ok(apps) = fs::read_dir(&p) {
                    for app in apps.flatten() {
                        let ap = app.path();
                        if !ap.is_dir() {
                            continue;
                        }
                        for name in ["wrangler.toml", "wrangler.json", "wrangler.jsonc"] {
                            let c = ap.join(name);
                            if c.is_file() {
                                return Some(c);
                            }
                        }
                    }
                }
            }
        }
    }
    None
}

fn find_wrangler_dir(project: &Path) -> Option<PathBuf> {
    find_wrangler_file(project).and_then(|f| f.parent().map(|p| p.to_path_buf()))
}

pub fn parse_wrangler_secret_names(wrangler_path: &Path) -> Result<Vec<String>> {
    let raw = fs::read_to_string(wrangler_path).unwrap_or_default();
    let mut names = Vec::new();
    let mut seen = std::collections::HashSet::new();
    let mut in_block = false;
    for line in raw.lines() {
        let trim = line.trim();
        if !trim.starts_with('#') {
            if in_block {
                break;
            }
            continue;
        }
        let body = trim.trim_start_matches('#').trim();
        let lower = body.to_ascii_lowercase();
        if is_secrets_header(&lower) {
            in_block = true;
            if let Some(idx) = body.find(':') {
                add_secret_tokens(&body[idx + 1..], &mut seen, &mut names);
            }
            continue;
        }
        if in_block {
            if body.is_empty() {
                break;
            }
            add_secret_tokens(body, &mut seen, &mut names);
        }
    }
    Ok(names)
}

fn is_secrets_header(lower: &str) -> bool {
    lower.starts_with("secrets")
        || lower.contains("secrets (")
        || lower.contains("secrets:")
}

fn add_secret_tokens(text: &str, seen: &mut std::collections::HashSet<String>, names: &mut Vec<String>) {
    for tok in extract_env_tokens(text) {
        if seen.insert(tok.clone()) {
            names.push(tok);
        }
    }
}

fn extract_env_tokens(text: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut cur = String::new();
    for ch in text.chars() {
        if ch.is_ascii_uppercase() || ch.is_ascii_digit() || ch == '_' {
            if cur.is_empty() && !ch.is_ascii_uppercase() {
                continue;
            }
            cur.push(ch);
        } else if !cur.is_empty() {
            if cur.len() >= 3 && cur.chars().next().is_some_and(|c| c.is_ascii_uppercase()) {
                out.push(std::mem::take(&mut cur));
            } else {
                cur.clear();
            }
        }
    }
    if cur.len() >= 3 && cur.chars().next().is_some_and(|c| c.is_ascii_uppercase()) {
        out.push(cur);
    }
    out
}

fn empty_env_keys(project: &Path, filenames: &[&str]) -> Vec<(String, String)> {
    let mut out = Vec::new();
    let mut roots = vec![project.to_path_buf()];
    // Cloudflare local files live next to wrangler.toml (often apps/api/).
    if filenames.iter().any(|f| *f == ".dev.vars") {
        if let Some(wdir) = find_wrangler_dir(project) {
            if wdir != project {
                roots.push(wdir);
            }
        }
    }
    for root in roots {
        for name in filenames {
            let path = root.join(name);
            if !path.is_file() {
                continue;
            }
            let label = if root == project {
                (*name).to_string()
            } else {
                format!("{} / {}", root.display(), name)
            };
            out.extend(parse_empty_keys(&path, &label));
        }
    }
    out
}

/// Keep one hint per secret name, preferring real put destinations over catalog stubs.
fn dedupe_hints(hints: &mut Vec<SecretHint>) {
    fn rank(provider: &str) -> u8 {
        match provider {
            "cloudflare" => 0,
            "vercel" => 1,
            "netlify" => 2,
            "github" => 3,
            "polar" => 4,
            _ => 9,
        }
    }
    let mut best: std::collections::BTreeMap<String, SecretHint> =
        std::collections::BTreeMap::new();
    for h in hints.drain(..) {
        match best.get(&h.name) {
            None => {
                best.insert(h.name.clone(), h);
            }
            Some(prev) => {
                if rank(&h.provider) < rank(&prev.provider) {
                    best.insert(h.name.clone(), h);
                }
            }
        }
    }
    let mut names: Vec<String> = best.keys().cloned().collect();
    names.sort_by(|a, b| {
        let rank_name = |n: &str| {
            if n == "GITHUB_TOKEN" {
                0
            } else if n.starts_with("POLAR_") {
                1
            } else if n == "API_KEY_PEPPER" {
                3
            } else {
                2
            }
        };
        rank_name(a).cmp(&rank_name(b)).then(a.cmp(b))
    });
    *hints = names
        .into_iter()
        .filter_map(|n| best.remove(&n))
        .collect();
}

fn parse_empty_keys(path: &Path, source_label: &str) -> Vec<(String, String)> {
    let Ok(raw) = fs::read_to_string(path) else {
        return Vec::new();
    };
    let mut out = Vec::new();
    for line in raw.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let Some((k, v)) = line.split_once('=') else {
            continue;
        };
        let key = k.trim();
        let val = v.trim().trim_matches('"').trim_matches('\'');
        if key.is_empty() {
            continue;
        }
        if val.is_empty() || val.eq_ignore_ascii_case("changeme") || val == "…" {
            out.push((key.to_string(), format!("empty key · {source_label}")));
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tempfile_dir() -> PathBuf {
        let dir = std::env::temp_dir().join(format!(
            "shipctl-secrets-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_nanos())
                .unwrap_or(0)
        ));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn parse_wrangler_secrets_block() {
        let dir = tempfile_dir();
        let w = dir.join("wrangler.toml");
        fs::write(
            &w,
            r#"
name = "x"
# Secrets (set via wrangler secret put):
# GITHUB_TOKEN
# POLAR_WEBHOOK_SECRET
# POLAR_CHECKOUT_URL

[vars]
"#,
        )
        .unwrap();
        let names = parse_wrangler_secret_names(&w).unwrap();
        assert_eq!(
            names,
            vec![
                "GITHUB_TOKEN".to_string(),
                "POLAR_WEBHOOK_SECRET".to_string(),
                "POLAR_CHECKOUT_URL".to_string()
            ]
        );
    }

    #[test]
    fn empty_dev_vars_become_hints() {
        let dir = tempfile_dir();
        fs::write(dir.join("wrangler.toml"), "name = \"x\"\n").unwrap();
        fs::write(
            dir.join(".dev.vars"),
            "API_KEY_PEPPER=set\nGITHUB_TOKEN=\nPOLAR_CHECKOUT_URL=\n",
        )
        .unwrap();
        let plan = plan_for(&dir, Some(ProviderId::Cloudflare)).unwrap();
        let names: Vec<_> = plan.hints.iter().map(|h| h.name.as_str()).collect();
        assert!(names.contains(&"GITHUB_TOKEN"));
        assert!(names.contains(&"POLAR_CHECKOUT_URL"));
        assert!(!names.contains(&"API_KEY_PEPPER"));
    }

    #[test]
    fn dedupe_prefers_cloudflare_over_catalog() {
        let dir = tempfile_dir();
        fs::write(
            dir.join("wrangler.toml"),
            "# Secrets\n# - GITHUB_TOKEN\n# - POLAR_WEBHOOK_SECRET\nname = \"x\"\n",
        )
        .unwrap();
        let plan = plan_for(&dir, None).unwrap();
        let gh: Vec<_> = plan
            .hints
            .iter()
            .filter(|h| h.name == "GITHUB_TOKEN")
            .collect();
        assert_eq!(gh.len(), 1, "expected one GITHUB_TOKEN hint, got {gh:?}");
        assert_eq!(gh[0].provider, "cloudflare");
    }
}
