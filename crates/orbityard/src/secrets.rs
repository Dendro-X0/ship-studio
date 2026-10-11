//! Paste-secret assist — hint discovery + interactive put (never store values).
//! Bulk import from local `.env` / `.dev.vars` pushes values to the host CLI only
//! (stdin / ephemeral temp for Netlify) — never into `.ship/` or logs.

use crate::config;
use crate::portal::{self, ProviderId};
use anyhow::{bail, Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

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
    // Empty detect → empty hints (not ProviderId::all). Obstacle-course diet.
    let detect_empty = providers.is_empty() && filter.is_none();

    let mut hints = Vec::new();
    for id in &providers {
        hints.extend(hints_for_provider(&project, *id)?);
    }
    hints.extend(graduate_commerce_catalog_hints(&project, &detected));
    dedupe_hints(&mut hints);

    let mut notes = vec![
        "Paste values into the provider CLI — orbityard never stores secret values.".into(),
        "Run: orbityard secrets put --project . --provider cloudflare --name <NAME>".into(),
        "Or use TUI → Secrets → Enter to put the selected hint.".into(),
        "Graduate / Gumroad / Lemon / Stripe / Paddle rows are name-only catalogs — set in CI or vendor dashboards, not .ship/.".into(),
        "Optional backup: orbityard vault export --out ship-secrets.km --from-hints".into(),
    ];
    if detect_empty {
        notes.insert(
            0,
            "No host/commerce providers detected — choose a host on Platforms or opt in via .ship/markets (no full-catalog flood).".into(),
        );
    }

    Ok(SecretsPlan {
        schema: "orbit-yard/secrets/v1".into(),
        project: project.display().to_string(),
        hints,
        notes,
    })
}

fn graduate_commerce_catalog_hints(
    project: &Path,
    detected: &config::Detected,
) -> Vec<SecretHint> {
    let mut out = Vec::new();
    let work = project.display().to_string();

    if detected.graduate_sign {
        let names = [
            ("SIGNET_OV_CERT", "OV / code-signing cert material (CI secret)"),
            ("SIGNET_AZURE_CLIENT_ID", "Azure Trusted Signing app id"),
            ("SIGNET_AZURE_CLIENT_SECRET", "Azure Trusted Signing secret"),
            ("SIGNET_AZURE_TENANT_ID", "Azure tenant id"),
            ("SIGNET_NOTARY_PROFILE", "Apple notarytool profile name"),
            ("WIN_CERT_PFX_PASS", "Windows PFX password if using local cert"),
            ("APPLE_API_KEY_ID", "App Store Connect API key id"),
            ("APPLE_API_ISSUER_ID", "App Store Connect issuer id"),
        ];
        for (name, why) in names {
            out.push(SecretHint {
                provider: "graduate".into(),
                name: name.into(),
                source: "catalog · graduate".into(),
                put_cli: vec![
                    "orbityard".into(),
                    "portal".into(),
                    "--provider".into(),
                    "github".into(),
                    "--open".into(),
                ],
                work_dir: work.clone(),
                entry_url: Some(portal::source_url_for_secret_name(name).into()),
                detail: format!(
                    "{why}. Set as GitHub Actions / local Signet env — never commit. {}",
                    portal::once_hint_for_secret_name(name)
                ),
            });
        }
        for (n, src) in empty_env_keys(
            project,
            &[".env", ".env.local", ".dev.vars"],
        ) {
            if (n.starts_with("SIGNET_OV_")
                || n.starts_with("SIGNET_AZURE_")
                || n.starts_with("SIGNET_NOTARY_")
                || n.starts_with("WIN_CERT_")
                || n.starts_with("APPLE_API_")
                || n.starts_with("NOTARY_"))
                && !out.iter().any(|h| h.name == n)
            {
                out.push(SecretHint {
                    provider: "graduate".into(),
                    name: n,
                    source: src,
                    put_cli: vec![
                        "orbityard".into(),
                        "portal".into(),
                        "--provider".into(),
                        "github".into(),
                        "--open".into(),
                    ],
                    work_dir: work.clone(),
                    entry_url: Some(
                        "https://learn.microsoft.com/en-us/azure/trusted-signing/".into(),
                    ),
                    detail: "Empty env key for graduate signing — fill in CI secrets, not .ship/."
                        .into(),
                });
            }
        }
    }

    if detected.gumroad {
        for (name, why) in [
            ("GUMROAD_ACCESS_TOKEN", "Gumroad API / access token"),
            ("GUMROAD_PRODUCT_ID", "Product permalink or id"),
            ("GUMROAD_CHECKOUT_URL", "Public checkout CTA URL"),
        ] {
            out.push(SecretHint {
                provider: "gumroad".into(),
                name: name.into(),
                source: "catalog · gumroad".into(),
                put_cli: vec![
                    "orbityard".into(),
                    "portal".into(),
                    "--provider".into(),
                    "gumroad".into(),
                    "--open".into(),
                ],
                work_dir: work.clone(),
                entry_url: Some("https://app.gumroad.com/".into()),
                detail: format!("{why}. Create on Gumroad; put checkout URL into the marketing CTA."),
            });
        }
    }

    if detected.lemon {
        for (name, why) in [
            ("LEMON_API_KEY", "Lemon Squeezy API key"),
            ("LEMONSQUEEZY_WEBHOOK_SECRET", "Webhook signing secret"),
            ("LEMON_CHECKOUT_URL", "Public checkout / buy URL"),
        ] {
            out.push(SecretHint {
                provider: "lemon".into(),
                name: name.into(),
                source: "catalog · lemon".into(),
                put_cli: vec![
                    "orbityard".into(),
                    "portal".into(),
                    "--provider".into(),
                    "lemon".into(),
                    "--open".into(),
                ],
                work_dir: work.clone(),
                entry_url: Some("https://app.lemonsqueezy.com/".into()),
                detail: format!("{why}. Create on Lemon; wire fulfillment webhook separately."),
            });
        }
    }

    if detected.stripe {
        for (name, why) in [
            ("STRIPE_SECRET_KEY", "Stripe secret API key"),
            ("STRIPE_PUBLISHABLE_KEY", "Stripe publishable key"),
            ("STRIPE_WEBHOOK_SECRET", "Webhook signing secret"),
            ("STRIPE_PRICE_ID", "Price / Payment Link id"),
        ] {
            out.push(SecretHint {
                provider: "stripe".into(),
                name: name.into(),
                source: "catalog · stripe".into(),
                put_cli: vec![
                    "orbityard".into(),
                    "portal".into(),
                    "--provider".into(),
                    "stripe".into(),
                    "--open".into(),
                ],
                work_dir: work.clone(),
                entry_url: Some("https://dashboard.stripe.com/".into()),
                detail: format!(
                    "{why}. Create on Stripe; put on the deploy target — never in .ship/."
                ),
            });
        }
    }

    if detected.paddle {
        for (name, why, url) in [
            (
                "PUBLIC_PADDLE_CLIENT_TOKEN",
                "Paddle.js client-side token (test_… on sandbox)",
                "https://sandbox-vendors.paddle.com/authentication",
            ),
            (
                "PUBLIC_PADDLE_PRICE_ID",
                "Catalog price id (pri_…)",
                "https://sandbox-vendors.paddle.com/products",
            ),
        ] {
            out.push(SecretHint {
                provider: "paddle".into(),
                name: name.into(),
                source: "catalog · paddle".into(),
                put_cli: vec![
                    "orbityard".into(),
                    "portal".into(),
                    "--provider".into(),
                    "paddle".into(),
                    "--open".into(),
                ],
                work_dir: work.clone(),
                entry_url: Some(url.into()),
                detail: format!(
                    "{why}. Create on Paddle sandbox; put on the website host — never in .ship/."
                ),
            });
        }
    }

    if detected.creem {
        for (name, why) in [
            ("CREEM_API_KEY", "Creem API key"),
            ("CREEM_WEBHOOK_SECRET", "Webhook signing secret"),
            ("CREEM_PRODUCT_ID", "Product id"),
        ] {
            out.push(SecretHint {
                provider: "creem".into(),
                name: name.into(),
                source: "catalog · creem".into(),
                put_cli: vec![
                    "orbityard".into(),
                    "portal".into(),
                    "--provider".into(),
                    "creem".into(),
                    "--open".into(),
                ],
                work_dir: work.clone(),
                entry_url: Some("https://creem.io/dashboard".into()),
                detail: format!(
                    "{why}. Create on Creem; put on the deploy target — never in .ship/."
                ),
            });
        }
    }

    if detected.waffo {
        for (name, why) in [
            ("WAFFO_MERCHANT_ID", "Waffo merchant id"),
            ("WAFFO_PRIVATE_KEY", "Waffo private key"),
            ("WAFFO_WEBHOOK_SECRET", "Webhook signing secret"),
        ] {
            out.push(SecretHint {
                provider: "waffo".into(),
                name: name.into(),
                source: "catalog · waffo".into(),
                put_cli: vec![
                    "orbityard".into(),
                    "portal".into(),
                    "--provider".into(),
                    "waffo".into(),
                    "--open".into(),
                ],
                work_dir: work.clone(),
                entry_url: Some("https://pancake.waffo.ai/merchant/auth/signin".into()),
                detail: format!(
                    "{why}. Create on Waffo; put on the deploy target — never in .ship/."
                ),
            });
        }
    }

    out
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
        ProviderId::Neon => {
            names.push(("DATABASE_URL".into(), "catalog".into()));
            names.push(("NEON_DATABASE_URL".into(), "catalog".into()));
            for (n, src) in empty_env_keys(project, &[".env", ".env.local", ".dev.vars"]) {
                if (n.starts_with("NEON_") || n == "DATABASE_URL")
                    && !names.iter().any(|(x, _)| x == &n)
                {
                    names.push((n, src));
                }
            }
        }
        ProviderId::Supabase => {
            names.push(("SUPABASE_URL".into(), "catalog".into()));
            names.push(("SUPABASE_ANON_KEY".into(), "catalog".into()));
            names.push(("SUPABASE_SERVICE_ROLE_KEY".into(), "catalog".into()));
            names.push(("DATABASE_URL".into(), "catalog".into()));
            for (n, src) in empty_env_keys(project, &[".env", ".env.local", ".dev.vars"]) {
                if (n.starts_with("SUPABASE_") || n == "DATABASE_URL")
                    && !names.iter().any(|(x, _)| x == &n)
                {
                    names.push((n, src));
                }
            }
        }
        ProviderId::D1 => {
            names.push(("DATABASE_URL".into(), "catalog · optional (D1 is usually a binding)".into()));
            for (n, src) in empty_env_keys(project, &[".dev.vars", ".env", ".env.local"]) {
                if (n == "DATABASE_URL" || n.contains("D1")) && !names.iter().any(|(x, _)| x == &n)
                {
                    names.push((n, src));
                }
            }
        }
        ProviderId::Turso => {
            names.push(("TURSO_DATABASE_URL".into(), "catalog".into()));
            names.push(("TURSO_AUTH_TOKEN".into(), "catalog".into()));
            for (n, src) in empty_env_keys(project, &[".env", ".env.local", ".dev.vars"]) {
                if (n.starts_with("TURSO_") || n.starts_with("LIBSQL_"))
                    && !names.iter().any(|(x, _)| x == &n)
                {
                    names.push((n, src));
                }
            }
        }
        ProviderId::Container => {
            // First slice: docs/portal only — no forced registry secret names.
        }
        ProviderId::Firebase | ProviderId::Appwrite | ProviderId::Convex => {
            // First slice: portal Open only — no forced BaaS secret names.
        }
        ProviderId::Fly | ProviderId::Railway | ProviderId::Render | ProviderId::DigitalOcean
        | ProviderId::Heroku | ProviderId::Amplify | ProviderId::CloudRun | ProviderId::AzureStatic => {
            // First slice: portal Open only — deploy stays on vendor CLI/UI.
        }
        ProviderId::Gumroad | ProviderId::Lemon | ProviderId::Stripe | ProviderId::Paddle
        | ProviderId::Creem | ProviderId::Waffo => {
            // Catalog names come from graduate_commerce_catalog_hints — portal Open only here.
        }
    }

    let work = match id {
        ProviderId::Cloudflare => find_wrangler_dir(project).unwrap_or_else(|| project.to_path_buf()),
        _ => project.to_path_buf(),
    };

    let mut out = Vec::new();
    for (name, source) in names {
        let put_cli = put_cli_for(id, &name);
        let source_url = portal::entry_url_for_secret(&name, Some(id))
            .unwrap_or_else(|| portal::source_url_for_secret_name(&name));
        let once = portal::once_hint_for_secret_name(&name);
        let detail = match id {
            ProviderId::Github => {
                format!("{once} Then put on the deploy target (e.g. wrangler secret put GITHUB_TOKEN).")
            }
            ProviderId::Polar => {
                format!("{once} Then put on Cloudflare/Vercel with secrets put.")
            }
            ProviderId::Neon | ProviderId::Supabase | ProviderId::D1 | ProviderId::Turso => {
                format!(
                    "{once} Create/copy on the vendor dashboard, then put on Cloudflare/Vercel/Netlify. Source: {source}"
                )
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

/// Host CLI argv for Put — never includes the secret value.
pub fn put_cli_public(id: ProviderId, name: &str) -> Vec<String> {
    put_cli_for(id, name)
}

/// Project-aware Put argv (Cloudflare Pages vs Workers).
pub fn put_cli_public_for(project: &Path, id: ProviderId, name: &str) -> Vec<String> {
    match id {
        ProviderId::Cloudflare => cloudflare_put_argv(project, name, None),
        _ => put_cli_public(id, name),
    }
}

fn sanitize_host_project_name(raw: &str) -> String {
    let mut s: String = raw
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '-' || c == '_' {
                c.to_ascii_lowercase()
            } else {
                '-'
            }
        })
        .collect();
    while s.contains("--") {
        s = s.replace("--", "-");
    }
    let s = s.trim_matches('-').to_string();
    if s.is_empty() {
        "orbit-yard".into()
    } else {
        s.chars().take(58).collect()
    }
}

fn cloudflare_pages_secret_argv(
    project: &Path,
    name: &str,
    host_name: Option<&str>,
) -> Vec<String> {
    let pname = host_name
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(sanitize_host_project_name)
        .unwrap_or_else(|| {
            sanitize_host_project_name(
                project
                    .file_name()
                    .and_then(|s| s.to_str())
                    .unwrap_or("orbit-yard"),
            )
        });
    vec![
        "wrangler".into(),
        "pages".into(),
        "secret".into(),
        "put".into(),
        name.into(),
        "--project-name".into(),
        pname,
    ]
}

/// Prefer Workers `secret put` when wrangler config exists; else Pages secret.
fn cloudflare_put_argv(project: &Path, name: &str, host_name: Option<&str>) -> Vec<String> {
    if find_wrangler_file(project).is_some() {
        vec![
            "wrangler".into(),
            "secret".into(),
            "put".into(),
            name.into(),
        ]
    } else {
        cloudflare_pages_secret_argv(project, name, host_name)
    }
}

/// Strip CSI / OSC ANSI sequences so Desktop toasts stay readable.
fn strip_ansi(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut it = text.chars().peekable();
    while let Some(c) = it.next() {
        if c == '\u{1b}' {
            match it.peek().copied() {
                Some('[') => {
                    it.next();
                    for d in it.by_ref() {
                        if ('\u{40}'..='\u{7e}').contains(&d) {
                            break;
                        }
                    }
                }
                Some(']') => {
                    it.next();
                    while let Some(d) = it.next() {
                        if d == '\u{07}' {
                            break;
                        }
                        if d == '\u{1b}' && it.peek() == Some(&'\\') {
                            it.next();
                            break;
                        }
                    }
                }
                _ => {}
            }
            continue;
        }
        // Bare SGR remnants like `[31m` when ESC was lost in piping.
        if c == '[' {
            let mut probe = it.clone();
            let mut n = 0usize;
            let mut matched = false;
            while let Some(d) = probe.peek().copied() {
                if d.is_ascii_digit() || d == ';' {
                    probe.next();
                    n += 1;
                    continue;
                }
                if d == 'm' && n > 0 {
                    matched = true;
                }
                break;
            }
            if matched {
                for _ in 0..n {
                    it.next();
                }
                it.next(); // m
                continue;
            }
        }
        out.push(c);
    }
    out
}

fn scrub_cli_detail(text: &str, secret: &str) -> String {
    let mut t = strip_ansi(text).replace('\r', " ");
    if !secret.is_empty() && t.contains(secret) {
        t = t.replace(secret, "••••");
    }
    // Drop wrangler “Logs were written to …” noise from toast snippets.
    let one_line: String = t
        .lines()
        .map(str::trim)
        .filter(|l| !l.is_empty())
        .filter(|l| !l.to_ascii_lowercase().contains("logs were written"))
        .take(3)
        .collect::<Vec<_>>()
        .join(" · ");
    let cleaned: String = one_line.chars().take(180).collect();
    if crate::hostdeploy::is_missing_pages_project(&cleaned) {
        format!("{cleaned} [missing_project]")
    } else {
        cleaned
    }
}

/// Create Cloudflare Pages project when absent (hostdeploy create-if-missing parity).
fn ensure_cloudflare_pages_project(work: &Path, host_name: &str) -> Result<()> {
    let name = sanitize_host_project_name(host_name);
    let bin = ensure_put_cli_on_path(ProviderId::Cloudflare)?;
    eprintln!("import-env · ensuring Pages project `{name}`…");
    let mut child = Command::new(&bin)
        .args([
            "pages",
            "project",
            "create",
            &name,
            "--production-branch",
            "main",
        ])
        .current_dir(work)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .context("spawn wrangler pages project create")?;
    let (code, detail) = wait_child_timeout(&mut child, "wrangler pages project create", "")?;
    let blob = detail.unwrap_or_default();
    let ok = code == 0
        || {
            let l = blob.to_ascii_lowercase();
            l.contains("already exists") || l.contains("already created")
        };
    if ok {
        eprintln!("import-env · Pages project `{name}` ready");
        Ok(())
    } else {
        bail!(
            "could not create Pages project `{name}` — {} [missing_project]",
            if blob.is_empty() {
                format!("exit {code}")
            } else {
                blob
            }
        );
    }
}

fn put_cli_for(id: ProviderId, name: &str) -> Vec<String> {
    match id {
        // Default Workers shape for public recipe; project-aware path uses cloudflare_put_argv.
        ProviderId::Cloudflare => vec!["wrangler".into(), "secret".into(), "put".into(), name.into()],
        ProviderId::Vercel => vec!["vercel".into(), "env".into(), "add".into(), name.into()],
        ProviderId::Netlify => vec!["netlify".into(), "env:set".into(), name.into()],
        // Interactive: value on stdin (NAME=- / --stdin) — never on argv.
        ProviderId::Fly => vec![
            "fly".into(),
            "secrets".into(),
            "set".into(),
            format!("{name}=-"),
        ],
        ProviderId::Railway => vec![
            "railway".into(),
            "variable".into(),
            "set".into(),
            name.into(),
            "--stdin".into(),
        ],
        ProviderId::Github => vec!["gh".into(), "auth".into(), "login".into()],
        ProviderId::Polar
        | ProviderId::Neon
        | ProviderId::Supabase
        | ProviderId::D1
        | ProviderId::Turso
        | ProviderId::Container
        | ProviderId::Firebase
        | ProviderId::Appwrite
        | ProviderId::Convex
        | ProviderId::Render
        | ProviderId::DigitalOcean
        | ProviderId::Heroku
        | ProviderId::Amplify
        | ProviderId::CloudRun
        | ProviderId::AzureStatic
        | ProviderId::Gumroad
        | ProviderId::Lemon
        | ProviderId::Stripe
        | ProviderId::Paddle
        | ProviderId::Creem
        | ProviderId::Waffo => vec![
            "orbityard".into(),
            "portal".into(),
            "--provider".into(),
            id.as_str().into(),
            "--open".into(),
        ],
    }
}

/// Interactive put — value is entered in the child CLI, not captured by orbityard.
pub fn put_secret(project: &Path, provider: ProviderId, name: &str) -> Result<i32> {
    if name.is_empty() || name == "<NAME>" {
        bail!("pass a real secret name with --name");
    }
    if provider == ProviderId::Github {
        bail!("GitHub has no deploy secret put here — open the token page, then put on cloudflare/vercel/netlify");
    }
    if provider.is_commerce() {
        bail!(
            "{} has no secret put CLI — open the commerce dashboard, then `orbityard secrets put --provider cloudflare|vercel|netlify --name …`",
            provider.label()
        );
    }
    if provider.is_db() {
        bail!(
            "{} has no secret put CLI — open the vendor console, then `orbityard secrets put --provider cloudflare|vercel|netlify --name …`",
            provider.label()
        );
    }
    if provider == ProviderId::Container {
        bail!("Container has no secret put CLI — use docker login / gh auth, then push locally");
    }
    if provider.is_baas() {
        bail!(
            "{} has no secret put CLI — open the vendor console, then put keys on cloudflare|vercel|netlify (or the mobile app host)",
            provider.label()
        );
    }
    if matches!(
        provider,
        ProviderId::Render
            | ProviderId::DigitalOcean
            | ProviderId::Heroku
            | ProviderId::Amplify
            | ProviderId::CloudRun
            | ProviderId::AzureStatic
    ) {
        bail!(
            "{} has no secret put CLI — open the dashboard or use their CLI",
            provider.label()
        );
    }
    let project = fs::canonicalize(project).unwrap_or_else(|_| project.to_path_buf());
    let work = put_workdir(&project, provider);
    let args = put_cli_public_for(&work, provider, name);
    let bin_name = &args[0];
    let rest = &args[1..];
    let bin = if matches!(
        provider,
        ProviderId::Cloudflare
            | ProviderId::Vercel
            | ProviderId::Netlify
            | ProviderId::Fly
            | ProviderId::Railway
    ) {
        ensure_put_cli_on_path(provider)?
    } else {
        PathBuf::from(bin_name)
    };
    let status = Command::new(&bin)
        .args(rest)
        .current_dir(&work)
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()
        .with_context(|| format!("run {} in {}", bin.display(), work.display()))?;
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

/// Directory where `wrangler secret *` should run (often `apps/api`).
pub fn wrangler_workdir(project: &Path) -> PathBuf {
    find_wrangler_dir(project).unwrap_or_else(|| project.to_path_buf())
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
            "neon" | "supabase" | "d1" | "turso" | "container" | "firebase" | "appwrite"
            | "convex" | "fly" | "railway" | "render" | "digitalocean" | "heroku" | "amplify"
            | "cloudrun" | "azurestatic" | "gumroad" | "lemon" | "stripe" | "paddle"
            | "creem" | "waffo" => 5,
            "graduate" => 6,
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
            } else if n == "DATABASE_URL"
                || n.starts_with("NEON_")
                || n.starts_with("SUPABASE_")
                || n.starts_with("TURSO_")
                || n.starts_with("LIBSQL_")
            {
                2
            } else if n.starts_with("SIGNET_")
                || n.starts_with("WIN_CERT_")
                || n.starts_with("APPLE_API_")
                || n.starts_with("NOTARY_")
            {
                5
            } else if n.starts_with("GUMROAD_") || n.starts_with("LEMON") {
                6
            } else if n == "API_KEY_PEPPER" {
                4
            } else {
                3
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

// ── Bulk import from local env file → host CLI ─────────────────────────────

#[derive(Debug, Clone)]
struct EnvEntry {
    name: String,
    value: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportCandidate {
    pub name: String,
    pub empty: bool,
    pub denied: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportPutResult {
    pub name: String,
    pub ok: bool,
    pub code: i32,
    /// Scrubbed vendor CLI stderr snippet (never secret values).
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportSkip {
    pub name: String,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportEnvReport {
    pub schema: String,
    pub provider: String,
    pub file: String,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub packs: Vec<EnvPack>,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub candidates: Vec<ImportCandidate>,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub put: Vec<ImportPutResult>,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub skipped: Vec<ImportSkip>,
    pub notes: Vec<String>,
    pub dry_run: bool,
}

fn is_denied_env_name(name: &str) -> bool {
    let upper = name.to_ascii_uppercase();
    matches!(
        upper.as_str(),
        "PATH"
            | "HOME"
            | "USERPROFILE"
            | "USERNAME"
            | "USER"
            | "TMP"
            | "TEMP"
            | "PWD"
            | "SHELL"
            | "COMSPEC"
            | "PATHEXT"
            | "SYSTEMROOT"
            | "WINDIR"
            | "NUMBER_OF_PROCESSORS"
            | "OS"
            | "LANG"
            | "TERM"
    ) || upper.ends_with("_LOCAL")
}

/// Parse KEY=VALUE lines (supports optional `export `). Values stay in memory only.
pub fn parse_env_file_entries(path: &Path) -> Result<Vec<(String, String)>> {
    let s = path
        .to_string_lossy()
        .to_ascii_lowercase();
    if s.starts_with("http://") || s.starts_with("https://") {
        bail!("refuse network URLs — pick a local .env / .dev.vars file");
    }
    let raw = fs::read_to_string(path)
        .with_context(|| format!("read env file {}", path.display()))?;
    let mut out = Vec::new();
    let mut seen = std::collections::HashSet::new();
    for line in raw.lines() {
        let mut line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if let Some(rest) = line.strip_prefix("export ") {
            line = rest.trim();
        }
        let Some((k, v)) = line.split_once('=') else {
            continue;
        };
        let key = k.trim();
        if key.is_empty() || !seen.insert(key.to_string()) {
            continue;
        }
        let mut val = v.trim();
        if (val.starts_with('"') && val.ends_with('"') && val.len() >= 2)
            || (val.starts_with('\'') && val.ends_with('\'') && val.len() >= 2)
        {
            val = &val[1..val.len() - 1];
        }
        out.push((key.to_string(), val.to_string()));
    }
    Ok(out)
}

/// Tier A (CF/Vercel/Netlify) + Tier B (Fly/Railway) Put / import-env hosts.
fn ensure_put_provider(provider: ProviderId) -> Result<()> {
    if !matches!(
        provider,
        ProviderId::Cloudflare
            | ProviderId::Vercel
            | ProviderId::Netlify
            | ProviderId::Fly
            | ProviderId::Railway
    ) {
        bail!(
            "{} has no import-env put CLI — use Cloudflare / Vercel / Netlify / Fly / Railway",
            provider.label()
        );
    }
    Ok(())
}

const ENV_FILE_NAMES: &[&str] = &[".dev.vars", ".env", ".env.local", ".env.production"];

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvPack {
    pub id: String,
    pub label: String,
    pub kind: String,
    pub rel_dir: String,
    pub file: String,
    pub host_hint: Option<String>,
    pub key_count: usize,
    pub nonempty_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvPacksReport {
    pub schema: String,
    pub project: String,
    pub packs: Vec<EnvPack>,
    pub notes: Vec<String>,
}

fn rel_posix(project: &Path, p: &Path) -> String {
    p.strip_prefix(project)
        .map(|x| x.to_string_lossy().replace('\\', "/"))
        .unwrap_or_else(|_| p.display().to_string().replace('\\', "/"))
}

fn host_hint_for_dir(dir: &Path) -> Option<String> {
    if dir.join("wrangler.toml").is_file()
        || dir.join("wrangler.json").is_file()
        || dir.join("wrangler.jsonc").is_file()
    {
        return Some("cloudflare".into());
    }
    if dir.join("netlify.toml").is_file() || dir.join(".netlify").is_dir() {
        return Some("netlify".into());
    }
    if dir.join("vercel.json").is_file() || dir.join(".vercel").is_dir() {
        return Some("vercel".into());
    }
    if dir.join("fly.toml").is_file() {
        return Some("fly".into());
    }
    if dir.join("railway.toml").is_file()
        || dir.join("railway.json").is_file()
        || dir.join(".railway").is_dir()
    {
        return Some("railway".into());
    }
    None
}

fn find_marker_dir(project: &Path, markers: &[&str]) -> Option<PathBuf> {
    for name in markers {
        if project.join(name).is_file() || project.join(name).is_dir() {
            return Some(project.to_path_buf());
        }
    }
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
            for name in markers {
                if p.join(name).is_file() || p.join(name).is_dir() {
                    return Some(p);
                }
            }
            // one more level (apps/*)
            if let Ok(nested) = fs::read_dir(&p) {
                for ent2 in nested.flatten() {
                    let p2 = ent2.path();
                    if !p2.is_dir() {
                        continue;
                    }
                    for name in markers {
                        if p2.join(name).is_file() || p2.join(name).is_dir() {
                            return Some(p2);
                        }
                    }
                }
            }
        }
    }
    None
}

fn put_workdir(project: &Path, provider: ProviderId) -> PathBuf {
    match provider {
        ProviderId::Cloudflare => find_wrangler_dir(project).unwrap_or_else(|| project.to_path_buf()),
        ProviderId::Fly => find_marker_dir(project, &["fly.toml"]).unwrap_or_else(|| project.to_path_buf()),
        ProviderId::Railway => find_marker_dir(
            project,
            &["railway.toml", "railway.json", ".railway"],
        )
        .unwrap_or_else(|| project.to_path_buf()),
        _ => project.to_path_buf(),
    }
}

fn count_env_keys(path: &Path) -> (usize, usize) {
    let Ok(entries) = parse_env_file_entries(path) else {
        return (0, 0);
    };
    let key_count = entries.len();
    let nonempty_count = entries
        .iter()
        .filter(|(n, v)| {
            !is_denied_env_name(n)
                && !v.is_empty()
                && !v.eq_ignore_ascii_case("changeme")
                && v != "…"
        })
        .count();
    (key_count, nonempty_count)
}

fn kind_label(kind: &crate::scopes::ScopeKind) -> (&'static str, &'static str) {
    use crate::scopes::ScopeKind;
    match kind {
        ScopeKind::Web => ("web", "Website"),
        ScopeKind::Api => ("api", "API"),
        ScopeKind::Desktop => ("desktop", "Desktop"),
        ScopeKind::Docs => ("docs", "Docs"),
        ScopeKind::Mobile => ("mobile", "Mobile"),
        ScopeKind::Container => ("container", "Container"),
        ScopeKind::Root => ("root", "Project"),
    }
}

/// All importable env files under scopes (monorepo-aware). Skips `.env.example`.
pub fn list_env_packs(project: &Path) -> Result<EnvPacksReport> {
    let project = fs::canonicalize(project).unwrap_or_else(|_| project.to_path_buf());
    let scopes = crate::scopes::detect(&project);
    let mut packs = Vec::new();
    let mut seen = std::collections::HashSet::new();

    let mut roots: Vec<(PathBuf, String, String, String)> = Vec::new();
    for s in &scopes {
        let dir = PathBuf::from(&s.root);
        let (kind, fallback_label) = kind_label(&s.kind);
        let label = if s.label.is_empty() {
            fallback_label.to_string()
        } else {
            s.label.clone()
        };
        roots.push((dir, s.relative.clone(), kind.to_string(), label));
    }
    // Always include project root if scopes somehow omitted it.
    if !roots.iter().any(|(d, ..)| d == &project) {
        roots.insert(
            0,
            (
                project.clone(),
                ".".into(),
                "root".into(),
                "Project".into(),
            ),
        );
    }
    if let Some(w) = find_wrangler_dir(&project) {
        if !roots.iter().any(|(d, ..)| d == &w) {
            roots.push((
                w.clone(),
                rel_posix(&project, &w),
                "api".into(),
                "API (wrangler)".into(),
            ));
        }
    }
    if let Some(w) = find_marker_dir(&project, &["fly.toml"]) {
        if !roots.iter().any(|(d, ..)| d == &w) {
            roots.push((
                w.clone(),
                rel_posix(&project, &w),
                "api".into(),
                "API (Fly)".into(),
            ));
        }
    }
    if let Some(w) = find_marker_dir(&project, &["railway.toml", "railway.json", ".railway"]) {
        if !roots.iter().any(|(d, ..)| d == &w) {
            roots.push((
                w.clone(),
                rel_posix(&project, &w),
                "api".into(),
                "API (Railway)".into(),
            ));
        }
    }

    for (dir, rel_dir, kind, label) in roots {
        let hint = host_hint_for_dir(&dir);
        for name in ENV_FILE_NAMES {
            let p = dir.join(name);
            if !p.is_file() {
                continue;
            }
            let canon = fs::canonicalize(&p).unwrap_or(p.clone());
            if !seen.insert(canon.clone()) {
                continue;
            }
            let file_rel = rel_posix(&project, &canon);
            let (key_count, nonempty_count) = count_env_keys(&canon);
            packs.push(EnvPack {
                id: file_rel.clone(),
                label: label.clone(),
                kind: kind.clone(),
                rel_dir: if rel_dir.is_empty() {
                    ".".into()
                } else {
                    rel_dir.replace('\\', "/")
                },
                file: file_rel,
                host_hint: hint.clone(),
                key_count,
                nonempty_count,
            });
        }
    }

    packs.sort_by(|a, b| a.file.cmp(&b.file));
    Ok(EnvPacksReport {
        schema: "orbit-yard/secrets-env-packs/v1".into(),
        project: project.display().to_string(),
        packs,
        notes: vec![
            "Pick one ship unit (pack) — same idea as Targets Deploy.".into(),
            "Counts only; values never appear in this report.".into(),
            "Skip .env.example (template). Use --pack <file> or --file.".into(),
        ],
    })
}

/// Prefer `.dev.vars` then `.env` / `.env.local` under project + scopes + wrangler.
#[allow(dead_code)] // public helper for CLI/Desktop callers / future MCP
pub fn discover_env_import_files(project: &Path, _provider: ProviderId) -> Vec<PathBuf> {
    let project = fs::canonicalize(project).unwrap_or_else(|_| project.to_path_buf());
    list_env_packs(&project)
        .map(|r| {
            r.packs
                .into_iter()
                .map(|p| {
                    let abs = project.join(&p.file);
                    fs::canonicalize(&abs).unwrap_or(abs)
                })
                .collect()
        })
        .unwrap_or_default()
}

fn resolve_import_file(
    project: &Path,
    provider: ProviderId,
    file: Option<&Path>,
    pack: Option<&str>,
) -> Result<PathBuf> {
    if let Some(f) = file {
        let s = f.to_string_lossy();
        if s.starts_with("http://") || s.starts_with("https://") {
            bail!("refuse network URLs — pick a local file");
        }
        let p = if f.is_absolute() {
            f.to_path_buf()
        } else {
            project.join(f)
        };
        if !p.is_file() {
            bail!("env file not found: {}", p.display());
        }
        return Ok(fs::canonicalize(&p).unwrap_or(p));
    }
    if let Some(pack_raw) = pack.map(str::trim).filter(|s| !s.is_empty()) {
        let pack_id = pack_raw.replace('\\', "/");
        let packs = list_env_packs(project)?;
        let norm = |s: &str| s.replace('\\', "/");
        // Exact file id, or directory pack → preferred file under that dir.
        if let Some(hit) = packs.packs.iter().find(|p| {
            norm(&p.id) == pack_id
                || norm(&p.file) == pack_id
                || norm(&p.file).ends_with(&pack_id)
                || pack_id.ends_with(&norm(&p.file))
        }) {
            let abs = project.join(&hit.file);
            return Ok(fs::canonicalize(&abs).unwrap_or(abs));
        }
        // Treat --pack as a relative file path (Desktop may pass apps/website/.env
        // while cwd is already apps/website — also try basename / local join).
        for candidate in [
            project.join(&pack_id),
            project.join(Path::new(&pack_id).file_name().unwrap_or_default()),
        ] {
            if candidate.is_file() {
                return Ok(fs::canonicalize(&candidate).unwrap_or(candidate));
            }
        }
        let under: Vec<_> = packs
            .packs
            .iter()
            .filter(|p| {
                norm(&p.rel_dir) == pack_id || norm(&p.file).starts_with(&format!("{pack_id}/"))
            })
            .collect();
        if let Some(hit) = under
            .iter()
            .find(|p| p.host_hint.as_deref() == Some(provider.as_str()))
            .or_else(|| under.first())
        {
            let abs = project.join(&hit.file);
            return Ok(fs::canonicalize(&abs).unwrap_or(abs));
        }
        bail!("unknown --pack {pack_id} — run --list-packs");
    }
    // Prefer pack matching provider host_hint, else first.
    let packs = list_env_packs(project)?;
    if packs.packs.is_empty() {
        bail!("no .dev.vars / .env / .env.local found — pass --file/--pack or add keys in the repo");
    }
    let preferred = packs
        .packs
        .iter()
        .find(|p| p.host_hint.as_deref() == Some(provider.as_str()))
        .or_else(|| packs.packs.first())
        .unwrap();
    let abs = project.join(&preferred.file);
    Ok(fs::canonicalize(&abs).unwrap_or(abs))
}

fn put_cli_bin(provider: ProviderId) -> Result<&'static str> {
    ensure_put_provider(provider)?;
    Ok(match provider {
        ProviderId::Cloudflare => "wrangler",
        ProviderId::Vercel => "vercel",
        ProviderId::Netlify => "netlify",
        ProviderId::Fly => "fly",
        ProviderId::Railway => "railway",
        _ => unreachable!(),
    })
}

/// Fail fast with `[missing_cli]` when the host CLI is not on PATH (hostdeploy parity).
pub fn ensure_put_cli_on_path(provider: ProviderId) -> Result<PathBuf> {
    let name = put_cli_bin(provider)?;
    crate::hostdeploy::resolve_auth_cli(name).map_err(|e| {
        anyhow::anyhow!("{e} [missing_cli]")
    })
}

const PUT_CHILD_TIMEOUT_SECS: u64 = 45;

fn take_child_stderr(child: &mut std::process::Child) -> String {
    let mut buf = String::new();
    if let Some(mut err) = child.stderr.take() {
        let _ = err.read_to_string(&mut buf);
    }
    buf
}

fn wait_child_timeout(
    child: &mut std::process::Child,
    label: &str,
    secret: &str,
) -> Result<(i32, Option<String>)> {
    let started = Instant::now();
    let timeout = Duration::from_secs(PUT_CHILD_TIMEOUT_SECS);
    loop {
        match child.try_wait() {
            Ok(Some(s)) => {
                let detail = scrub_cli_detail(&take_child_stderr(child), secret);
                return Ok((
                    s.code().unwrap_or(-1),
                    if detail.is_empty() {
                        None
                    } else {
                        Some(detail)
                    },
                ));
            }
            Ok(None) if started.elapsed() > timeout => {
                let _ = child.kill();
                let _ = child.wait();
                let detail = scrub_cli_detail(&take_child_stderr(child), secret);
                let msg = if detail.is_empty() {
                    format!(
                        "{label} timed out after {PUT_CHILD_TIMEOUT_SECS}s — Login CLI / link the site, then retry"
                    )
                } else {
                    format!(
                        "{label} timed out after {PUT_CHILD_TIMEOUT_SECS}s — {detail}"
                    )
                };
                bail!("{msg}");
            }
            Ok(None) => std::thread::sleep(Duration::from_millis(40)),
            Err(e) => bail!("wait {label}: {e}"),
        }
    }
}

fn write_value_stdin(child: &mut std::process::Child, value: &str) -> Result<()> {
    if let Some(mut stdin) = child.stdin.take() {
        stdin.write_all(value.as_bytes())?;
        if !value.ends_with('\n') {
            stdin.write_all(b"\n")?;
        }
    }
    Ok(())
}

/// Returns (exit code, scrubbed stderr snippet). Never returns secret values.
fn put_secret_value(
    project: &Path,
    provider: ProviderId,
    name: &str,
    value: &str,
    host_name: Option<&str>,
) -> Result<(i32, Option<String>)> {
    ensure_put_provider(provider)?;
    let bin = ensure_put_cli_on_path(provider)?;
    let project = fs::canonicalize(project).unwrap_or_else(|_| project.to_path_buf());
    let work = put_workdir(&project, provider);

    match provider {
        ProviderId::Cloudflare => {
            let argv = cloudflare_put_argv(&work, name, host_name);
            let rest = &argv[1..];
            let label = if rest.first().map(|s| s.as_str()) == Some("pages") {
                "wrangler pages secret put"
            } else {
                "wrangler secret put"
            };
            let mut child = Command::new(&bin)
                .args(rest)
                .current_dir(&work)
                .stdin(Stdio::piped())
                .stdout(Stdio::null())
                .stderr(Stdio::piped())
                .spawn()
                .with_context(|| format!("spawn {label}"))?;
            write_value_stdin(&mut child, value)?;
            wait_child_timeout(&mut child, label, value)
        }
        ProviderId::Vercel => {
            let mut child = Command::new(&bin)
                .args(["env", "add", name, "production"])
                .current_dir(&work)
                .stdin(Stdio::piped())
                .stdout(Stdio::null())
                .stderr(Stdio::piped())
                .spawn()
                .context("spawn vercel env add")?;
            write_value_stdin(&mut child, value)?;
            wait_child_timeout(&mut child, "vercel env add", value)
        }
        ProviderId::Netlify => {
            // env:set takes value on argv — use env:import of a one-key temp file instead.
            let tmp = std::env::temp_dir().join(format!(
                "orbityard-env-import-{}-{}.env",
                std::process::id(),
                name
            ));
            let line = format!("{name}={value}\n");
            fs::write(&tmp, &line).context("write ephemeral env import")?;
            let mut child = Command::new(&bin)
                .args(["env:import", tmp.to_str().unwrap_or_default()])
                .current_dir(&work)
                .stdin(Stdio::null())
                .stdout(Stdio::null())
                .stderr(Stdio::piped())
                .spawn()
                .context("spawn netlify env:import")?;
            let out = wait_child_timeout(&mut child, "netlify env:import", value);
            let _ = fs::write(&tmp, "x".repeat(line.len().max(1)));
            let _ = fs::remove_file(&tmp);
            out
        }
        ProviderId::Fly => {
            // --stage: bulk import without restarting Machines per key.
            let mut child = Command::new(&bin)
                .args(["secrets", "set", &format!("{name}=-"), "--stage"])
                .current_dir(&work)
                .stdin(Stdio::piped())
                .stdout(Stdio::null())
                .stderr(Stdio::piped())
                .spawn()
                .context("spawn fly secrets set")?;
            write_value_stdin(&mut child, value)?;
            wait_child_timeout(&mut child, "fly secrets set", value)
        }
        ProviderId::Railway => {
            let mut child = Command::new(&bin)
                .args(["variable", "set", name, "--stdin", "--skip-deploys"])
                .current_dir(&work)
                .stdin(Stdio::piped())
                .stdout(Stdio::null())
                .stderr(Stdio::piped())
                .spawn()
                .context("spawn railway variable set")?;
            write_value_stdin(&mut child, value)?;
            wait_child_timeout(&mut child, "railway variable set", value)
        }
        _ => bail!("unsupported provider"),
    }
}

/// Import selected (or all eligible) keys from a local env file to the host CLI.
/// Never writes values under `.ship/`. Dry-run lists names only.
pub fn import_env_to_host(
    project: &Path,
    provider: ProviderId,
    file: Option<&Path>,
    pack: Option<&str>,
    only_names: &[String],
    dry_run: bool,
    yes: bool,
    host_name: Option<&str>,
) -> Result<ImportEnvReport> {
    ensure_put_provider(provider)?;
    let project = fs::canonicalize(project).unwrap_or_else(|_| project.to_path_buf());
    let packs = list_env_packs(&project)?.packs;
    let file_path = resolve_import_file(&project, provider, file, pack)?;
    let entries = parse_env_file_entries(&file_path)?;
    let filter: Option<std::collections::HashSet<String>> = if only_names.is_empty() {
        None
    } else {
        Some(only_names.iter().cloned().collect())
    };

    let mut candidates = Vec::new();
    let mut skipped = Vec::new();
    let mut to_put: Vec<EnvEntry> = Vec::new();

    for (name, value) in entries {
        if let Some(ref f) = filter {
            if !f.contains(&name) {
                skipped.push(ImportSkip {
                    name: name.clone(),
                    reason: "not selected".into(),
                });
                continue;
            }
        }
        let empty = value.is_empty()
            || value.eq_ignore_ascii_case("changeme")
            || value == "…";
        let denied = is_denied_env_name(&name);
        candidates.push(ImportCandidate {
            name: name.clone(),
            empty,
            denied,
        });
        if denied {
            skipped.push(ImportSkip {
                name,
                reason: "denied local/shell name".into(),
            });
            continue;
        }
        if empty {
            skipped.push(ImportSkip {
                name,
                reason: "empty value".into(),
            });
            continue;
        }
        to_put.push(EnvEntry { name, value });
    }

    let mut notes = vec![
        "Values stay on this PC — pushed only via the host CLI (wrangler / vercel / netlify / fly / railway).".into(),
        "Orbit Yard does not save secret values in .ship/ or print them.".into(),
        "Providers still bill hosting — Studio complements their CLI, it does not replace their dashboard.".into(),
        "Monorepo: pick one env pack (ship unit) — use --list-packs / --pack.".into(),
    ];
    if let Some(hn) = host_name.map(str::trim).filter(|s| !s.is_empty()) {
        notes.push(format!(
            "Host project/site name for Pages/site-scoped put: {}.",
            sanitize_host_project_name(hn)
        ));
    }

    let file_display = rel_posix(&project, &file_path);

    if dry_run {
        return Ok(ImportEnvReport {
            schema: "orbit-yard/secrets-import/v1".into(),
            provider: provider.as_str().into(),
            file: file_display,
            packs,
            candidates,
            put: vec![],
            skipped,
            notes,
            dry_run: true,
        });
    }

    if !yes {
        bail!(
            "refusing import without --yes (confirm names in Desktop or pass --yes after dry-run). Candidates: {}",
            to_put
                .iter()
                .map(|e| e.name.as_str())
                .collect::<Vec<_>>()
                .join(", ")
        );
    }

    // Preflight once — clearer than spawn "program not found" mid-loop.
    ensure_put_cli_on_path(provider)?;

    // Put from the env file's directory as work when possible (Netlify/Vercel link).
    let work_project = file_path
        .parent()
        .filter(|p| p.starts_with(&project))
        .unwrap_or(&project);

    // Cloudflare Pages: create-if-missing before secrets (Deploy already does this).
    if provider == ProviderId::Cloudflare
        && find_wrangler_file(work_project).is_none()
        && !to_put.is_empty()
    {
        let pname = host_name
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .map(sanitize_host_project_name)
            .unwrap_or_else(|| {
                sanitize_host_project_name(
                    work_project
                        .file_name()
                        .and_then(|s| s.to_str())
                        .unwrap_or("orbit-yard"),
                )
            });
        ensure_cloudflare_pages_project(work_project, &pname)?;
        notes.push(format!(
            "Ensured Cloudflare Pages project `{pname}` before put (create-if-missing)."
        ));
    }

    // Prove .ship/ is not used as a value store: never open it for write here.
    let mut put = Vec::new();
    let mut ensured_after_miss = false;
    for entry in to_put {
        let (mut code, mut error) =
            put_secret_value(work_project, provider, &entry.name, &entry.value, host_name)?;
        // One create+retry if put still reports missing project (race / ensure skipped).
        if code != 0
            && !ensured_after_miss
            && provider == ProviderId::Cloudflare
            && find_wrangler_file(work_project).is_none()
            && error
                .as_deref()
                .map(crate::hostdeploy::is_missing_pages_project)
                .unwrap_or(false)
        {
            let pname = host_name
                .map(str::trim)
                .filter(|s| !s.is_empty())
                .map(sanitize_host_project_name)
                .unwrap_or_else(|| {
                    sanitize_host_project_name(
                        work_project
                            .file_name()
                            .and_then(|s| s.to_str())
                            .unwrap_or("orbit-yard"),
                    )
                });
            if ensure_cloudflare_pages_project(work_project, &pname).is_ok() {
                ensured_after_miss = true;
                let retry =
                    put_secret_value(work_project, provider, &entry.name, &entry.value, host_name)?;
                code = retry.0;
                error = retry.1;
            }
        }
        if code == 0 {
            eprintln!("PUT {} ok", entry.name);
        } else if let Some(ref detail) = error {
            eprintln!("PUT {} fail — {}", entry.name, detail);
        } else {
            eprintln!("PUT {} fail", entry.name);
        }
        put.push(ImportPutResult {
            name: entry.name,
            ok: code == 0,
            code,
            error,
        });
    }

    Ok(ImportEnvReport {
        schema: "orbit-yard/secrets-import/v1".into(),
        provider: provider.as_str().into(),
        file: file_display,
        packs: vec![],
        candidates,
        put,
        skipped,
        notes,
        dry_run: false,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tempfile_dir() -> PathBuf {
        let dir = std::env::temp_dir().join(format!(
            "orbityard-secrets-{}-{}",
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
    fn empty_detect_does_not_flood_all_secret_catalogs() {
        let dir = tempfile_dir();
        let plan = plan_for(&dir, None).unwrap();
        assert!(
            plan.hints.is_empty(),
            "empty project must not dump Neon/Supabase/… catalogs, got {} hints",
            plan.hints.len()
        );
        assert!(plan
            .notes
            .iter()
            .any(|n| n.contains("No host/commerce providers detected")));
    }

    #[test]
    fn public_paddle_env_yields_paddle_hints() {
        let dir = tempfile_dir();
        fs::write(
            dir.join(".env"),
            "PUBLIC_PADDLE_CLIENT_TOKEN=\nPUBLIC_PADDLE_PRICE_ID=\n",
        )
        .unwrap();
        let plan = plan_for(&dir, None).unwrap();
        assert!(plan.hints.iter().any(|h| h.provider == "paddle"));
        assert!(plan
            .hints
            .iter()
            .any(|h| h.name == "PUBLIC_PADDLE_CLIENT_TOKEN"));
        assert!(!plan.hints.iter().any(|h| h.provider == "neon"));
    }

    #[test]
    fn vercel_empty_env_entry_urls_not_cloudflare() {
        let dir = tempfile_dir();
        fs::write(dir.join("vercel.json"), "{}\n").unwrap();
        fs::write(
            dir.join(".env"),
            "ANTHROPIC_API_KEY=\nCRON_SECRET=\nRESEND_API_KEY=\n",
        )
        .unwrap();
        let plan = plan_for(&dir, Some(ProviderId::Vercel)).unwrap();
        let anthropic = plan
            .hints
            .iter()
            .find(|h| h.name == "ANTHROPIC_API_KEY")
            .expect("anthropic hint");
        assert!(
            anthropic
                .entry_url
                .as_deref()
                .unwrap_or("")
                .contains("anthropic.com"),
            "got {:?}",
            anthropic.entry_url
        );
        let cron = plan
            .hints
            .iter()
            .find(|h| h.name == "CRON_SECRET")
            .expect("cron hint");
        let cron_url = cron.entry_url.as_deref().unwrap_or("");
        assert!(
            cron_url.contains("vercel.com"),
            "CRON_SECRET should open Vercel put destination, got {cron_url}"
        );
        assert!(!cron_url.contains("cloudflare"));
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

    #[test]
    fn graduate_commerce_catalog_hints_when_opted_in() {
        let dir = tempfile_dir();
        fs::create_dir_all(dir.join(".ship")).unwrap();
        fs::write(
            dir.join(".ship/markets.json"),
            r#"["graduate","gumroad","lemon"]"#,
        )
        .unwrap();
        let plan = plan_for(&dir, None).unwrap();
        let names: Vec<_> = plan.hints.iter().map(|h| h.name.as_str()).collect();
        assert!(names.contains(&"SIGNET_AZURE_CLIENT_ID"));
        assert!(names.contains(&"GUMROAD_CHECKOUT_URL"));
        assert!(names.contains(&"LEMON_API_KEY"));
        assert!(plan.hints.iter().any(|h| h.provider == "graduate"));
        assert!(plan.hints.iter().any(|h| h.provider == "gumroad"));
        assert!(plan.hints.iter().any(|h| h.provider == "lemon"));
    }

    #[test]
    fn parse_env_file_entries_export_and_quotes() {
        let dir = tempfile_dir();
        let p = dir.join(".env");
        fs::write(
            &p,
            "export FOO=bar\n# comment\nBAZ=\"qux\"\nEMPTY=\nPATH=/usr/bin\nFOO=ignored\n",
        )
        .unwrap();
        let entries = parse_env_file_entries(&p).unwrap();
        assert_eq!(entries.len(), 4);
        assert_eq!(entries[0], ("FOO".into(), "bar".into()));
        assert_eq!(entries[1], ("BAZ".into(), "qux".into()));
        assert_eq!(entries[2], ("EMPTY".into(), "".into()));
        assert_eq!(entries[3], ("PATH".into(), "/usr/bin".into()));
    }

    #[test]
    fn import_env_dry_run_lists_names_never_values_or_ship() {
        let dir = tempfile_dir();
        let ship = dir.join(".ship");
        fs::create_dir_all(&ship).unwrap();
        fs::write(ship.join("marker"), "keep").unwrap();
        let env = dir.join(".dev.vars");
        fs::write(&env, "API_TOKEN=super-secret-value\nPATH=/nope\nEMPTY=\n").unwrap();
        let report = import_env_to_host(
            &dir,
            ProviderId::Cloudflare,
            Some(&env),
            None,
            &[],
            true,
            false,
            None,
        )
        .unwrap();
        assert!(report.dry_run);
        let json = serde_json::to_string(&report).unwrap();
        assert!(!json.contains("super-secret-value"));
        assert!(report.candidates.iter().any(|c| c.name == "API_TOKEN" && !c.empty));
        assert!(report.candidates.iter().any(|c| c.name == "PATH" && c.denied));
        assert!(report.skipped.iter().any(|s| s.name == "PATH"));
        assert!(report.skipped.iter().any(|s| s.name == "EMPTY"));
        assert_eq!(fs::read_to_string(ship.join("marker")).unwrap(), "keep");
        // No new files under .ship from import
        let names: Vec<_> = fs::read_dir(&ship)
            .unwrap()
            .filter_map(|e| e.ok().map(|e| e.file_name()))
            .collect();
        assert_eq!(names.len(), 1);
    }

    #[test]
    fn import_env_refuses_without_yes() {
        let dir = tempfile_dir();
        let env = dir.join(".env");
        fs::write(&env, "API_TOKEN=x\n").unwrap();
        let err =
            import_env_to_host(
                &dir,
                ProviderId::Vercel,
                Some(&env),
                None,
                &[],
                false,
                false,
                None,
            )
                .unwrap_err();
        assert!(err.to_string().contains("--yes"), "{err}");
    }

    #[test]
    fn import_env_preflight_missing_cli_tag() {
        // Prefer a CLI that is often absent in CI/dev so we can assert the tag.
        let provider = if crate::hostdeploy::resolve_auth_cli("netlify").is_err() {
            ProviderId::Netlify
        } else if crate::hostdeploy::resolve_auth_cli("vercel").is_err() {
            ProviderId::Vercel
        } else if crate::hostdeploy::resolve_auth_cli("wrangler").is_err() {
            ProviderId::Cloudflare
        } else {
            return; // all Tier A CLIs present — cannot assert missing path here
        };
        let dir = tempfile_dir();
        let env = dir.join(".env");
        fs::write(&env, "API_TOKEN=x\n").unwrap();
        let err = import_env_to_host(
            &dir,
            provider,
            Some(&env),
            None,
            &[],
            false,
            true,
            None,
        )
            .unwrap_err();
        let s = format!("{err:#}");
        assert!(s.contains("[missing_cli]"), "{s}");
        assert!(s.contains("not on PATH"), "{s}");
    }

    #[cfg(windows)]
    #[test]
    fn put_cli_prefers_windows_cmd_shim() {
        let Ok(p) = ensure_put_cli_on_path(ProviderId::Netlify) else {
            return;
        };
        let s = p.to_string_lossy().to_ascii_lowercase();
        assert!(
            s.ends_with("netlify.cmd") || s.ends_with("netlify.exe"),
            "expected .cmd/.exe, got {s}"
        );
    }

    #[test]
    fn import_env_refuses_http_file() {
        let dir = tempfile_dir();
        let err = import_env_to_host(
            &dir,
            ProviderId::Netlify,
            Some(Path::new("https://evil.example/x.env")),
            None,
            &[],
            true,
            false,
            None,
        )
        .unwrap_err();
        assert!(err.to_string().contains("refuse"), "{err}");
    }

    #[test]
    fn list_env_packs_monorepo_two_apps() {
        let dir = tempfile_dir();
        fs::create_dir_all(dir.join("apps/website")).unwrap();
        fs::create_dir_all(dir.join("apps/api")).unwrap();
        fs::write(dir.join("apps/website/netlify.toml"), "[build]\n").unwrap();
        fs::write(dir.join("apps/website/.env"), "PUBLIC_URL=https://x.test\nSECRET=shh\n").unwrap();
        fs::write(dir.join("apps/api/wrangler.toml"), "name = \"api\"\n").unwrap();
        fs::write(dir.join("apps/api/.dev.vars"), "API_TOKEN=tok\n").unwrap();
        let report = list_env_packs(&dir).unwrap();
        assert!(
            report.packs.len() >= 2,
            "expected ≥2 packs, got {:?}",
            report.packs
        );
        let json = serde_json::to_string(&report).unwrap();
        assert!(!json.contains("shh"));
        assert!(!json.contains("tok"));
        assert!(report.packs.iter().any(|p| p.file.contains("website") && p.file.ends_with(".env")));
        assert!(report
            .packs
            .iter()
            .any(|p| p.file.contains("api") && p.file.contains(".dev.vars")));
        let dry = import_env_to_host(
            &dir,
            ProviderId::Netlify,
            None,
            Some("apps/website"),
            &[],
            true,
            false,
            None,
        )
        .unwrap();
        assert!(dry.file.replace('\\', "/").contains("apps/website"));
        assert!(dry.candidates.iter().any(|c| c.name == "PUBLIC_URL"));
        assert!(!serde_json::to_string(&dry).unwrap().contains("shh"));

        // Pack id as full relative file (Desktop Deploy panel).
        let by_file = import_env_to_host(
            &dir,
            ProviderId::Netlify,
            None,
            Some("apps/website/.env"),
            &[],
            true,
            false,
            None,
        )
        .unwrap();
        assert!(by_file.file.replace('\\', "/").ends_with("apps/website/.env"));

        // Simulate Deploy focus cwd = apps/website with monorepo-relative pack id.
        let website = dir.join("apps/website");
        let nested = import_env_to_host(
            &website,
            ProviderId::Netlify,
            None,
            Some("apps/website/.env"),
            &[],
            true,
            false,
            None,
        )
        .unwrap();
        assert!(
            nested.candidates.iter().any(|c| c.name == "PUBLIC_URL"),
            "basename fallback should open local .env when cwd is the app folder"
        );
    }

    #[test]
    fn tier_b_fly_pack_hint_and_dry_run() {
        let dir = tempfile_dir();
        fs::create_dir_all(dir.join("apps/api")).unwrap();
        fs::write(dir.join("apps/api/fly.toml"), "app = \"demo\"\n").unwrap();
        fs::write(dir.join("apps/api/.env"), "API_TOKEN=secret-fly\n").unwrap();
        let packs = list_env_packs(&dir).unwrap();
        assert!(
            packs
                .packs
                .iter()
                .any(|p| p.host_hint.as_deref() == Some("fly") && p.file.contains("apps/api")),
            "expected fly host_hint, got {:?}",
            packs.packs
        );
        let dry = import_env_to_host(
            &dir,
            ProviderId::Fly,
            None,
            Some("apps/api"),
            &[],
            true,
            false,
            None,
        )
        .unwrap();
        assert!(dry.candidates.iter().any(|c| c.name == "API_TOKEN"));
        assert!(!serde_json::to_string(&dry).unwrap().contains("secret-fly"));
    }

    #[test]
    fn tier_b_railway_pack_hint_and_dry_run() {
        let dir = tempfile_dir();
        fs::create_dir_all(dir.join("apps/worker")).unwrap();
        fs::write(dir.join("apps/worker/railway.toml"), "[build]\n").unwrap();
        fs::write(dir.join("apps/worker/.env"), "DB_URL=secret-rw\n").unwrap();
        let packs = list_env_packs(&dir).unwrap();
        assert!(
            packs.packs.iter().any(|p| p.host_hint.as_deref() == Some("railway")),
            "expected railway host_hint, got {:?}",
            packs.packs
        );
        let dry = import_env_to_host(
            &dir,
            ProviderId::Railway,
            Some(&dir.join("apps/worker/.env")),
            None,
            &[],
            true,
            false,
            None,
        )
        .unwrap();
        assert!(dry.candidates.iter().any(|c| c.name == "DB_URL"));
        assert!(!serde_json::to_string(&dry).unwrap().contains("secret-rw"));
    }

    #[test]
    fn ensure_put_rejects_non_tier_hosts() {
        let err = ensure_put_provider(ProviderId::Render).unwrap_err();
        assert!(err.to_string().contains("no import-env"), "{err}");
    }

    #[test]
    fn cloudflare_put_argv_pages_vs_workers() {
        let dir = tempfile_dir();
        fs::create_dir_all(dir.join("apps/website")).unwrap();
        fs::write(dir.join("apps/website/index.html"), "<h1>x</h1>").unwrap();
        let pages = cloudflare_put_argv(&dir, "API_TOKEN", None);
        assert!(pages.iter().any(|a| a == "pages"), "{pages:?}");
        assert!(pages.iter().any(|a| a == "secret"), "{pages:?}");
        assert!(
            pages.windows(2).any(|w| w[0] == "--project-name"),
            "pages path needs --project-name, got {pages:?}"
        );

        let named = cloudflare_put_argv(&dir, "API_TOKEN", Some("Ship Studio"));
        assert!(
            named.windows(2).any(|w| w[0] == "--project-name" && w[1] == "ship-studio"),
            "expected sanitized host_name, got {named:?}"
        );

        let website = dir.join("apps/website");
        let from_folder = cloudflare_put_argv(&website, "API_TOKEN", None);
        assert!(
            from_folder
                .windows(2)
                .any(|w| w[0] == "--project-name" && w[1] == "website"),
            "folder basename fallback, got {from_folder:?}"
        );

        fs::write(dir.join("wrangler.toml"), "name = \"w\"\n").unwrap();
        let workers = cloudflare_put_argv(&dir, "API_TOKEN", Some("ignored-for-workers"));
        assert_eq!(
            workers,
            vec![
                "wrangler".to_string(),
                "secret".to_string(),
                "put".to_string(),
                "API_TOKEN".to_string()
            ]
        );
    }

    #[test]
    fn scrub_cli_detail_redacts_secret() {
        let d = scrub_cli_detail("Error: bad token super-secret-value here", "super-secret-value");
        assert!(!d.contains("super-secret-value"));
        assert!(d.contains("••••"));
    }

    #[test]
    fn scrub_cli_detail_strips_ansi_and_tags_missing_project() {
        let raw = "\u{1b}[31mX\u{1b}[0m \u{1b}[41;31m[\u{1b}[41;97mERROR\u{1b}[41;31m]\u{1b}[0m Project \"ship-studio\" does not exist.";
        let d = scrub_cli_detail(raw, "");
        assert!(!d.contains('\u{1b}'), "{d}");
        assert!(!d.contains("[31m"), "{d}");
        assert!(d.contains("Project \"ship-studio\" does not exist"), "{d}");
        assert!(d.contains("[missing_project]"), "{d}");
    }

    #[test]
    fn strip_ansi_bare_sgr_remnants() {
        let d = strip_ansi("[31mX [0mERROR Project \"x\" does not exist.[0m");
        assert!(!d.contains("[31m"), "{d}");
        assert!(d.contains("ERROR Project"), "{d}");
    }
}
