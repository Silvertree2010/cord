use serde_json::Value;
use std::path::PathBuf;

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

pub fn rd(n: &str) -> Option<Value> {
    std::fs::read_to_string(dir().join(n)).ok().and_then(|s| serde_json::from_str(&s).ok())
}

pub fn wr(n: &str, v: &Value) -> anyhow::Result<()> {
    let p = dir().join(n);
    let t = p.with_extension("tmp");
    std::fs::write(&t, serde_json::to_vec_pretty(v)?)?;
    std::fs::rename(t, p)?;
    Ok(())
}

pub fn ls(sub: &str, ext: &str) -> Vec<PathBuf> {
    let d = dir().join(sub);
    let _ = std::fs::create_dir_all(&d);
    let mut v: Vec<PathBuf> = std::fs::read_dir(&d).into_iter().flatten().flatten().map(|e| e.path()).filter(|p| ext.is_empty() || p.extension().map(|x| x == ext).unwrap_or(false)).collect();
    v.sort();
    v
}
