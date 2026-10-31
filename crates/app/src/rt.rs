use std::sync::OnceLock;
use tokio::runtime::Runtime;

static RT: OnceLock<Runtime> = OnceLock::new();

pub fn init() {
    RT.get_or_init(|| tokio::runtime::Builder::new_multi_thread().enable_all().worker_threads(4).build().unwrap());
}

pub fn rt() -> &'static Runtime {
    RT.get().unwrap()
}
