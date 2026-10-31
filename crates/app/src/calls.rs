use crate::{
    bks,
    frames::{self, Fr},
    rt::rt,
};
use cord_core::*;
use cord_hub::Hub;
use std::sync::Arc;
use tokio::sync::watch;

pub fn pump(mut rx: watch::Receiver<Option<Fr>>, key: String) {
    rt().spawn(async move {
        while rx.changed().await.is_ok() {
            let f = rx.borrow().clone();
            match f {
                Some(f) => frames::put(&key, f),
                None => break,
            }
        }
    });
}

async fn cur(h: &Hub) -> Option<Rid> {
    h.st.read().await.call.chan.clone()
}

pub async fn join(h: Arc<Hub>, c: Rid) -> Res<()> {
    if let Some(o) = cur(&h).await {
        if o.acc != c.acc {
            h.bk(o.acc).await?.leave().await?;
            frames::clear();
        }
    }
    h.bk(c.acc).await?.join(&c).await
}

pub async fn leave(h: Arc<Hub>) -> Res<()> {
    if let Some(o) = cur(&h).await {
        h.bk(o.acc).await?.leave().await?;
    }
    frames::clear();
    Ok(())
}

pub async fn vset(h: Arc<Hub>, mute: bool, deaf: bool) -> Res<()> {
    if let Some(o) = cur(&h).await {
        h.bk(o.acc).await?.vset(mute, deaf).await?;
    }
    Ok(())
}

pub async fn golive(h: Arc<Hub>, on: bool, lq: Lq) -> Res<()> {
    let Some(o) = cur(&h).await else { anyhow::bail!("not in a call") };
    h.bk(o.acc).await?.golive(on, lq).await?;
    if !on {
        frames::drop_k("me");
        return Ok(());
    }
    let me = h.st.read().await.accs.get(&o.acc).and_then(|a| a.me.id.clone());
    if let (Some(d), Some(me)) = (bks::dc_of(o.acc), me) {
        pump(d.frames(&me), "me".into());
    }
    Ok(())
}

pub async fn watch(h: Arc<Hub>, u: Rid, on: bool) -> Res<()> {
    h.bk(u.acc).await?.watch(&u, on).await?;
    let k = u.to_string();
    if !on {
        frames::drop_k(&k);
        return Ok(());
    }
    if let Some(d) = bks::dc_of(u.acc) {
        pump(d.frames(&u), k);
    }
    Ok(())
}
