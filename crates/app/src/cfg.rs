use serde_json::Value;
use std::{
    collections::HashMap,
    hash::{Hash, Hasher},
    path::{Component, Path, PathBuf},
    sync::{Mutex, OnceLock},
};

pub const SUBS: &[&str] = &["mods", "themes", "blocks"];

static MADE: Mutex<Vec<PathBuf>> = Mutex::new(Vec::new());

fn ensure(d: PathBuf) -> PathBuf {
    if let Ok(mut g) = MADE.lock() {
        if !g.contains(&d) {
            let _ = std::fs::create_dir_all(&d);
            g.push(d.clone());
        }
    }
    d
}

pub fn dir() -> PathBuf {
    ensure(std::env::var_os("CORD_HOME").map(PathBuf::from).unwrap_or_else(|| dirs::config_dir().unwrap_or_else(|| PathBuf::from(".")).join("cord")))
}

pub fn data() -> PathBuf {
    ensure(std::env::var_os("CORD_HOME").map(|p| PathBuf::from(p).join("data")).unwrap_or_else(|| dirs::data_dir().unwrap_or_else(|| PathBuf::from(".")).join("cord")))
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

static OWN: OnceLock<Mutex<HashMap<String, u64>>> = OnceLock::new();
static UPD: Mutex<()> = Mutex::new(());

fn own() -> &'static Mutex<HashMap<String, u64>> {
    OWN.get_or_init(Default::default)
}

fn hash(b: &[u8]) -> u64 {
    let mut h = std::collections::hash_map::DefaultHasher::new();
    b.hash(&mut h);
    h.finish()
}

pub fn is_own(p: &Path) -> bool {
    let Some(n) = p.file_name().map(|n| n.to_string_lossy().to_string()) else { return false };
    let Some(want) = own().lock().ok().and_then(|g| g.get(&n).copied()) else { return false };
    std::fs::read(p).map(|b| hash(&b) == want).unwrap_or(false)
}

pub fn wr(n: &str, v: &Value) -> anyhow::Result<()> {
    anyhow::ensure!(valid_name(n), "bad file name");
    let b = serde_json::to_vec_pretty(v)?;
    if let Ok(mut g) = own().lock() {
        g.insert(n.to_string(), hash(&b));
    }
    let r = wr_bytes(&dir(), n, &b);
    if r.is_err() {
        if let Ok(mut g) = own().lock() {
            g.remove(n);
        }
    }
    r
}

pub fn update(n: &str, f: impl FnOnce(&mut Value)) -> anyhow::Result<()> {
    let _g = UPD.lock().unwrap_or_else(|e| e.into_inner());
    let mut v = rd_in(&dir(), n)?.unwrap_or(Value::Object(Default::default()));
    f(&mut v);
    wr(n, &v)
}

static TMPN: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);

pub fn tmp_for(p: &Path, tail: &str) -> PathBuf {
    let n = p.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_default();
    p.with_file_name(format!(".{n}.{}.{}.{tail}", std::process::id(), TMPN.fetch_add(1, std::sync::atomic::Ordering::Relaxed)))
}

pub fn sync_dir(d: &Path) {
    if let Ok(f) = std::fs::File::open(d) {
        let _ = f.sync_all();
    }
}

pub fn atomic_write(p: &Path, b: &[u8], tail: &str, secret: bool) -> std::io::Result<()> {
    use std::io::Write;
    let t = tmp_for(p, tail);
    let mut o = std::fs::OpenOptions::new();
    o.write(true).create_new(true);
    #[cfg(unix)]
    if secret {
        use std::os::unix::fs::OpenOptionsExt;
        o.mode(0o600);
    }
    let _ = secret;
    let r = (|| {
        let mut f = o.open(&t)?;
        f.write_all(b)?;
        f.sync_all()?;
        std::fs::rename(&t, p)?;
        if let Some(d) = p.parent() {
            sync_dir(d);
        }
        Ok(())
    })();
    if r.is_err() {
        let _ = std::fs::remove_file(&t);
    }
    r
}

fn wr_bytes(d: &Path, n: &str, b: &[u8]) -> anyhow::Result<()> {
    atomic_write(&d.join(n), b, "tmp", false)?;
    Ok(())
}

#[cfg(test)]
pub fn wr_in(d: &Path, n: &str, v: &Value) -> anyhow::Result<()> {
    anyhow::ensure!(valid_name(n), "bad file name");
    wr_bytes(d, n, &serde_json::to_vec_pretty(v)?)
}

pub fn ls(sub: &str, ext: &str) -> Vec<PathBuf> {
    let Some(rel) = safe_rel(sub) else { return vec![] };
    let d = dir().join(&rel);
    if rel.components().count() == 1 && SUBS.contains(&rel.to_string_lossy().as_ref()) {
        ensure(d.clone());
    }
    if in_data(&d) {
        return vec![];
    }
    let mut v: Vec<PathBuf> = std::fs::read_dir(&d)
        .into_iter()
        .flatten()
        .flatten()
        .map(|e| e.path())
        .filter(|p| ext.is_empty() || p.extension().map(|x| x == ext).unwrap_or(false))
        .filter(|p| !is_sec(p) && !in_data(p))
        .collect();
    v.sort();
    v
}

fn is_sec(p: &Path) -> bool {
    p.extension().is_some_and(|x| x == "sec") || p.to_string_lossy().ends_with(".sec.tmp")
}

fn in_data(p: &Path) -> bool {
    let Ok(d) = data().canonicalize() else { return false };
    p.canonicalize().map(|c| c.starts_with(d)).unwrap_or(false)
}

pub fn readable(p: &Path) -> bool {
    let Ok(c) = p.canonicalize() else { return false };
    if is_sec(&c) || in_data(&c) {
        return false;
    }
    let ok = |b: PathBuf| b.canonicalize().map(|b| c.starts_with(b)).unwrap_or(false);
    ok(dir()) || ok(qml_dir())
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use serde_json::json;

    pub static TENV: Mutex<()> = Mutex::new(());

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
    fn own_writes_match_by_content_hash() {
        let _l = TENV.lock().unwrap_or_else(|e| e.into_inner());
        let d = tmp("own");
        unsafe { std::env::set_var("CORD_HOME", &d) };
        wr("o.json", &json!({"a": 1})).unwrap();
        assert!(is_own(&d.join("o.json")));
        std::fs::write(d.join("o.json"), "{\"a\":2}").unwrap();
        assert!(!is_own(&d.join("o.json")));
    }

    #[test]
    fn secrets_and_data_are_not_listed_or_readable() {
        let _l = TENV.lock().unwrap_or_else(|e| e.into_inner());
        let d = tmp("sec");
        unsafe { std::env::set_var("CORD_HOME", &d) };
        std::fs::create_dir_all(d.join("data")).unwrap();
        std::fs::write(d.join("data").join("acc1.sec"), "tok").unwrap();
        std::fs::write(d.join("x.sec"), "tok").unwrap();
        std::fs::write(d.join("ok.txt"), "hi").unwrap();
        assert!(!readable(&d.join("data").join("acc1.sec")));
        assert!(!readable(&d.join("x.sec")));
        assert!(readable(&d.join("ok.txt")));
        assert!(ls("data", "").is_empty());
        assert!(ls("", "").iter().all(|p| !p.to_string_lossy().ends_with(".sec")));
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
