use cord_core::Svc;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::{
    path::Path,
    sync::atomic::{AtomicU8, Ordering},
};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
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

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Reg {
    pub next: u32,
    pub list: Vec<Ac>,
}

impl Reg {
    pub fn parse(v: &Value) -> Reg {
        let (arr, next) = match v {
            Value::Array(_) => (v.clone(), 0),
            Value::Object(o) => (o.get("accs").cloned().unwrap_or(Value::Null), o.get("next").and_then(|n| n.as_u64()).unwrap_or(0) as u32),
            _ => (Value::Null, 0),
        };
        let list: Vec<Ac> = serde_json::from_value(arr).unwrap_or_default();
        let floor = list.iter().map(|a| a.id + 1).max().unwrap_or(1);
        Reg { next: next.max(floor).max(1), list }
    }

    pub fn to_json(&self) -> Value {
        json!({"next": self.next, "accs": self.list})
    }

    pub fn reserve(&mut self) -> u32 {
        let id = self.next;
        self.next += 1;
        id
    }
}

pub fn load_in(d: &Path) -> anyhow::Result<Reg> {
    Ok(crate::cfg::rd_in(d, "accs.json")?.map(|v| Reg::parse(&v)).unwrap_or(Reg { next: 1, list: vec![] }))
}

pub fn save_in(d: &Path, r: &Reg) -> anyhow::Result<()> {
    crate::cfg::wr_in(d, "accs.json", &r.to_json())
}

pub fn reg() -> anyhow::Result<Reg> {
    load_in(&crate::cfg::dir())
}

pub fn save_reg(r: &Reg) -> anyhow::Result<()> {
    crate::cfg::wr("accs.json", &r.to_json())
}

pub fn load() -> Vec<Ac> {
    reg().map(|r| r.list).unwrap_or_default()
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Be {
    Keyring,
    File,
}

impl Be {
    pub fn name(self) -> &'static str {
        match self {
            Be::Keyring => "keyring",
            Be::File => "file",
        }
    }
}

static BE: AtomicU8 = AtomicU8::new(0);

fn note(b: Be) {
    BE.store(if b == Be::Keyring { 1 } else { 2 }, Ordering::Relaxed);
}

pub fn backend() -> &'static str {
    match BE.load(Ordering::Relaxed) {
        1 => "keyring",
        2 => "file",
        _ => "",
    }
}

fn ent(id: u32) -> keyring::Result<keyring::Entry> {
    keyring::Entry::new("cord", &format!("acc{id}"))
}

pub fn write_secret_file(p: &Path, s: &str) -> std::io::Result<()> {
    use std::io::Write;
    let t = p.with_extension("sec.tmp");
    let mut o = std::fs::OpenOptions::new();
    o.write(true).create(true).truncate(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        o.mode(0o600);
    }
    let mut f = o.open(&t)?;
    f.write_all(s.as_bytes())?;
    f.sync_all()?;
    std::fs::rename(t, p)
}

pub fn put(id: u32, s: &str) -> anyhow::Result<Be> {
    if let Ok(e) = ent(id) {
        if e.set_password(s).is_ok() && e.get_password().ok().as_deref() == Some(s) {
            note(Be::Keyring);
            return Ok(Be::Keyring);
        }
    }
    write_secret_file(&crate::cfg::data().join(format!("acc{id}.sec")), s)?;
    note(Be::File);
    Ok(Be::File)
}

pub fn get(id: u32) -> Option<String> {
    if let Some(s) = ent(id).ok().and_then(|e| e.get_password().ok()) {
        note(Be::Keyring);
        return Some(s);
    }
    let r = std::fs::read_to_string(crate::cfg::data().join(format!("acc{id}.sec"))).ok();
    if r.is_some() {
        note(Be::File);
    }
    r
}

pub fn drop_sec(id: u32) {
    if let Ok(e) = ent(id) {
        let _ = e.delete_credential();
    }
    let _ = std::fs::remove_file(crate::cfg::data().join(format!("acc{id}.sec")));
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tmp(n: &str) -> std::path::PathBuf {
        let d = std::env::temp_dir().join(format!("cord-acc-{n}-{}", std::process::id()));
        let _ = std::fs::create_dir_all(&d);
        d
    }

    fn ac(id: u32) -> Ac {
        Ac { id, svc: Svc::Dc, lbl: String::new(), base: String::new(), on: true }
    }

    #[test]
    fn ids_are_monotonic_and_never_reused() {
        let d = tmp("ids");
        let mut r = load_in(&d).unwrap();
        let a = r.reserve();
        let b = r.reserve();
        let c = r.reserve();
        assert_eq!((a, b, c), (1, 2, 3));
        r.list.push(ac(a));
        r.list.push(ac(b));
        r.list.push(ac(c));
        save_in(&d, &r).unwrap();
        let mut r = load_in(&d).unwrap();
        r.list.retain(|x| x.id != 3);
        save_in(&d, &r).unwrap();
        let mut r = load_in(&d).unwrap();
        assert_eq!(r.list.len(), 2);
        assert_eq!(r.reserve(), 4);
    }

    #[test]
    fn legacy_array_format() {
        let v = json!([{"id": 5, "svc": "Dc", "on": true}, {"id": 2, "svc": "Mx"}]);
        let r = Reg::parse(&v);
        assert_eq!(r.next, 6);
        assert_eq!(r.list.len(), 2);
        assert_eq!(Reg::parse(&r.to_json()), r);
    }

    #[test]
    fn corrupt_registry_is_an_error_not_empty() {
        let d = tmp("bad");
        std::fs::write(d.join("accs.json"), "{{{").unwrap();
        assert!(load_in(&d).is_err());
    }

    #[test]
    fn secret_file_mode() {
        let d = tmp("sec");
        let p = d.join("acc1.sec");
        write_secret_file(&p, "tok").unwrap();
        assert_eq!(std::fs::read_to_string(&p).unwrap(), "tok");
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            assert_eq!(std::fs::metadata(&p).unwrap().permissions().mode() & 0o777, 0o600);
        }
    }
}
