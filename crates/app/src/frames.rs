use std::{
    collections::HashMap,
    sync::{Arc, Mutex, OnceLock},
};

pub type Fr = Arc<cord_cap::Frame>;

static F: OnceLock<Mutex<HashMap<String, Fr>>> = OnceLock::new();

fn m() -> &'static Mutex<HashMap<String, Fr>> {
    F.get_or_init(Default::default)
}

pub fn put(k: &str, f: Fr) {
    if let Ok(mut g) = m().lock() {
        g.insert(k.to_string(), f);
    }
}

pub fn get(k: &str) -> Option<Fr> {
    m().lock().ok()?.get(k).cloned()
}

pub fn drop_k(k: &str) {
    if let Ok(mut g) = m().lock() {
        g.remove(k);
    }
}
