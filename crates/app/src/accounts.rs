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

async fn finish(h: &Arc<Hub>, e: &Em, ac: acc::Ac, sec: String) -> Res<()> {
    let demo = ac.base == "demo";
    if !demo {
        acc::put(ac.id, &sec)?;
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
    let ac = acc::Ac { id, svc: sv, lbl: v["lbl"].as_str().unwrap_or("").into(), base, on: true };
    finish(&h, &e, ac, sec).await
}

pub async fn qr(h: Arc<Hub>, e: Em) -> Res<()> {
    let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();
    let e2 = e.clone();
    tokio::spawn(async move {
        while let Some(r) = rx.recv().await {
            match r {
                cord_dc::ra::Ra::Qr(u) => e2.s(&Out::Qr(u)),
                cord_dc::ra::Ra::Usr(u) => e2.s(&Out::QrUsr { name: u.name, id: u.id }),
            }
        }
    });
    let tok = cord_dc::ra::login(tx).await?;
    let id = reserve().await?;
    let ac = acc::Ac { id, svc: Svc::Dc, lbl: String::new(), base: String::new(), on: true };
    finish(&h, &e, ac, tok).await
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
    acc::drop_sec(id);
    let d = crate::cfg::data().join(format!("mx{id}"));
    if d.starts_with(crate::cfg::data()) && d.is_dir() {
        let _ = std::fs::remove_dir_all(d);
    }
    e.s(&Out::AccGone { acc: id });
    e.s(&Out::Accs);
    Ok(())
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
        let s = acc::get(a.id);
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
