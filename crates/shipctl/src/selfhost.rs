//! Local self-host deploy — Studio-owned lane (stream + last-run + health).
//! Slice 2: detect static surface, stream phases, write `.ship/last-run.json`.
//! Slice 3: bind local static server, GET health, auto-complete (no Confirm).

use anyhow::{bail, Context, Result};
use serde::Serialize;
use std::fs;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

use crate::config;

#[derive(Debug, Clone, Serialize)]
pub struct SelfhostTarget {
    pub kind: &'static str,
    pub path: PathBuf,
    pub rel: String,
}

#[derive(Debug, Serialize)]
pub struct SelfhostReport {
    pub ok: bool,
    pub project: String,
    pub target: Option<SelfhostTarget>,
    pub health_url: Option<String>,
    pub message: String,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct SelfhostOpts {
    /// Keep serving after health check until the process is killed / Cancel.
    pub serve: bool,
}

fn rel_display(project: &Path, path: &Path) -> String {
    path.strip_prefix(project)
        .map(|p| p.display().to_string().replace('\\', "/"))
        .unwrap_or_else(|_| path.display().to_string().replace('\\', "/"))
}

/// Prefer documented static roots; never steals Dockerfile layouts for serve.
pub fn resolve_target(project: &Path) -> Option<SelfhostTarget> {
    let candidates: &[&str] = &[
        "apps/website/index.html",
        "apps/docs/index.html",
        "public/index.html",
        "dist/index.html",
        "index.html",
    ];
    for rel in candidates {
        let path = project.join(rel);
        if path.is_file() {
            let dir = path.parent().unwrap_or(project).to_path_buf();
            let rel = rel_display(project, &dir);
            return Some(SelfhostTarget {
                kind: "static",
                path: dir,
                rel,
            });
        }
    }
    None
}

fn respond_static(mut stream: TcpStream, root: &Path) {
    let mut req = [0u8; 2048];
    let _ = stream.read(&mut req);
    let index = root.join("index.html");
    let body = fs::read(&index).unwrap_or_else(|_| b"selfhost: missing index.html".to_vec());
    let header = format!(
        "HTTP/1.1 200 OK\r\nContent-Type: text/html; charset=utf-8\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
        body.len()
    );
    let _ = stream.write_all(header.as_bytes());
    let _ = stream.write_all(&body);
}

fn http_get_status(host: &str, port: u16) -> Result<u16> {
    let mut stream = TcpStream::connect((host, port))
        .with_context(|| format!("connect health {host}:{port}"))?;
    stream
        .set_read_timeout(Some(Duration::from_secs(2)))
        .ok();
    stream
        .set_write_timeout(Some(Duration::from_secs(2)))
        .ok();
    write!(
        stream,
        "GET / HTTP/1.1\r\nHost: {host}:{port}\r\nConnection: close\r\n\r\n"
    )?;
    let mut buf = Vec::new();
    stream.read_to_end(&mut buf)?;
    let text = String::from_utf8_lossy(&buf);
    let status = text
        .lines()
        .next()
        .and_then(|line| line.split_whitespace().nth(1))
        .and_then(|s| s.parse::<u16>().ok())
        .unwrap_or(0);
    if status == 0 {
        bail!("health response missing status: {}", text.chars().take(80).collect::<String>());
    }
    Ok(status)
}

struct LocalServer {
    port: u16,
    url: String,
    stop: Arc<AtomicBool>,
    join: Option<thread::JoinHandle<()>>,
}

impl LocalServer {
    fn start(root: PathBuf) -> Result<Self> {
        let listener = TcpListener::bind(("127.0.0.1", 0)).context("bind selfhost listener")?;
        let port = listener.local_addr()?.port();
        let url = format!("http://127.0.0.1:{port}/");
        let stop = Arc::new(AtomicBool::new(false));
        let stop_flag = stop.clone();
        listener
            .set_nonblocking(true)
            .context("selfhost listener nonblocking")?;
        let join = thread::spawn(move || {
            while !stop_flag.load(Ordering::SeqCst) {
                match listener.accept() {
                    Ok((stream, _)) => respond_static(stream, &root),
                    Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                        thread::sleep(Duration::from_millis(20));
                    }
                    Err(_) => break,
                }
            }
        });
        // Brief settle so the accept loop is live before the health GET.
        thread::sleep(Duration::from_millis(40));
        Ok(Self {
            port,
            url,
            stop,
            join: Some(join),
        })
    }

    fn stop(mut self) {
        self.stop.store(true, Ordering::SeqCst);
        if let Some(j) = self.join.take() {
            let _ = j.join();
        }
    }
}

fn write_run(
    project: &Path,
    started: String,
    ok: bool,
    message: String,
    urls: Vec<String>,
    detail_checks: &str,
) -> Result<()> {
    let finished = config::now_rfc3339();
    config::write_last_run(
        project,
        &config::LastRun {
            finished: true,
            ok,
            started_at: started,
            finished_at: finished,
            dry_run: false,
            offline: true,
            steps: vec![
                config::StepResult {
                    id: "selfhost".into(),
                    ok,
                    exit_code: if ok { 0 } else { 1 },
                    detail: message.clone(),
                },
                config::StepResult {
                    id: "selfhost.check".into(),
                    ok,
                    exit_code: if ok { 0 } else { 1 },
                    detail: detail_checks.into(),
                },
            ],
            message,
            urls,
        },
    )?;
    Ok(())
}

pub fn run(project: &Path, opts: SelfhostOpts) -> Result<SelfhostReport> {
    let project = project
        .canonicalize()
        .with_context(|| format!("canonicalize {}", project.display()))?;
    println!("selfhost · starting local auto lane");
    println!("selfhost · project {}", project.display());
    println!("selfhost · detecting static surfaces…");

    let started = config::now_rfc3339();
    let Some(target) = resolve_target(&project) else {
        println!("selfhost · no static index yet (tried apps/website, public, dist, root)");
        println!("selfhost · add an index.html under one of those paths, then Deploy again");
        let message = "selfhost: no static surface detected".to_string();
        write_run(&project, started, false, message.clone(), vec![], "artifact:miss")?;
        bail!("{message}");
    };

    println!("selfhost · found {} ({})", target.rel, target.kind);
    println!("selfhost · check · artifact");
    let probe = target.path.join("index.html");
    if !probe.is_file() {
        let message = format!("selfhost: artifact missing {}", probe.display());
        write_run(
            &project,
            started,
            false,
            message.clone(),
            vec![],
            "artifact:miss",
        )?;
        bail!("{message}");
    }
    let _ = fs::metadata(&probe).with_context(|| format!("stat {}", probe.display()))?;
    println!("selfhost · check · artifact ok · {}", target.rel);

    println!("selfhost · check · bind 127.0.0.1");
    let server = LocalServer::start(target.path.clone())?;
    let health_url = server.url.clone();
    println!("selfhost · check · GET {health_url}");

    let status = match http_get_status("127.0.0.1", server.port) {
        Ok(s) => s,
        Err(err) => {
            server.stop();
            let message = format!("selfhost: health failed · {err}");
            write_run(
                &project,
                started,
                false,
                message.clone(),
                vec![],
                "health:fail",
            )?;
            bail!("{message}");
        }
    };
    if status != 200 {
        server.stop();
        let message = format!("selfhost: health status {status}");
        write_run(
            &project,
            started,
            false,
            message.clone(),
            vec![health_url],
            &format!("health:{status}"),
        )?;
        bail!("{message}");
    }

    println!("selfhost · check ok · {health_url}");
    let message = format!("selfhost ok · static · {} · {health_url}", target.rel);
    write_run(
        &project,
        started.clone(),
        true,
        message.clone(),
        vec![health_url.clone()],
        "artifact:ok health:200",
    )?;
    println!("selfhost · done · non-step complete (no Confirm)");

    if opts.serve {
        println!("selfhost · serving until Cancel…");
        // Block the main thread so Desktop Cancel can kill the process tree.
        loop {
            thread::sleep(Duration::from_secs(3600));
        }
    } else {
        server.stop();
    }

    Ok(SelfhostReport {
        ok: true,
        project: project.display().to_string(),
        target: Some(target),
        health_url: Some(health_url),
        message,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn tmp(label: &str) -> PathBuf {
        let n = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let dir = std::env::temp_dir().join(format!("shipctl-selfhost-{label}-{n}"));
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn resolves_harbor_style_website() {
        let dir = tmp("site");
        let web = dir.join("apps/website");
        fs::create_dir_all(&web).unwrap();
        fs::write(web.join("index.html"), "<!doctype html><title>ok</title>").unwrap();
        let t = resolve_target(&dir).expect("target");
        assert_eq!(t.kind, "static");
        assert!(t.rel.contains("apps/website"));
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn run_health_and_last_run() {
        let dir = tmp("run");
        let web = dir.join("apps/website");
        fs::create_dir_all(&web).unwrap();
        fs::write(web.join("index.html"), "<!doctype html><title>harbor</title>").unwrap();
        let report = run(&dir, SelfhostOpts::default()).expect("selfhost");
        assert!(report.ok);
        let url = report.health_url.expect("health url");
        assert!(url.starts_with("http://127.0.0.1:"));
        let last = config::read_last_run(&dir).unwrap();
        assert_eq!(last["ok"], true);
        let urls = last["urls"].as_array().cloned().unwrap_or_default();
        assert!(urls.iter().any(|u| u.as_str() == Some(url.as_str())));
        let steps = last["steps"].as_array().cloned().unwrap_or_default();
        assert!(steps.iter().any(|s| s["id"] == "selfhost.check"));
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn missing_surface_fails() {
        let dir = tmp("empty");
        assert!(run(&dir, SelfhostOpts::default()).is_err());
        let _ = fs::remove_dir_all(&dir);
    }
}
