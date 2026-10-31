use serde_json::Value;
use std::{
    collections::HashMap,
    path::{Component, Path, PathBuf},
    sync::{Mutex, OnceLock},
    time::{Duration, Instant},
};

pub const SUBS: &[&str] = &["mods", "themes", "blocks"];

pub fn dir() -> PathBuf {
    let d = std::env::var_os("CORD_HOME").map(PathBuf::from).unwrap_or_else(|| dirs::config_dir().unwrap_or_else(|| PathBuf::from(".")).join("cord"));
    let _ = std::fs::create_dir_all(&d);
    d
}

pub fn data() -> PathBuf {
    let d = std::env::var_os("CORD_HOME").map(|p| PathBuf::from(p).join("data")).unwrap_or_else(|| dirs::data_dir().unwrap_or_else(|| PathBuf::from(".")).join("cord"));
    let _ = std::fs::create_dir_all(&d);
    d
}

pub fn qml_dir() -> PathBuf {
    if let Some(p) = std::env::var_os("CORD_QML") {
        return PathBuf::from(p);
    }
    if let Ok(e) = std::env::current_exe() {
        if let Some(p) = e.parent().and_then(|p| p.parent()) {
            let s = p.join("share/cord/qml");
            if s.join("Main.qml").exists() {
                return s;
            }
        }
    }
    PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR"), "/qml"))
}

pub fn valid_name(n: &str) -> bool {
    let mut c = Path::new(n).components();
    matches!((c.next(), c.next()), (Some(Component::Normal(_)), None)) && !n.contains('\\')
}

pub fn safe_rel(sub: &str) -> Option<PathBuf> {
    let p = Path::new(sub);
    if p.is_absolute() {
        return None;
    }
    let mut o = PathBuf::new();
    for c in p.components() {
        match c {
            Component::Normal(x) => o.push(x),
            Component::CurDir => {}
            _ => return None,
        }
    }
    Some(o)
}

pub fn rd(n: &str) -> Option<Value> {
    rd_in(&dir(), n).ok().flatten()
}

pub fn rd_in(d: &Path, n: &str) -> anyhow::Result<Option<Value>> {
    if !valid_name(n) {
        anyhow::bail!("bad file name");
    }
    match std::fs::read_to_string(d.join(n)) {
        Ok(s) => Ok(Some(serde_json::from_str(&s).map_err(|e| anyhow::anyhow!("{n} is not valid JSON: {e}"))?)),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(e) => Err(e.into()),
    }
}

static OWN: OnceLock<Mutex<HashMap<String, Instant>>> = OnceLock::new();

fn own() -> &'static Mutex<HashMap<String, Instant>> {
    OWN.get_or_init(Default::default)
}

pub fn is_own(p: &Path) -> bool {
    let Some(n) = p.file_name().map(|n| n.to_string_lossy().to_string()) else { return false };
    let n = n.trim_end_matches(".tmp").to_string();
    own().lock().ok().is_some_and(|g| g.iter().any(|(k, t)| t.elapsed() < Duration::from_millis(1500) && (k == &n || k.trim_end_matches(".json") == n)))
}

pub fn wr(n: &str, v: &Value) -> anyhow::Result<()> {
    wr_in(&dir(), n, v)?;
    if let Ok(mut g) = own().lock() {
        g.insert(n.to_string(), Instant::now());
    }
    Ok(())
}

pub fn wr_in(d: &Path, n: &str, v: &Value) -> anyhow::Result<()> {
    anyhow::ensure!(valid_name(n), "bad file name");
    let p = d.join(n);
    let t = p.with_extension("tmp");
    std::fs::write(&t, serde_json::to_vec_pretty(v)?)?;
    std::fs::rename(t, p)?;
    Ok(())
}

pub fn ls(sub: &str, ext: &str) -> Vec<PathBuf> {
    let Some(rel) = safe_rel(sub) else { return vec![] };
    let d = dir().join(&rel);
    if rel.components().count() == 1 && SUBS.contains(&rel.to_string_lossy().as_ref()) {
        let _ = std::fs::create_dir_all(&d);
    }
    let mut v: Vec<PathBuf> = std::fs::read_dir(&d).into_iter().flatten().flatten().map(|e| e.path()).filter(|p| ext.is_empty() || p.extension().map(|x| x == ext).unwrap_or(false)).collect();
    v.sort();
    v
}

pub fn readable(p: &Path) -> bool {
    let Ok(c) = p.canonicalize() else { return false };
    let ok = |b: PathBuf| b.canonicalize().map(|b| c.starts_with(b)).unwrap_or(false);
    ok(dir()) || ok(qml_dir())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn tmp(n: &str) -> PathBuf {
        let d = std::env::temp_dir().join(format!("cord-cfg-{n}-{}", std::process::id()));
        let _ = std::fs::create_dir_all(&d);
        d
    }

    #[test]
    fn names() {
        assert!(valid_name("ui.json"));
        assert!(!valid_name("../ui.json"));
        assert!(!valid_name("/etc/passwd"));
        assert!(!valid_name("a/b.json"));
        assert!(!valid_name(".."));
        assert!(!valid_name(""));
        assert_eq!(safe_rel("mods"), Some(PathBuf::from("mods")));
        assert_eq!(safe_rel("mods/x"), Some(PathBuf::from("mods/x")));
        assert_eq!(safe_rel(""), Some(PathBuf::new()));
        assert!(safe_rel("/etc").is_none());
        assert!(safe_rel("a/../../b").is_none());
    }

    #[test]
    fn roundtrip_and_bad_json() {
        let d = tmp("rt");
        wr_in(&d, "x.json", &json!({"a": [1, 2]})).unwrap();
        assert_eq!(rd_in(&d, "x.json").unwrap(), Some(json!({"a": [1, 2]})));
        assert_eq!(rd_in(&d, "none.json").unwrap(), None);
        std::fs::write(d.join("bad.json"), "{nope").unwrap();
        assert!(rd_in(&d, "bad.json").is_err());
        assert!(rd_in(&d, "../x.json").is_err());
    }
}
