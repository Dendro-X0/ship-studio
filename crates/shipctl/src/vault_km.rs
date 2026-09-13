//! Clavis/Keys Manager–compatible encrypted vault (kmvault v1).
//! Format: docs from keys-manager `docs/vault-format.md`.

use anyhow::{bail, Context, Result};
use aes_gcm::aead::{Aead, KeyInit};
use aes_gcm::{Aes256Gcm, Key, Nonce};
use argon2::{
    password_hash::{PasswordHasher, SaltString},
    Algorithm, Argon2, Params, Version,
};
use rand::RngCore;
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::{self, IsTerminal, Write};
use std::path::{Path, PathBuf};
use zeroize::Zeroize;

const MAGIC: &[u8] = b"kmvault";
const VERSION: u8 = 1;
const SALT_LEN: usize = 16;
const NONCE_LEN: usize = 12;
const KEY_LEN: usize = 32;
const DEFAULT_M_COST: u32 = 19_456;
const DEFAULT_T_COST: u32 = 2;
const DEFAULT_P_COST: u32 = 1;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct VaultMeta {
    name: String,
    created_at: String,
    updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Entry {
    id: String,
    entry_type: String,
    title: String,
    #[serde(default)]
    username: String,
    #[serde(default)]
    password: String,
    #[serde(default)]
    url: String,
    #[serde(default)]
    notes: String,
    #[serde(default)]
    notes_format: String,
    #[serde(default)]
    otp_secret: String,
    #[serde(default)]
    custom_fields: Vec<serde_json::Value>,
    #[serde(default)]
    tags: Vec<String>,
    #[serde(default)]
    attachments: Vec<serde_json::Value>,
    created_at: String,
    updated_at: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    deleted_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Workspace {
    id: String,
    name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    source_file: Option<String>,
    created_at: String,
    updated_at: String,
    #[serde(default)]
    entries: Vec<Entry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct VaultDocument {
    meta: VaultMeta,
    #[serde(default)]
    workspaces: Vec<Workspace>,
    #[serde(default)]
    active_workspace_id: String,
    /// Legacy flat list (Clavis migrates on load); we keep empty on write.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    entries: Vec<Entry>,
}

#[derive(Clone)]
struct VaultKey {
    bytes: [u8; KEY_LEN],
}

impl Drop for VaultKey {
    fn drop(&mut self) {
        self.bytes.zeroize();
    }
}

fn now_rfc3339() -> String {
    time::OffsetDateTime::now_utc()
        .format(&time::format_description::well_known::Rfc3339)
        .unwrap_or_else(|_| "1970-01-01T00:00:00Z".into())
}

fn new_id() -> String {
    let mut b = [0u8; 16];
    rand::thread_rng().fill_bytes(&mut b);
    // UUID-ish hex without pulling uuid crate.
    format!(
        "{:02x}{:02x}{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}",
        b[0], b[1], b[2], b[3], b[4], b[5], b[6], b[7], b[8], b[9], b[10], b[11], b[12], b[13],
        b[14], b[15]
    )
}

fn derive_key(password: &str, salt: &[u8], m: u32, t: u32, p: u32) -> Result<VaultKey> {
    let argon_params = Params::new(m, t, p, Some(KEY_LEN))
        .map_err(|e| anyhow::anyhow!("argon2 params: {e}"))?;
    let argon2 = Argon2::new(Algorithm::Argon2id, Version::V0x13, argon_params);
    let salt_string =
        SaltString::encode_b64(salt).map_err(|e| anyhow::anyhow!("salt: {e}"))?;
    let hash = argon2
        .hash_password(password.as_bytes(), &salt_string)
        .map_err(|e| anyhow::anyhow!("argon2: {e}"))?;
    let hash_bytes = hash
        .hash
        .ok_or_else(|| anyhow::anyhow!("missing argon2 hash"))?;
    let raw = hash_bytes.as_bytes();
    if raw.len() < KEY_LEN {
        bail!("argon2 output too short");
    }
    let mut key = [0u8; KEY_LEN];
    key.copy_from_slice(&raw[..KEY_LEN]);
    Ok(VaultKey { bytes: key })
}

fn encrypt(key: &VaultKey, plaintext: &[u8], nonce: &[u8; NONCE_LEN]) -> Result<Vec<u8>> {
    let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(&key.bytes));
    cipher
        .encrypt(Nonce::from_slice(nonce), plaintext)
        .map_err(|e| anyhow::anyhow!("encrypt: {e}"))
}

fn decrypt(key: &VaultKey, ciphertext: &[u8], nonce: &[u8; NONCE_LEN]) -> Result<Vec<u8>> {
    let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(&key.bytes));
    cipher
        .decrypt(Nonce::from_slice(nonce), ciphertext)
        .map_err(|_| anyhow::anyhow!("wrong passphrase or corrupt vault"))
}

fn write_blob(
    salt: &[u8; SALT_LEN],
    m: u32,
    t: u32,
    p: u32,
    nonce: &[u8; NONCE_LEN],
    ciphertext: &[u8],
) -> Vec<u8> {
    let mut out = Vec::with_capacity(MAGIC.len() + 1 + SALT_LEN + 12 + NONCE_LEN + ciphertext.len());
    out.extend_from_slice(MAGIC);
    out.push(VERSION);
    out.extend_from_slice(salt);
    out.extend_from_slice(&m.to_le_bytes());
    out.extend_from_slice(&t.to_le_bytes());
    out.extend_from_slice(&p.to_le_bytes());
    out.extend_from_slice(nonce);
    out.extend_from_slice(ciphertext);
    out
}

fn read_blob(bytes: &[u8]) -> Result<(
    [u8; SALT_LEN],
    u32,
    u32,
    u32,
    [u8; NONCE_LEN],
    Vec<u8>,
)> {
    let min = MAGIC.len() + 1 + SALT_LEN + 12 + NONCE_LEN;
    if bytes.len() < min || &bytes[..MAGIC.len()] != MAGIC {
        bail!("not a kmvault file");
    }
    let mut i = MAGIC.len();
    let ver = bytes[i];
    i += 1;
    if ver != VERSION {
        bail!("unsupported kmvault version {ver}");
    }
    let mut salt = [0u8; SALT_LEN];
    salt.copy_from_slice(&bytes[i..i + SALT_LEN]);
    i += SALT_LEN;
    let m = u32::from_le_bytes(bytes[i..i + 4].try_into()?);
    i += 4;
    let t = u32::from_le_bytes(bytes[i..i + 4].try_into()?);
    i += 4;
    let p = u32::from_le_bytes(bytes[i..i + 4].try_into()?);
    i += 4;
    let mut nonce = [0u8; NONCE_LEN];
    nonce.copy_from_slice(&bytes[i..i + NONCE_LEN]);
    i += NONCE_LEN;
    Ok((salt, m, t, p, nonce, bytes[i..].to_vec()))
}

fn encode_document(doc: &VaultDocument, passphrase: &str) -> Result<Vec<u8>> {
    let mut salt = [0u8; SALT_LEN];
    rand::thread_rng().fill_bytes(&mut salt);
    let key = derive_key(passphrase, &salt, DEFAULT_M_COST, DEFAULT_T_COST, DEFAULT_P_COST)?;
    let mut plaintext = serde_json::to_vec(doc)?;
    let mut nonce = [0u8; NONCE_LEN];
    rand::thread_rng().fill_bytes(&mut nonce);
    let ciphertext = encrypt(&key, &plaintext, &nonce)?;
    plaintext.zeroize();
    Ok(write_blob(
        &salt,
        DEFAULT_M_COST,
        DEFAULT_T_COST,
        DEFAULT_P_COST,
        &nonce,
        &ciphertext,
    ))
}

fn decode_document(bytes: &[u8], passphrase: &str) -> Result<VaultDocument> {
    let (salt, m, t, p, nonce, ciphertext) = read_blob(bytes)?;
    let key = derive_key(passphrase, &salt, m, t, p)?;
    let mut plaintext = decrypt(&key, &ciphertext, &nonce)?;
    let doc: VaultDocument = serde_json::from_slice(&plaintext)
        .context("vault JSON")?;
    plaintext.zeroize();
    Ok(doc)
}

fn empty_doc(name: &str) -> VaultDocument {
    let now = now_rfc3339();
    let ws_id = new_id();
    VaultDocument {
        meta: VaultMeta {
            name: name.into(),
            created_at: now.clone(),
            updated_at: now.clone(),
        },
        workspaces: vec![Workspace {
            id: ws_id.clone(),
            name: "Ship Studio".into(),
            source_file: Some("shipctl vault export".into()),
            created_at: now.clone(),
            updated_at: now,
            entries: Vec::new(),
        }],
        active_workspace_id: ws_id,
        entries: Vec::new(),
    }
}

fn make_api_entry(title: &str, value: &str, url: &str, notes: &str, tags: &[String]) -> Entry {
    let now = now_rfc3339();
    Entry {
        id: new_id(),
        entry_type: "api".into(),
        title: title.into(),
        username: String::new(),
        password: value.into(),
        url: url.into(),
        notes: notes.into(),
        notes_format: "plain".into(),
        otp_secret: String::new(),
        custom_fields: Vec::new(),
        tags: tags.to_vec(),
        attachments: Vec::new(),
        created_at: now.clone(),
        updated_at: now,
        deleted_at: None,
    }
}

pub fn read_passphrase(confirm: bool) -> Result<String> {
    if let Ok(p) = std::env::var("SHIP_VAULT_PASSPHRASE") {
        if !p.is_empty() {
            return Ok(p);
        }
    }
    if !io::stdin().is_terminal() {
        bail!("passphrase required: set SHIP_VAULT_PASSPHRASE or use a TTY");
    }
    eprint!("Vault passphrase: ");
    let _ = io::stderr().flush();
    let mut pass = rpassword::read_password().context("read passphrase")?;
    if confirm {
        eprint!("Confirm passphrase: ");
        let _ = io::stderr().flush();
        let mut again = rpassword::read_password().context("confirm passphrase")?;
        if pass != again {
            pass.zeroize();
            again.zeroize();
            bail!("passphrases do not match");
        }
        again.zeroize();
    }
    if pass.is_empty() {
        bail!("passphrase must not be empty");
    }
    Ok(pass)
}

fn read_line_prompt(prompt: &str) -> Result<String> {
    eprint!("{prompt}");
    let _ = io::stderr().flush();
    let mut line = String::new();
    io::stdin().read_line(&mut line)?;
    Ok(line.trim().to_string())
}

fn read_secret_prompt(prompt: &str) -> Result<String> {
    if !io::stdin().is_terminal() {
        bail!("secret value requires TTY (or use --value-env)");
    }
    eprint!("{prompt}");
    let _ = io::stderr().flush();
    Ok(rpassword::read_password().context("read secret")?)
}

/// Create a new vault from prepared entries (non-interactive; passphrase still required).
pub fn export_entries(
    out: &Path,
    vault_name: &str,
    entries: &[(String, String, String, String)],
) -> Result<PathBuf> {
    if entries.is_empty() {
        bail!("no secrets to export");
    }
    let mut pass = read_passphrase(true)?;
    let mut doc = empty_doc(vault_name);
    for (title, value, url, notes) in entries {
        doc.workspaces[0].entries.push(make_api_entry(
            title,
            value,
            url,
            notes,
            &["ship-studio".into()],
        ));
    }
    let bytes = encode_document(&doc, &pass)?;
    pass.zeroize();
    if let Some(parent) = out.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(out, &bytes).with_context(|| format!("write {}", out.display()))?;
    Ok(out.to_path_buf())
}

/// Load entries from JSON array: `[{title,value,url?,notes?}, …]`.
pub fn load_entries_file(path: &Path) -> Result<Vec<(String, String, String, String)>> {
    let raw = fs::read_to_string(path).with_context(|| format!("read {}", path.display()))?;
    let v: serde_json::Value = serde_json::from_str(&raw).context("parse entries JSON")?;
    let arr = v
        .as_array()
        .ok_or_else(|| anyhow::anyhow!("entries file must be a JSON array"))?;
    let mut out = Vec::new();
    for item in arr {
        let title = item
            .get("title")
            .or_else(|| item.get("name"))
            .and_then(|x| x.as_str())
            .unwrap_or("")
            .trim()
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
            .unwrap_or("Exported from shipctl vault")
            .to_string();
        out.push((title, value, url, notes));
    }
    if out.is_empty() {
        bail!("entries file had no usable title/value pairs");
    }
    Ok(out)
}

/// Create a new vault with a single entry (non-interactive; passphrase still required).
pub fn export_one(
    out: &Path,
    vault_name: &str,
    title: &str,
    value: &str,
    url: &str,
) -> Result<PathBuf> {
    let mut pass = read_passphrase(true)?;
    let mut doc = empty_doc(vault_name);
    doc.workspaces[0].entries.push(make_api_entry(
        title,
        value,
        url,
        "Exported from shipctl vault",
        &["ship-studio".into()],
    ));
    let bytes = encode_document(&doc, &pass)?;
    pass.zeroize();
    if let Some(parent) = out.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(out, &bytes).with_context(|| format!("write {}", out.display()))?;
    Ok(out.to_path_buf())
}

/// Create a new vault file from interactive entries (or hint names then values).
pub fn export_interactive(
    out: &Path,
    vault_name: &str,
    hint_names: &[String],
) -> Result<PathBuf> {
    let mut pass = read_passphrase(true)?;
    let mut doc = empty_doc(vault_name);
    let ws = &mut doc.workspaces[0];

    if hint_names.is_empty() {
        eprintln!("Enter secrets to store (empty name to finish). Values are hidden.");
        loop {
            let title = read_line_prompt("Title / name (empty=done): ")?;
            if title.is_empty() {
                break;
            }
            let url = read_line_prompt("Source URL (optional): ")?;
            let value = read_secret_prompt("Secret value: ")?;
            if value.is_empty() {
                eprintln!("skipped empty value");
                continue;
            }
            ws.entries.push(make_api_entry(
                &title,
                &value,
                &url,
                "Exported from shipctl vault",
                &["ship-studio".into()],
            ));
        }
    } else {
        eprintln!(
            "Paste values for {} hint(s). Leave empty to skip a name.",
            hint_names.len()
        );
        for name in hint_names {
            let url = crate::portal::source_url_for_secret_name(name);
            let value = read_secret_prompt(&format!("{name} value: "))?;
            if value.is_empty() {
                eprintln!("  skipped {name}");
                continue;
            }
            ws.entries.push(make_api_entry(
                name,
                &value,
                url,
                crate::portal::once_hint_for_secret_name(name),
                &["ship-studio".into(), name.clone()],
            ));
        }
    }

    if ws.entries.is_empty() {
        pass.zeroize();
        bail!("no secrets entered — vault not written");
    }

    let bytes = encode_document(&doc, &pass)?;
    pass.zeroize();
    if let Some(parent) = out.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(out, &bytes).with_context(|| format!("write {}", out.display()))?;
    Ok(out.to_path_buf())
}

pub fn add_secret(
    file: &Path,
    title: &str,
    value: &str,
    url: &str,
    notes: &str,
) -> Result<()> {
    let mut pass = read_passphrase(false)?;
    let bytes = fs::read(file).with_context(|| format!("read {}", file.display()))?;
    let mut doc = decode_document(&bytes, &pass)?;
    if doc.workspaces.is_empty() {
        doc = empty_doc(&doc.meta.name);
    }
    let now = now_rfc3339();
    doc.meta.updated_at = now.clone();
    let ws = &mut doc.workspaces[0];
    ws.updated_at = now;
    ws.entries.push(make_api_entry(
        title,
        value,
        url,
        notes,
        &["ship-studio".into()],
    ));
    let out = encode_document(&doc, &pass)?;
    pass.zeroize();
    fs::write(file, out)?;
    Ok(())
}

pub fn list_titles(file: &Path) -> Result<Vec<String>> {
    let mut pass = read_passphrase(false)?;
    let bytes = fs::read(file)?;
    let doc = decode_document(&bytes, &pass)?;
    pass.zeroize();
    let mut titles = Vec::new();
    for ws in &doc.workspaces {
        for e in &ws.entries {
            if e.deleted_at.is_none() {
                titles.push(format!("{} · {}", ws.name, e.title));
            }
        }
    }
    Ok(titles)
}

pub fn show_value(file: &Path, title: &str) -> Result<String> {
    let mut pass = read_passphrase(false)?;
    let bytes = fs::read(file)?;
    let doc = decode_document(&bytes, &pass)?;
    pass.zeroize();
    for ws in &doc.workspaces {
        for e in &ws.entries {
            if e.deleted_at.is_none() && e.title.eq_ignore_ascii_case(title) {
                return Ok(e.password.clone());
            }
        }
    }
    bail!("no entry titled '{title}'");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trip_kmvault() {
        let mut doc = empty_doc("test");
        doc.workspaces[0]
            .entries
            .push(make_api_entry("GITHUB_TOKEN", "ghp_test_secret", "https://github.com", "", &[]));
        let bytes = encode_document(&doc, "correct horse battery").unwrap();
        assert!(bytes.starts_with(MAGIC));
        let decoded = decode_document(&bytes, "correct horse battery").unwrap();
        assert_eq!(decoded.workspaces[0].entries[0].password, "ghp_test_secret");
        assert!(decode_document(&bytes, "wrong").is_err());
    }
}
