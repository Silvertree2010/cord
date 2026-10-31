use std::{
    collections::HashMap,
    sync::{Arc, Mutex, OnceLock},
};

pub type Fr = Arc<cord_cap::Frame>;

#[derive(Default)]
struct Slot {
    f: Option<Fr>,
    seq: u64,
}

static F: OnceLock<Mutex<HashMap<String, Slot>>> = OnceLock::new();

fn m() -> &'static Mutex<HashMap<String, Slot>> {
    F.get_or_init(Default::default)
}

pub fn put(k: &str, f: Fr) {
    if let Ok(mut g) = m().lock() {
        let s = g.entry(k.to_string()).or_default();
        s.seq += 1;
        s.f = Some(f);
    }
}

pub fn get(k: &str) -> Option<Fr> {
    m().lock().ok()?.get(k).and_then(|s| s.f.clone())
}

pub fn seq(k: &str) -> u64 {
    m().lock().ok().and_then(|g| g.get(k).map(|s| s.seq)).unwrap_or(0)
}

pub fn drop_k(k: &str) {
    if let Ok(mut g) = m().lock() {
        if let Some(s) = g.get_mut(k) {
            s.f = None;
            s.seq += 1;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn seq_increments_per_key() {
        let f = Arc::new(cord_cap::Frame { w: 1, h: 1, rgba: vec![0; 4] });
        put("t-a", f.clone());
        put("t-a", f.clone());
        put("t-b", f);
        assert_eq!(seq("t-a"), 2);
        assert_eq!(seq("t-b"), 1);
        drop_k("t-a");
        assert_eq!(seq("t-a"), 3);
        assert!(get("t-a").is_none());
    }
}
