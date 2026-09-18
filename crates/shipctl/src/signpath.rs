//! Dual signing paths: local Signet (self) vs official vendor wizards.
//!
//! `kind`: `self` | `official` (certs / signing) | `submit` (store review upload).

use crate::config;
use serde::Serialize;
use std::path::Path;

#[derive(Debug, Serialize)]
pub struct SignPath {
    pub id: String,
    pub kind: String,
    pub title: String,
    pub detail: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub entry_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub run: Option<Vec<String>>,
}

#[derive(Debug, Serialize)]
pub struct SignPortal {
    pub schema: String,
    pub project: String,
    pub recommended: String,
    pub paths: Vec<SignPath>,
    pub notes: Vec<String>,
}

pub fn plan_for(project: &Path) -> SignPortal {
    let detected = config::probe(project);
    let wants = detected.tauri || detected.signet_toml;
    // Apple notarization + optional MAS: desktop Tauri and/or iOS/Expo layouts.
    let wants_apple = detected.tauri || detected.ios || detected.mobile || detected.expo;
    // Play Console is mobile-only — Tauri desktop must not pull Android lanes.
    let wants_play = detected.android || detected.mobile || detected.expo;
    let mut paths = vec![SignPath {
        id: "self.build".into(),
        kind: "self".into(),
        title: "Self-sign — Signet build".into(),
        detail: "Local identity + `signet build`. Good for dogfood and OSS checksums.".into(),
        entry_url: None,
        run: Some(vec!["signet".into(), "build".into()]),
    }];
    if wants && !detected.signet_toml {
        paths.insert(
            0,
            SignPath {
                id: "self.scan".into(),
                kind: "self".into(),
                title: "Self-sign — scan / init signet.toml".into(),
                detail: "Create signing config from repo scan.".into(),
                entry_url: None,
                run: Some(vec!["signet".into(), "scan".into(), "--apply".into()]),
            },
        );
    }
    paths.push(SignPath {
        id: "self.identity".into(),
        kind: "self".into(),
        title: "Self-sign — identity".into(),
        detail: "List or create a local Signet identity (once per machine).".into(),
        entry_url: None,
        run: Some(vec!["signet".into(), "identity".into(), "list".into()]),
    });

    // --- Official certificates / signing (not store review) ---
    if wants_apple || wants {
        paths.push(SignPath {
            id: "official.apple".into(),
            kind: "official".into(),
            title: "Official — Apple notarization / certificates".into(),
            detail: "Certificates & notarization in Apple Developer. Not App Store review.".into(),
            entry_url: Some(
                "https://developer.apple.com/account/resources/certificates/list".into(),
            ),
            run: None,
        });
    }
    if wants {
        paths.push(SignPath {
            id: "official.windows".into(),
            kind: "official".into(),
            title: "Official — Windows Authenticode / certificates".into(),
            detail: "Code-signing certificate setup. Store listing/submission is a separate step.".into(),
            entry_url: Some("https://partner.microsoft.com/dashboard".into()),
            run: None,
        });
    }
    if detected.graduate_sign {
        paths.push(SignPath {
            id: "graduate.checklist".into(),
            kind: "official".into(),
            title: "Graduate — OV / notarization checklist".into(),
            detail: "Azure Trusted Signing or OV Authenticode + Apple notarization secrets. Do not claim verified publisher until real. Run `signet graduate notes`, then apply/ov-sign/azure-sign/notarize.".into(),
            entry_url: Some(
                "https://learn.microsoft.com/en-us/azure/trusted-signing/".into(),
            ),
            run: Some(vec!["signet".into(), "graduate".into(), "notes".into()]),
        });
    }
    if wants_play {
        paths.push(SignPath {
            id: "official.android".into(),
            kind: "official".into(),
            title: "Official — Play Console / upload key".into(),
            detail: "App signing by Google Play or your upload key — not production review yet.".into(),
            entry_url: Some("https://play.google.com/console".into()),
            run: None,
        });
    }

    // --- Store listing portals (metadata; still not “send for review”) ---
    if wants_apple {
        paths.push(SignPath {
            id: "official.app_store_connect".into(),
            kind: "official".into(),
            title: "Official — App Store Connect (listing)".into(),
            detail: "App record, metadata, pricing, TestFlight builds. Submit-for-review is separate.".into(),
            entry_url: Some("https://appstoreconnect.apple.com".into()),
            run: None,
        });
    }

    // --- Submission / send for review ---
    if wants_apple {
        paths.push(SignPath {
            id: "submit.app_store".into(),
            kind: "submit".into(),
            title: "Submit — App Store review".into(),
            detail: "Upload build if needed, then Submit for Review on App Store Connect.".into(),
            entry_url: Some("https://appstoreconnect.apple.com".into()),
            run: None,
        });
    }
    if wants_play {
        paths.push(SignPath {
            id: "submit.play".into(),
            kind: "submit".into(),
            title: "Submit — Play production / review".into(),
            detail: "Promote the release track and send for review on Play Console.".into(),
            entry_url: Some("https://play.google.com/console".into()),
            run: None,
        });
    }
    if wants {
        paths.push(SignPath {
            id: "submit.microsoft".into(),
            kind: "submit".into(),
            title: "Submit — Microsoft Store".into(),
            detail: "Partner Center product submission / certification — operator-only.".into(),
            entry_url: Some("https://partner.microsoft.com/dashboard/products".into()),
            run: None,
        });
    }

    paths.push(SignPath {
        id: "official.github".into(),
        kind: "official".into(),
        title: "Official — GitHub Release".into(),
        detail: "`signet release` after gh auth, or confirm on github.com/releases/new.".into(),
        entry_url: Some("https://github.com/releases/new".into()),
        run: Some(vec![
            "signet".into(),
            "release".into(),
            "--dry-run".into(),
            "--tag".into(),
            std::env::var("SIGNET_RELEASE_TAG").unwrap_or_else(|_| "v0.1.0".into()),
        ]),
    });

    SignPortal {
        schema: "ship-studio/sign/v1".into(),
        project: project.display().to_string(),
        recommended: if wants {
            "self_then_official".into()
        } else {
            "official_listing_only".into()
        },
        notes: vec![
            "Self-sign is local Signet. Official = certificates; Submit = store review.".into(),
            "Listing (metadata) ≠ Submit for review — confirm each gate separately.".into(),
        ],
        paths,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn mobile_sign_portal_splits_certs_and_submit() {
        let dir = std::env::temp_dir().join(format!(
            "shipctl-signpath-mobile-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_nanos())
                .unwrap_or(0)
        ));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(dir.join("android")).unwrap();
        fs::create_dir_all(dir.join("ios")).unwrap();
        fs::write(
            dir.join("app.json"),
            r#"{"expo":{"name":"demo","slug":"demo"}}"#,
        )
        .unwrap();
        let portal = plan_for(&dir);
        assert!(portal.paths.iter().any(|p| p.id == "official.android"));
        assert!(portal.paths.iter().any(|p| p.id == "official.apple"));
        assert!(portal.paths.iter().any(|p| p.id == "submit.play"));
        assert!(portal.paths.iter().any(|p| p.id == "submit.app_store"));
        assert!(portal
            .paths
            .iter()
            .any(|p| p.id == "submit.play" && p.kind == "submit"));
    }

    #[test]
    fn tauri_sign_portal_includes_microsoft_submit() {
        let dir = std::env::temp_dir().join(format!(
            "shipctl-signpath-tauri-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_nanos())
                .unwrap_or(0)
        ));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(dir.join("src-tauri")).unwrap();
        let portal = plan_for(&dir);
        assert!(portal.paths.iter().any(|p| p.id == "official.windows"));
        assert!(portal.paths.iter().any(|p| p.id == "official.apple"));
        assert!(portal.paths.iter().any(|p| p.id == "submit.microsoft"));
        assert!(portal.paths.iter().any(|p| p.id == "submit.app_store"));
        assert!(
            !portal.paths.iter().any(|p| p.id == "official.android"),
            "desktop Tauri must not pull Play/Android cert lanes"
        );
        assert!(
            !portal.paths.iter().any(|p| p.id == "submit.play"),
            "desktop Tauri must not pull Play submit"
        );
    }
}
