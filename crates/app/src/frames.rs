use std::{
    collections::HashMap,
    sync::{
        Arc, Mutex, OnceLock,
        atomic::{AtomicBool, Ordering},
    },
    time::Duration,
};

pub type Fr = Arc<cord_cap::Frame>;
type Tick = Box<dyn Fn(Vec<(String, u64)>) + Send + Sync>;

#[derive(Default)]
struct Slot {
    f: Option<Fr>,
    seq: u64,
}

static F: OnceLock<Mutex<HashMap<String, Slot>>> = OnceLock::new();
static TICK: OnceLock<Tick> = OnceLock::new();
static PEND: Mutex<Vec<(String, u64)>> = Mutex::new(Vec::new());
static SCHED: AtomicBool = AtomicBool::new(false);

fn m() -> &'static Mutex<HashMap<String, Slot>> {
    F.get_or_init(Default::default)
}

pub fn set_tick(f: impl Fn(Vec<(String, u64)>) + Send + Sync + 'static) {
    let _ = TICK.set(Box::new(f));
}

fn note(k: &str, seq: u64) {
    if TICK.get().is_none() {
        return;
    }
    if let Ok(mut p) = PEND.lock() {
        match p.iter_mut().find(|x| x.0 == k) {
            Some(x) => x.1 = seq,
            None => p.push((k.to_string(), seq)),
        }
    }
    if !SCHED.swap(true, Ordering::SeqCst) {
        crate::rt::rt().spawn(async {
            tokio::time::sleep(Duration::from_millis(16)).await;
            SCHED.store(false, Ordering::SeqCst);
            let v: Vec<(String, u64)> = PEND.lock().map(|mut p| std::mem::take(&mut *p)).unwrap_or_default();
            if let (false, Some(t)) = (v.is_empty(), TICK.get()) {
                t(v);
            }
        });
    }
}

pub fn put(k: &str, f: Fr) {
    let seq = {
        let Ok(mut g) = m().lock() else { return };
        let s = g.entry(k.to_string()).or_default();
        s.seq += 1;
        s.f = Some(f);
        s.seq
    };
    note(k, seq);
}

pub fn get(k: &str) -> Option<Fr> {
    m().lock().ok()?.get(k).and_then(|s| s.f.clone())
}

pub fn seq(k: &str) -> u64 {
    m().lock().ok().and_then(|g| g.get(k).map(|s| s.seq)).unwrap_or(0)
}

pub fn drop_k(k: &str) {
    let seq = {
        let Ok(mut g) = m().lock() else { return };
        let Some(s) = g.get_mut(k) else { return };
        s.f = None;
        s.seq += 1;
        s.seq
    };
    note(k, seq);
}

pub fn clear() {
    let ks: Vec<String> = m().lock().map(|g| g.iter().filter(|(_, s)| s.f.is_some()).map(|(k, _)| k.clone()).collect()).unwrap_or_default();
    for k in ks {
        drop_k(&k);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    static G: std::sync::Mutex<()> = std::sync::Mutex::new(());

    #[test]
    fn seq_increments_per_key() {
        let _g = G.lock().unwrap_or_else(|e| e.into_inner());
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

    #[test]
    fn clear_drops_all_frames_and_bumps() {
        let _g = G.lock().unwrap_or_else(|e| e.into_inner());
        let f = Arc::new(cord_cap::Frame { w: 1, h: 1, rgba: vec![0; 4] });
        put("t-c", f.clone());
        put("t-d", f);
        clear();
        assert!(get("t-c").is_none() && get("t-d").is_none());
        assert_eq!(seq("t-c"), 2);
    }
}
