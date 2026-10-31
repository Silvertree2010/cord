use crate::{acc, br::Em, bks, out::{ErrV, Out}};
use cord_core::*;
use cord_hub::Hub;
use serde_json::Value;
use std::sync::Arc;
use tokio::sync::Mutex;

static LOCK: Mutex<()> = Mutex::const_new(());

pub async fn mk(a: &acc::Ac, sec: Option<String>) -> Res<Arc<dyn Bk>> {
    if a.base == "demo" {
        return Ok(Arc::new(cord_hub::demo::Demo::new(a.id, a.svc)));
    }
    bks::mk(a, sec.ok_or_else(|| anyhow::anyhow!("no stored credentials for account {}", a.id))?).await
}

async fn reserve() -> Res<u32> {
    let _g = LOCK.lock().await;
    let mut r = acc::reg()?;
    let id = r.reserve();
    acc::save_reg(&r)?;
    Ok(id)
}

async fn commit(ac: &acc::Ac) -> Res<()> {
    let _g = LOCK.lock().await;
    let mut r = acc::reg()?;
    r.list.retain(|a| a.id != ac.id);
    r.list.push(ac.clone());
    acc::save_reg(&r)
}

async fn finish(h: &Arc<Hub>, e: &Em, mut ac: acc::Ac, sec: String) -> Res<()> {
    let demo = ac.base == "demo";
    if !demo {
        ac.sec = Some(acc::put(ac.id, &sec)?.name().to_string());
    }
    commit(&ac).await?;
    e.s(&Out::Accs);
    let b = mk(&ac, Some(sec)).await?;
    h.add(b).await
}

pub async fn add(h: Arc<Hub>, e: Em, sv: Svc, v: Value) -> Res<()> {
    let id = reserve().await?;
    let demo = v["demo"].as_bool().unwrap_or(false);
    let (base, sec) = if demo { ("demo".to_string(), String::new()) } else { bks::login(sv, id, &v).await? };
    let ac = acc::Ac { id, svc: sv, lbl: v["lbl"].as_str().unwrap_or("").into(), base, on: true, sec: None };
    finish(&h, &e, ac, sec).await
}

static QR: parking_lot::Mutex<Option<tokio::task::JoinHandle<()>>> = parking_lot::Mutex::new(None);

pub fn qr_cancel() {
    if let Some(j) = QR.lock().take() {
        j.abort();
    }
}

pub fn qr(h: Arc<Hub>, e: Em) {
    let e2 = e.clone();
    let j = crate::rt::rt().spawn(async move {
        if let Err(x) = qr_run(h, e2.clone()).await {
            e2.err(0, "login", x);
        }
    });
    if let Some(o) = QR.lock().replace(j) {
        o.abort();
    }
}

fn fwd(e: &Em, r: cord_dc::ra::Ra) {
    match r {
        cord_dc::ra::Ra::Qr(u) => e.s(&Out::Qr(u)),
        cord_dc::ra::Ra::Usr(u) => e.s(&Out::QrUsr { name: u.name, id: u.id }),
    }
}

async fn qr_run(h: Arc<Hub>, e: Em) -> Res<()> {
    let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();
    let login = cord_dc::ra::login(tx);
    tokio::pin!(login);
    let tok = loop {
        tokio::select! {
            r = &mut login => break r?,
            Some(r) = rx.recv() => fwd(&e, r),
        }
    };
    while let Ok(r) = rx.try_recv() {
        fwd(&e, r);
    }
    let fin = tokio::spawn(async move {
        let id = reserve().await?;
        let ac = acc::Ac { id, svc: Svc::Dc, lbl: String::new(), base: String::new(), on: true, sec: None };
        finish(&h, &e, ac, tok).await
    });
    fin.await?
}

pub async fn remove(h: Arc<Hub>, e: Em, id: u32) -> Res<()> {
    {
        let _g = LOCK.lock().await;
        let mut r = acc::reg()?;
        r.list.retain(|a| a.id != id);
        acc::save_reg(&r)?;
    }
    h.remove(id).await;
    bks::forget(id);
    crate::frames::clear();
    acc::drop_sec(id);
    let d = crate::cfg::data().join(format!("mx{id}"));
    if d.starts_with(crate::cfg::data()) && d.is_dir() {
        let _ = std::fs::remove_dir_all(d);
    }
    e.s(&Out::AccGone { acc: id });
    e.s(&Out::Accs);
    Ok(())
}

async fn set_sec(id: u32, be: &str) {
    let _g = LOCK.lock().await;
    if let Ok(mut r) = acc::reg() {
        if let Some(a) = r.list.iter_mut().find(|a| a.id == id) {
            a.sec = Some(be.to_string());
            let _ = acc::save_reg(&r);
        }
    }
}

pub async fn start_all(h: Arc<Hub>, e: Em) {
    let reg = match acc::reg() {
        Ok(r) => r,
        Err(x) => {
            e.s(&Out::Err(ErrV::new(0, "accounts", x)));
            return;
        }
    };
    for a in reg.list.into_iter().filter(|a| a.on) {
        let s = if a.base == "demo" {
            None
        } else {
            match acc::get(a.id, a.sec.as_deref()) {
                Ok(s) => {
                    if a.sec.is_none() {
                        set_sec(a.id, acc::backend()).await;
                    }
                    Some(s)
                }
                Err(x) => {
                    e.s(&Out::Err(ErrV::new(a.id, "start", x)));
                    continue;
                }
            }
        };
        match mk(&a, s).await {
            Ok(b) => {
                if let Err(x) = h.add(b).await {
                    e.s(&Out::Err(ErrV::new(a.id, "start", x)));
                }
            }
            Err(x) => e.s(&Out::Err(ErrV::new(a.id, "start", x))),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn concurrent_reserve_unique() {
        let _l = crate::cfg::tests::TENV.lock().unwrap_or_else(|e| e.into_inner());
        let d = std::env::temp_dir().join(format!("cord-acct-{}", std::process::id()));
        let _ = std::fs::create_dir_all(&d);
        unsafe { std::env::set_var("CORD_HOME", &d) };
        let _ = std::fs::remove_file(d.join("accs.json"));
        let rt = tokio::runtime::Builder::new_multi_thread().worker_threads(4).enable_all().build().unwrap();
        let ids: Vec<u32> = rt.block_on(async {
            let hs: Vec<_> = (0..8).map(|_| tokio::spawn(reserve())).collect();
            let mut v = vec![];
            for h in hs {
                v.push(h.await.unwrap().unwrap());
            }
            v
        });
        let mut s = ids.clone();
        s.sort();
        s.dedup();
        assert_eq!(s.len(), 8);
        assert_eq!(*s.iter().max().unwrap(), 8);
    }
}
