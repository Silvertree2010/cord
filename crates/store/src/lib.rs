use rusqlite::{Connection, params};
use std::{
    collections::HashSet,
    path::Path,
    sync::{Mutex, mpsc},
};

pub struct Row {
    pub chan: String,
    pub id: String,
    pub json: String,
    pub ts: i64,
    pub deleted: bool,
    pub edited: bool,
}

pub struct Log {
    pub chan: String,
    pub id: String,
    pub kind: &'static str,
    pub body: String,
    pub ts: i64,
}

enum W {
    Msg(Row),
    Log(Log),
    Purge(String),
    PurgeChan(String),
    Del { chan: String, id: String, keep: bool, ts: i64 },
    Flush(mpsc::Sender<()>),
}

pub struct Store {
    rd: Mutex<Connection>,
    tx: Mutex<mpsc::Sender<W>>,
}

pub const VER: i64 = 1;
pub const CAP: i64 = 500;

const SCHEMA: &str = "
create table if not exists messages(chan text not null, id text not null, json text not null, ts integer not null, deleted integer not null default 0, edited integer not null default 0, primary key(chan,id));
create index if not exists messages_ts on messages(chan, ts desc);
create table if not exists logs(chan text not null, id text not null, kind text not null, body text not null, ts integer not null);
create index if not exists logs_id on logs(chan, id);
";

fn touch(p: &Path) -> std::io::Result<()> {
    let mut o = std::fs::OpenOptions::new();
    o.create(true).append(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        o.mode(0o600);
    }
    o.open(p).map(|_| ())
}

fn conn(p: &Path) -> rusqlite::Result<Connection> {
    let c = Connection::open(p)?;
    c.pragma_update(None, "journal_mode", "WAL")?;
    c.pragma_update(None, "synchronous", "NORMAL")?;
    c.busy_timeout(std::time::Duration::from_secs(5))?;
    Ok(c)
}

fn chk<T>(what: &str, r: rusqlite::Result<T>) {
    if let Err(e) = r {
        tracing::warn!("store {what}: {e}");
    }
}

fn prune(w: &Connection, chan: &str) {
    chk(
        "prune",
        w.execute("delete from messages where chan=?1 and id not in (select id from messages where chan=?1 order by ts desc, id desc limit ?2)", params![chan, CAP]),
    );
    chk("prune logs", w.execute("delete from logs where chan=?1 and id not in (select id from messages where chan=?1)", params![chan]));
}

fn apply(w: &Connection, x: W, touched: &mut HashSet<String>, acks: &mut Vec<mpsc::Sender<()>>) {
    match x {
        W::Msg(r) => {
            chk(
                "put",
                w.execute(
                    "insert into messages(chan,id,json,ts,deleted,edited) values(?1,?2,?3,?4,?5,?6) on conflict(chan,id) do update set json=excluded.json, ts=excluded.ts, deleted=excluded.deleted, edited=excluded.edited",
                    params![r.chan, r.id, r.json, r.ts, r.deleted as i64, r.edited as i64],
                ),
            );
            touched.insert(r.chan);
        }
        W::Log(l) => chk("log", w.execute("insert into logs(chan,id,kind,body,ts) values(?1,?2,?3,?4,?5)", params![l.chan, l.id, l.kind, l.body, l.ts])),
        W::Del { chan, id, keep, ts } => {
            if keep {
                let body: Option<String> = w.query_row("select json_extract(json,'$.body') from messages where chan=?1 and id=?2", params![chan, id], |r| r.get(0)).ok().flatten();
                if let Some(b) = body {
                    chk("log", w.execute("insert into logs(chan,id,kind,body,ts) values(?1,?2,'del',?3,?4)", params![chan, id, b, ts]));
                }
                chk("del", w.execute("update messages set deleted=1, json=json_set(json,'$.del',json('true')) where chan=?1 and id=?2", params![chan, id]));
            } else {
                chk("del", w.execute("delete from messages where chan=?1 and id=?2", params![chan, id]));
                chk("del logs", w.execute("delete from logs where chan=?1 and id=?2", params![chan, id]));
            }
        }
        W::Purge(pfx) => {
            let pat = format!("{}%", pfx.replace('%', "").replace('_', ""));
            chk("purge", w.execute("delete from messages where chan like ?1", params![pat]));
            chk("purge logs", w.execute("delete from logs where chan like ?1", params![pat]));
        }
        W::PurgeChan(c) => {
            chk("purge chan", w.execute("delete from messages where chan=?1", params![c]));
            chk("purge chan logs", w.execute("delete from logs where chan=?1", params![c]));
            touched.remove(&c);
        }
        W::Flush(a) => acks.push(a),
    }
}

impl Store {
    pub fn open(p: &Path) -> Result<Self, String> {
        touch(p).map_err(|e| e.to_string())?;
        let w = conn(p).map_err(|e| e.to_string())?;
        let v: i64 = w.pragma_query_value(None, "user_version", |r| r.get(0)).map_err(|e| e.to_string())?;
        if v > VER {
            return Err(format!("cache schema {v} is newer than supported {VER}"));
        }
        w.execute_batch(SCHEMA).map_err(|e| e.to_string())?;
        if v < VER {
            w.pragma_update(None, "user_version", VER).map_err(|e| e.to_string())?;
        }
        let rd = conn(p).map_err(|e| e.to_string())?;
        let (tx, rx) = mpsc::channel::<W>();
        std::thread::spawn(move || {
            while let Ok(first) = rx.recv() {
                let mut batch = vec![first];
                while let Ok(n) = rx.try_recv() {
                    batch.push(n);
                    if batch.len() >= 256 {
                        break;
                    }
                }
                chk("begin", w.execute_batch("begin"));
                let mut acks = vec![];
                let mut touched = HashSet::new();
                for x in batch {
                    apply(&w, x, &mut touched, &mut acks);
                }
                for c in &touched {
                    prune(&w, c);
                }
                chk("commit", w.execute_batch("commit"));
                for a in acks {
                    let _ = a.send(());
                }
            }
        });
        Ok(Self { rd: Mutex::new(rd), tx: Mutex::new(tx) })
    }

    fn send(&self, w: W) {
        if let Ok(t) = self.tx.lock() {
            if t.send(w).is_err() {
                tracing::warn!("store writer is gone");
            }
        }
    }

    pub fn put(&self, r: Row) {
        self.send(W::Msg(r));
    }

    pub fn log(&self, l: Log) {
        self.send(W::Log(l));
    }

    pub fn del(&self, chan: &str, id: &str, keep: bool, ts: i64) {
        self.send(W::Del { chan: chan.to_string(), id: id.to_string(), keep, ts });
    }

    pub fn purge_prefix(&self, pfx: &str) {
        self.send(W::Purge(pfx.to_string()));
    }

    pub fn purge_chan(&self, chan: &str) {
        self.send(W::PurgeChan(chan.to_string()));
    }

    pub fn flush(&self) {
        let (a, b) = mpsc::channel();
        if let Ok(t) = self.tx.lock() {
            if t.send(W::Flush(a)).is_ok() {
                let _ = b.recv_timeout(std::time::Duration::from_secs(5));
            }
        }
    }

    pub fn last(&self, chan: &str, n: u32) -> Vec<String> {
        let Ok(c) = self.rd.lock() else { return vec![] };
        let mut s = match c.prepare("select json from messages where chan=?1 order by ts desc, id desc limit ?2") {
            Ok(s) => s,
            Err(e) => {
                tracing::warn!("store read: {e}");
                return vec![];
            }
        };
        let it = match s.query_map(params![chan, n], |r| r.get::<_, String>(0)) {
            Ok(i) => i,
            Err(e) => {
                tracing::warn!("store read: {e}");
                return vec![];
            }
        };
        let mut v: Vec<String> = it
            .filter_map(|r| match r {
                Ok(x) => Some(x),
                Err(e) => {
                    tracing::warn!("store row: {e}");
                    None
                }
            })
            .collect();
        v.reverse();
        v
    }

    pub fn logs(&self, chan: &str, id: &str) -> Vec<(String, String, i64)> {
        let Ok(c) = self.rd.lock() else { return vec![] };
        let Ok(mut s) = c.prepare("select kind,body,ts from logs where chan=?1 and id=?2 order by ts") else { return vec![] };
        let Ok(it) = s.query_map(params![chan, id], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?))) else { return vec![] };
        it.flatten().collect()
    }

    pub fn logs_chan(&self, chan: &str) -> Vec<(String, String, String, i64)> {
        let Ok(c) = self.rd.lock() else { return vec![] };
        let Ok(mut s) = c.prepare("select id,kind,body,ts from logs where chan=?1 order by ts") else { return vec![] };
        let Ok(it) = s.query_map(params![chan], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?))) else { return vec![] };
        it.flatten().collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tmp(n: &str) -> std::path::PathBuf {
        let d = std::env::temp_dir().join(format!("cord-store-{n}-{}", std::process::id()));
        let _ = std::fs::create_dir_all(&d);
        d.join("cache.db")
    }

    #[test]
    fn roundtrip_survives_reopen() {
        let p = tmp("rt");
        {
            let s = Store::open(&p).unwrap();
            for i in 0..5 {
                s.put(Row { chan: "dc:1:c".into(), id: format!("m{i}"), json: format!("{{\"i\":{i}}}"), ts: i, deleted: i == 3, edited: false });
            }
            s.put(Row { chan: "dc:1:c".into(), id: "m3".into(), json: "{\"i\":3,\"del\":true}".into(), ts: 3, deleted: true, edited: true });
            s.log(Log { chan: "dc:1:c".into(), id: "m3".into(), kind: "del", body: "x".into(), ts: 9 });
            s.flush();
        }
        let s = Store::open(&p).unwrap();
        let v = s.last("dc:1:c", 3);
        assert_eq!(v, vec!["{\"i\":2}", "{\"i\":3,\"del\":true}", "{\"i\":4}"]);
        assert_eq!(s.logs("dc:1:c", "m3").len(), 1);
        s.purge_prefix("dc:1:");
        s.flush();
        assert!(s.last("dc:1:c", 3).is_empty());
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            assert_eq!(std::fs::metadata(&p).unwrap().permissions().mode() & 0o777, 0o600);
        }
    }

    #[test]
    fn cap_del_version_and_logs() {
        let p = tmp("cap");
        let s = Store::open(&p).unwrap();
        for i in 0..(CAP + 20) {
            s.put(Row { chan: "dc:1:c".into(), id: format!("{i:05}"), json: format!("{{\"body\":\"b{i}\"}}"), ts: i, deleted: false, edited: false });
        }
        s.flush();
        assert_eq!(s.last("dc:1:c", 1000).len(), CAP as usize);
        s.del("dc:1:c", "00519", true, 7);
        s.del("dc:1:c", "00518", false, 7);
        s.flush();
        let v = s.last("dc:1:c", 2);
        assert_eq!(v, vec!["{\"body\":\"b517\"}".to_string(), "{\"body\":\"b519\",\"del\":true}".to_string()]);
        let l = s.logs_chan("dc:1:c");
        assert_eq!(l, vec![("00519".to_string(), "del".to_string(), "b519".to_string(), 7)]);
        s.purge_chan("dc:1:c");
        s.flush();
        assert!(s.last("dc:1:c", 3).is_empty() && s.logs_chan("dc:1:c").is_empty());
        drop(s);
        let c = Connection::open(&p).unwrap();
        c.pragma_update(None, "user_version", VER + 1).unwrap();
        drop(c);
        assert!(Store::open(&p).is_err());
    }
}
