use crate::cfg;
use std::path::{Path, PathBuf};

static BAD: std::sync::LazyLock<std::sync::Mutex<std::collections::HashMap<String, String>>> = std::sync::LazyLock::new(Default::default);

pub fn cget(n: &str) -> (serde_json::Value, Option<String>) {
    match cfg::rd_in(&cfg::dir(), n) {
        Ok(v) => {
            if let Ok(mut g) = BAD.lock() {
                g.remove(n);
            }
            (v.unwrap_or(serde_json::Value::Null), None)
        }
        Err(x) => {
            let m = x.to_string();
            if let Ok(mut g) = BAD.lock() {
                g.insert(n.to_string(), m.clone());
            }
            (serde_json::Value::Null, Some(m))
        }
    }
}

pub fn bad(n: &str) -> String {
    BAD.lock().ok().and_then(|g| g.get(n).cloned()).unwrap_or_default()
}

pub fn bad_all() -> Vec<(String, String)> {
    BAD.lock().map(|g| g.iter().map(|(k, v)| (k.clone(), v.clone())).collect()).unwrap_or_default()
}

pub fn cset(n: &str, v: &serde_json::Value) -> anyhow::Result<()> {
    anyhow::ensure!(cfg::valid_name(n), "bad file name");
    anyhow::ensure!(n != "accs.json", "accs.json is managed by the account API");
    anyhow::ensure!(n != "cfg.json", "cfg.json is written through the feature API only");
    anyhow::ensure!(!n.ends_with(".sec") && !n.ends_with(".tmp"), "reserved file name");
    if let Err(x) = cfg::rd_in(&cfg::dir(), n) {
        anyhow::bail!("{x}; fix it by hand, not overwriting");
    }
    cfg::wr(n, v)
}

pub fn exists(p: &str) -> bool {
    let p = PathBuf::from(p.trim_start_matches("file://"));
    p.is_absolute() && cfg::readable(&p)
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
