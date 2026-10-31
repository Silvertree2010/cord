use crate::cfg;
use std::path::{Path, PathBuf};

pub fn cget(n: &str) -> serde_json::Value {
    match cfg::rd_in(&cfg::dir(), n) {
        Ok(Some(v)) => v,
        _ => serde_json::Value::Null,
    }
}

pub fn cset(n: &str, v: &serde_json::Value) -> anyhow::Result<()> {
    anyhow::ensure!(cfg::valid_name(n), "bad file name");
    if n == "cfg.json" {
        anyhow::ensure!(cfg::rd_in(&cfg::dir(), n).is_ok(), "cfg.json exists but is not valid JSON; fix it by hand");
    }
    cfg::wr(n, v)
}

pub fn ls(sub: &str, ext: &str) -> Vec<String> {
    cfg::ls(sub, ext).into_iter().map(|p| p.to_string_lossy().to_string()).collect()
}

pub fn rdf(p: &str) -> String {
    let p = PathBuf::from(p.trim_start_matches("file://"));
    if !p.is_absolute() || !cfg::readable(&p) {
        return String::new();
    }
    std::fs::read_to_string(p).unwrap_or_default()
}

pub fn scheme() -> String {
    dirs::state_dir().map(|d| d.join("caelestia/scheme.json")).and_then(|p| std::fs::read_to_string(p).ok()).unwrap_or_default()
}

pub fn watch_ignored(p: &Path) -> bool {
    let n = p.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_default();
    if n.ends_with(".tmp") || n.ends_with("-journal") || n.ends_with("-wal") || n.ends_with("-shm") {
        return true;
    }
    if let (Ok(d), Ok(c)) = (cfg::data().canonicalize(), p.canonicalize().or_else(|_| p.parent().map(|x| x.canonicalize()).unwrap_or(Err(std::io::Error::other("none"))))) {
        if c.starts_with(d) {
            return true;
        }
    }
    cfg::is_own(p)
}
