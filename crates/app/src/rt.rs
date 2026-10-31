use cord_store::Store;
use std::sync::{Arc, OnceLock};
use tokio::runtime::Runtime;

static RT: OnceLock<Runtime> = OnceLock::new();
static STORE: OnceLock<Arc<Store>> = OnceLock::new();

pub fn init() {
    RT.get_or_init(|| tokio::runtime::Builder::new_multi_thread().enable_all().worker_threads(4).build().unwrap());
}

pub fn rt() -> &'static Runtime {
    RT.get().unwrap()
}

pub fn keep_store(s: Arc<Store>) {
    let _ = STORE.set(s);
}

pub fn flush_store() {
    if let Some(s) = STORE.get() {
        s.flush();
    }
}
