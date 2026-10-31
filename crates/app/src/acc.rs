use cord_core::Svc;
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Ac {
    pub id: u32,
    pub svc: Svc,
    #[serde(default)]
    pub lbl: String,
    #[serde(default)]
    pub base: String,
    #[serde(default)]
    pub on: bool,
}

pub fn load() -> Vec<Ac> {
    crate::cfg::rd("accs.json").and_then(|v| serde_json::from_value(v).ok()).unwrap_or_default()
}

pub fn save(v: &[Ac]) {
    let _ = crate::cfg::wr("accs.json", &serde_json::to_value(v).unwrap_or(Value::Null));
}

pub fn next(v: &[Ac]) -> u32 {
    v.iter().map(|a| a.id + 1).max().unwrap_or(1)
}

fn ent(id: u32) -> keyring::Result<keyring::Entry> {
    keyring::Entry::new("cord", &format!("acc{id}"))
}

pub fn put(id: u32, s: &str) -> anyhow::Result<()> {
    if let Ok(e) = ent(id) {
        if e.set_password(s).is_ok() {
            return Ok(());
        }
    }
    let p = crate::cfg::data().join(format!("acc{id}.sec"));
    std::fs::write(&p, s)?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&p, std::fs::Permissions::from_mode(0o600))?;
    }
    Ok(())
}

pub fn get(id: u32) -> Option<String> {
    if let Some(s) = ent(id).ok().and_then(|e| e.get_password().ok()) {
        return Some(s);
    }
    std::fs::read_to_string(crate::cfg::data().join(format!("acc{id}.sec"))).ok()
}

pub fn drop_sec(id: u32) {
    if let Ok(e) = ent(id) {
        let _ = e.delete_credential();
    }
    let _ = std::fs::remove_file(crate::cfg::data().join(format!("acc{id}.sec")));
}
