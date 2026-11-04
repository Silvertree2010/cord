use crate::*;
use cord_core::*;
use cord_store::Store;
use std::sync::{
    Arc, Mutex,
    atomic::{AtomicU32, Ordering},
};

struct Stub {
    acks: AtomicU32,
    typs: AtomicU32,
    hists: AtomicU32,
    fail: bool,
    tx: Mutex<Option<Tx>>,
    ret: Mutex<Vec<Msg>>,
}

impl Stub {
    fn new() -> Arc<Self> {
        Arc::new(Self { acks: AtomicU32::new(0), typs: AtomicU32::new(0), hists: AtomicU32::new(0), fail: false, tx: Mutex::new(None), ret: Mutex::new(vec![]) })
    }
}

#[async_trait::async_trait]
impl Bk for Stub {
    fn svc(&self) -> Svc {
        Svc::Dc
    }
    fn acc(&self) -> u32 {
        1
    }
    async fn start(&self, tx: Tx) -> Res<()> {
        *self.tx.lock().unwrap() = Some(tx);
        if self.fail {
            anyhow::bail!("boom");
        }
        Ok(())
    }
    async fn hist(&self, _c: &Rid, b: Option<&Rid>, _n: u32) -> Res<Vec<Msg>> {
        self.hists.fetch_add(1, Ordering::SeqCst);
        tokio::task::yield_now().await;
        if b.is_some() {
            return Ok(vec![]);
        }
        Ok(self.ret.lock().unwrap().clone())
    }
    async fn send(&self, _c: &Rid, _o: cord_core::Out) -> Res<()> {
        Ok(())
    }
    async fn edit(&self, _c: &Rid, _m: &Rid, _b: String) -> Res<()> {
        Ok(())
    }
    async fn del(&self, _c: &Rid, _m: &Rid) -> Res<()> {
        Ok(())
    }
    async fn react(&self, _c: &Rid, _m: &Rid, _e: &Emo, _on: bool) -> Res<()> {
        Ok(())
    }
    async fn typing(&self, _c: &Rid) -> Res<()> {
        self.typs.fetch_add(1, Ordering::SeqCst);
        Ok(())
    }
    async fn ack(&self, _c: &Rid, _m: &Rid) -> Res<()> {
        self.acks.fetch_add(1, Ordering::SeqCst);
        Ok(())
    }
}

fn rid(r: &str) -> Rid {
    Rid::new(Svc::Dc, 1, r)
}

fn msg(id: &str, ts: i64, body: &str) -> Msg {
    Msg {
        id: rid(id),
        chan: rid("c"),
        auth: Usr { id: Some(rid("200")), name: "a".into(), ..Default::default() },
        body: body.into(),
        ts,
        ed: None,
        atts: vec![],
        embs: vec![],
        reacts: vec![],
        reply: None,
        stks: vec![],
        pin: false,
        sys: false,
        del: false,
        hist: vec![],
        ments: vec![],
        aroles: vec![],
    }
}

fn chan(id: &str, enc: bool) -> Chan {
    Chan { id: rid(id), gld: None, par: None, k: ChK::Txt, name: id.into(), topic: None, pos: 0, nsfw: false, hidden: false, last: Some(rid("9")), rcp: vec![], icon: None, enc, mute: false }
}

async fn settle() {
    tokio::time::sleep(std::time::Duration::from_millis(30)).await;
}

fn tmp(n: &str) -> std::path::PathBuf {
    let d = std::env::temp_dir().join(format!("cord-hub-{n}-{}", std::process::id()));
    let _ = std::fs::create_dir_all(&d);
    d.join("cache.db")
}

#[tokio::test]
async fn noread_and_silenttyping_skip_backend() {
    let h = Hub::new(Arc::new(|_| {}), None);
    let b = Stub::new();
    h.bks.write().await.insert(1, b.clone());
    let c = rid("c");
    h.st.write().await.chans.insert(c.clone(), chan("c", false));
    feat::set("noread", false);
    feat::set("silenttyping", false);
    h.st.write().await.unr.insert(c.clone(), Unr { n: 1, ment: 0 });
    h.ack(&c).await.unwrap();
    h.typing(&c).await.unwrap();
    assert_eq!((b.acks.load(Ordering::SeqCst), b.typs.load(Ordering::SeqCst)), (1, 1));
    h.st.write().await.unr.insert(c.clone(), Unr { n: 1, ment: 0 });
    h.ack(&c).await.unwrap();
    assert_eq!(b.acks.load(Ordering::SeqCst), 1);
    feat::set("noread", true);
    feat::set("silenttyping", true);
    h.st.write().await.unr.insert(c.clone(), Unr { n: 1, ment: 0 });
    h.st.write().await.chans.get_mut(&c).unwrap().last = Some(rid("10"));
    h.ack(&c).await.unwrap();
    h.typing(&c).await.unwrap();
    assert_eq!((b.acks.load(Ordering::SeqCst), b.typs.load(Ordering::SeqCst)), (1, 1));
    feat::set("noread", false);
    feat::set("silenttyping", false);
}

#[tokio::test]
async fn ack_without_unread_is_noop() {
    let h = Hub::new(Arc::new(|_| {}), None);
    let b = Stub::new();
    h.bks.write().await.insert(1, b.clone());
    h.st.write().await.chans.insert(rid("c"), chan("c", false));
    h.ack(&rid("c")).await.unwrap();
    assert_eq!(b.acks.load(Ordering::SeqCst), 0);
}

#[tokio::test]
async fn remove_drops_queued_and_late_events() {
    let seen = Arc::new(Mutex::new(0usize));
    let s2 = seen.clone();
    let h = Hub::new(Arc::new(move |_| *s2.lock().unwrap() += 1), None);
    let b = Stub::new();
    h.add(b.clone()).await.unwrap();
    let tx = b.tx.lock().unwrap().clone().unwrap();
    tx.send(Ev::Chan(chan("c", false))).unwrap();
    for i in 0..20 {
        tx.send(Ev::Msg(msg(&format!("{}", 100 + i), i, "x"))).unwrap();
    }
    h.remove(1).await;
    let after = *seen.lock().unwrap();
    for i in 0..20 {
        tx.send(Ev::Msg(msg(&format!("{}", 200 + i), i, "late"))).unwrap();
    }
    tx.send(Ev::Conn { acc: 1, up: true }).unwrap();
    settle().await;
    let st = h.st.read().await;
    assert!(st.msgs.is_empty() && st.chans.is_empty() && st.unr.is_empty() && st.up.is_empty());
    assert!(h.bks.read().await.is_empty());
    assert!(*seen.lock().unwrap() >= after);
    drop(st);
    let n = *seen.lock().unwrap();
    settle().await;
    assert_eq!(*seen.lock().unwrap(), n);
}

#[tokio::test]
async fn start_failure_unregisters_backend() {
    let h = Hub::new(Arc::new(|_| {}), None);
    let b = Arc::new(Stub { acks: AtomicU32::new(0), typs: AtomicU32::new(0), hists: AtomicU32::new(0), fail: true, tx: Mutex::new(None), ret: Mutex::new(vec![]) });
    assert!(h.add(b.clone()).await.is_err());
    assert!(h.bks.read().await.is_empty());
    let tx = b.tx.lock().unwrap().clone().unwrap();
    tx.send(Ev::Msg(msg("1", 1, "x"))).unwrap();
    settle().await;
    assert!(h.st.read().await.msgs.is_empty());
    assert!(h.bk(1).await.is_err());
}

#[tokio::test]
async fn hist_is_coalesced_and_synced() {
    let h = Hub::new(Arc::new(|_| {}), None);
    let b = Stub::new();
    *b.ret.lock().unwrap() = vec![msg("1", 1, "a"), msg("2", 2, "b")];
    h.add(b.clone()).await.unwrap();
    let c = rid("c");
    let (x, y) = tokio::join!(h.hist(&c, None), h.hist(&c, None));
    let (x, y) = (x.unwrap(), y.unwrap());
    assert_eq!(b.hists.load(Ordering::SeqCst), 1);
    assert!(x.noop ^ y.noop);
    assert!(h.hist(&c, None).await.unwrap().noop);
    assert_eq!(b.hists.load(Ordering::SeqCst), 1);
    let (list, synced) = h.msgs(&c).await;
    assert!(synced);
    assert_eq!(list.len(), 2);
    h.tx().send(Ev::Conn { acc: 1, up: false }).unwrap();
    settle().await;
    assert!(!h.msgs(&c).await.1);
    assert!(!h.hist(&c, None).await.unwrap().noop);
    assert_eq!(b.hists.load(Ordering::SeqCst), 2);
}

#[tokio::test]
async fn cache_edit_while_offline_server_wins() {
    let p = tmp("edit");
    let store = Arc::new(Store::open(&p).unwrap());
    {
        let h = Hub::new(Arc::new(|_| {}), Some(store.clone()));
        let b = Stub::new();
        h.add(b.clone()).await.unwrap();
        let tx = b.tx.lock().unwrap().clone().unwrap();
        tx.send(Ev::Msg(msg("1", 1, "old"))).unwrap();
        tx.send(Ev::Msg(msg("2", 2, "keep"))).unwrap();
        settle().await;
        store.flush();
    }
    let h = Hub::new(Arc::new(|_| {}), Some(store.clone()));
    let b = Stub::new();
    *b.ret.lock().unwrap() = vec![msg("1", 1, "new"), msg("2", 2, "keep")];
    h.add(b.clone()).await.unwrap();
    let c = rid("c");
    let (cached, synced) = h.msgs(&c).await;
    assert!(!synced);
    assert_eq!(cached[0].m.body, "old");
    assert!(h.st.read().await.stale.get(&c).is_some_and(|s| s.len() == 2));
    let r = h.hist(&c, None).await.unwrap();
    assert!(!r.noop && r.added.len() == 2);
    let (now, synced) = h.msgs(&c).await;
    assert!(synced);
    assert_eq!(now[0].m.body, "new");
    assert!(h.st.read().await.stale.get(&c).is_none());
    store.flush();
    assert!(store.last("dc:1:c", 5).iter().any(|j| j.contains("\"new\"")));
}

#[tokio::test]
async fn cache_gap_replaces_window_and_offline_delete_drops() {
    let p = tmp("gap");
    let store = Arc::new(Store::open(&p).unwrap());
    {
        let h = Hub::new(Arc::new(|_| {}), Some(store.clone()));
        let b = Stub::new();
        h.add(b.clone()).await.unwrap();
        let tx = b.tx.lock().unwrap().clone().unwrap();
        for i in 1..=4 {
            tx.send(Ev::Msg(msg(&format!("{i}"), i, "o"))).unwrap();
        }
        settle().await;
        store.flush();
    }
    let h = Hub::new(Arc::new(|_| {}), Some(store.clone()));
    let b = Stub::new();
    *b.ret.lock().unwrap() = vec![msg("1", 1, "o"), msg("2", 2, "o"), msg("4", 4, "o")];
    h.add(b.clone()).await.unwrap();
    let c = rid("c");
    h.msgs(&c).await;
    let r = h.hist(&c, None).await.unwrap();
    assert!(!r.added.is_empty());
    let ids: Vec<String> = h.msgs(&c).await.0.iter().map(|m| m.m.id.raw.to_string()).collect();
    assert_eq!(ids, vec!["1", "2", "4"]);

    let p = tmp("gap2");
    let store = Arc::new(Store::open(&p).unwrap());
    {
        let h = Hub::new(Arc::new(|_| {}), Some(store.clone()));
        let b = Stub::new();
        h.add(b.clone()).await.unwrap();
        let tx = b.tx.lock().unwrap().clone().unwrap();
        tx.send(Ev::Msg(msg("1", 1, "o"))).unwrap();
        tx.send(Ev::Msg(msg("2", 2, "o"))).unwrap();
        settle().await;
        store.flush();
    }
    let h = Hub::new(Arc::new(|_| {}), Some(store.clone()));
    let b = Stub::new();
    *b.ret.lock().unwrap() = vec![msg("100", 100, "n"), msg("101", 101, "n")];
    h.add(b.clone()).await.unwrap();
    h.msgs(&c).await;
    h.hist(&c, None).await.unwrap();
    let ids: Vec<String> = h.msgs(&c).await.0.iter().map(|m| m.m.id.raw.to_string()).collect();
    assert_eq!(ids, vec!["100", "101"]);
}

#[tokio::test]
async fn deletes_write_through_without_memory_and_chandel_purges() {
    let p = tmp("del");
    let store = Arc::new(Store::open(&p).unwrap());
    {
        let h = Hub::new(Arc::new(|_| {}), Some(store.clone()));
        let b = Stub::new();
        h.add(b.clone()).await.unwrap();
        let tx = b.tx.lock().unwrap().clone().unwrap();
        tx.send(Ev::Msg(msg("1", 1, "a"))).unwrap();
        tx.send(Ev::Msg(msg("2", 2, "b"))).unwrap();
        settle().await;
        store.flush();
    }
    let h = Hub::new(Arc::new(|_| {}), Some(store.clone()));
    let b = Stub::new();
    h.add(b.clone()).await.unwrap();
    let tx = b.tx.lock().unwrap().clone().unwrap();
    tx.send(Ev::MsgDel { chan: rid("c"), id: rid("1") }).unwrap();
    settle().await;
    store.flush();
    assert_eq!(store.last("dc:1:c", 5).len(), 1);
    tx.send(Ev::ChanDel(rid("c"))).unwrap();
    settle().await;
    store.flush();
    assert!(store.last("dc:1:c", 5).is_empty());
}

#[tokio::test]
async fn encrypted_rooms_are_not_cached() {
    let p = tmp("enc");
    let store = Arc::new(Store::open(&p).unwrap());
    let h = Hub::new(Arc::new(|_| {}), Some(store.clone()));
    let b = Stub::new();
    h.add(b.clone()).await.unwrap();
    let tx = b.tx.lock().unwrap().clone().unwrap();
    tx.send(Ev::Chan(chan("c", true))).unwrap();
    tx.send(Ev::Msg(msg("1", 1, "secret"))).unwrap();
    settle().await;
    store.flush();
    assert!(store.last("dc:1:c", 5).is_empty());
}

#[tokio::test]
async fn message_logger_restores_after_restart() {
    let p = tmp("logs");
    let store = Arc::new(Store::open(&p).unwrap());
    {
        let h = Hub::new(Arc::new(|_| {}), Some(store.clone()));
        h.st.write().await.cfg.log_ed = true;
        h.st.write().await.cfg.log_del = true;
        let b = Stub::new();
        h.add(b.clone()).await.unwrap();
        let tx = b.tx.lock().unwrap().clone().unwrap();
        tx.send(Ev::Msg(msg("1", 1, "first"))).unwrap();
        tx.send(Ev::Msg(msg("2", 2, "gone"))).unwrap();
        settle().await;
        let mut e = msg("1", 1, "second");
        e.ed = Some(5);
        tx.send(Ev::MsgUpd(e)).unwrap();
        tx.send(Ev::MsgDel { chan: rid("c"), id: rid("2") }).unwrap();
        settle().await;
        store.flush();
    }
    let h = Hub::new(Arc::new(|_| {}), Some(store.clone()));
    let (v, _) = h.msgs(&rid("c")).await;
    assert_eq!(v[0].m.hist, vec!["first".to_string()]);
    assert!(v[1].m.del);
}

struct DemoStub(Arc<Stub>);

#[async_trait::async_trait]
impl Bk for DemoStub {
    fn svc(&self) -> Svc {
        Svc::Dc
    }
    fn acc(&self) -> u32 {
        1
    }
    fn demo(&self) -> bool {
        true
    }
    async fn start(&self, tx: Tx) -> Res<()> {
        self.0.start(tx).await
    }
    async fn hist(&self, c: &Rid, b: Option<&Rid>, n: u32) -> Res<Vec<Msg>> {
        self.0.hist(c, b, n).await
    }
    async fn send(&self, _c: &Rid, _o: cord_core::Out) -> Res<()> {
        Ok(())
    }
    async fn edit(&self, _c: &Rid, _m: &Rid, _b: String) -> Res<()> {
        Ok(())
    }
    async fn del(&self, _c: &Rid, _m: &Rid) -> Res<()> {
        Ok(())
    }
    async fn react(&self, _c: &Rid, _m: &Rid, _e: &Emo, _on: bool) -> Res<()> {
        Ok(())
    }
    async fn typing(&self, _c: &Rid) -> Res<()> {
        Ok(())
    }
    async fn ack(&self, _c: &Rid, _m: &Rid) -> Res<()> {
        Ok(())
    }
}

#[tokio::test]
async fn demo_accounts_are_never_cached() {
    let p = tmp("demo");
    let store = Arc::new(Store::open(&p).unwrap());
    let h = Hub::new(Arc::new(|_| {}), Some(store.clone()));
    let st = Stub::new();
    h.add(Arc::new(DemoStub(st.clone()))).await.unwrap();
    let tx = st.tx.lock().unwrap().clone().unwrap();
    tx.send(Ev::Msg(msg("1", 1, "x"))).unwrap();
    settle().await;
    store.flush();
    assert!(store.last("dc:1:c", 5).is_empty());
}

#[tokio::test]
async fn live_msg_clears_del_and_empty_head_fetch_is_authoritative() {
    let h = Hub::new(Arc::new(|_| {}), None);
    let b = Stub::new();
    h.add(b.clone()).await.unwrap();
    let tx = b.tx.lock().unwrap().clone().unwrap();
    h.st.write().await.cfg.log_del = true;
    tx.send(Ev::Msg(msg("1", 1, "a"))).unwrap();
    tx.send(Ev::MsgDel { chan: rid("c"), id: rid("1") }).unwrap();
    settle().await;
    assert!(h.st.read().await.msgs[&rid("c")][0].del);
    tx.send(Ev::Msg(msg("1", 1, "a"))).unwrap();
    settle().await;
    assert!(!h.st.read().await.msgs[&rid("c")][0].del);
    {
        let mut st = h.st.write().await;
        st.stale.entry(rid("c")).or_default().insert(rid("1"));
        st.cfg.log_del = false;
    }
    let r = h.hist(&rid("c"), None).await.unwrap();
    assert!(!r.noop);
    assert!(h.st.read().await.msgs[&rid("c")].is_empty());
}

#[tokio::test]
async fn ack_sends_local_only_after_server_ack() {
    feat::set("noread", false);
    let h = Hub::new(Arc::new(|_| {}), None);
    let b = Stub::new();
    h.add(b.clone()).await.unwrap();
    let c = rid("c");
    let mut ch = chan("c", false);
    ch.last = None;
    h.st.write().await.chans.insert(c.clone(), ch);
    h.st.write().await.unr.insert(c.clone(), Unr { n: 1, ment: 0 });
    h.ack(&c).await.unwrap();
    settle().await;
    assert_eq!(b.acks.load(Ordering::SeqCst), 0);
    assert!(h.st.read().await.unr.contains_key(&c));
    let tx = b.tx.lock().unwrap().clone().unwrap();
    tx.send(Ev::Msg(msg("5", 5, "x"))).unwrap();
    settle().await;
    h.st.write().await.unr.insert(c.clone(), Unr { n: 1, ment: 0 });
    h.ack(&c).await.unwrap();
    settle().await;
    assert_eq!(b.acks.load(Ordering::SeqCst), 1);
    assert!(!h.st.read().await.unr.contains_key(&c));
}

#[tokio::test]
async fn deltas_carry_monotonic_seq() {
    let seen = Arc::new(Mutex::new(vec![]));
    let s2 = seen.clone();
    let h = Hub::new(Arc::new(move |d: Sd| s2.lock().unwrap().push(d.seq)), None);
    let b = Stub::new();
    h.add(b.clone()).await.unwrap();
    let tx = b.tx.lock().unwrap().clone().unwrap();
    tx.send(Ev::Msg(msg("1", 1, "a"))).unwrap();
    tx.send(Ev::Msg(msg("2", 2, "a"))).unwrap();
    settle().await;
    let v = seen.lock().unwrap().clone();
    assert!(v.len() >= 2);
    assert!(v.windows(2).all(|w| w[1] == w[0] + 1));
    assert_eq!(*v.last().unwrap(), h.seq());
}
