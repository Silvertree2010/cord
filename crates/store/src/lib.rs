use rusqlite::{Connection, params};
use std::{
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
    Flush(mpsc::Sender<()>),
}

pub struct Store {
    rd: Mutex<Connection>,
    tx: Mutex<mpsc::Sender<W>>,
}

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

impl Store {
    pub fn open(p: &Path) -> Result<Self, String> {
        touch(p).map_err(|e| e.to_string())?;
        let w = conn(p).map_err(|e| e.to_string())?;
        w.execute_batch(SCHEMA).map_err(|e| e.to_string())?;
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
                let _ = w.execute_batch("begin");
                let mut acks = vec![];
                for x in batch {
                    match x {
                        W::Msg(r) => {
                            let _ = w.execute(
                                "insert into messages(chan,id,json,ts,deleted,edited) values(?1,?2,?3,?4,?5,?6) on conflict(chan,id) do update set json=excluded.json, ts=excluded.ts, deleted=excluded.deleted, edited=excluded.edited",
                                params![r.chan, r.id, r.json, r.ts, r.deleted as i64, r.edited as i64],
                            );
                        }
                        W::Log(l) => {
                            let _ = w.execute("insert into logs(chan,id,kind,body,ts) values(?1,?2,?3,?4,?5)", params![l.chan, l.id, l.kind, l.body, l.ts]);
                        }
                        W::Purge(pfx) => {
                            let pat = format!("{}%", pfx.replace('%', "").replace('_', ""));
                            let _ = w.execute("delete from messages where chan like ?1", params![pat]);
                            let _ = w.execute("delete from logs where chan like ?1", params![pat]);
                        }
                        W::Flush(a) => acks.push(a),
                    }
                }
                let _ = w.execute_batch("commit");
                for a in acks {
                    let _ = a.send(());
                }
            }
        });
        Ok(Self { rd: Mutex::new(rd), tx: Mutex::new(tx) })
    }

    pub fn put(&self, r: Row) {
        if let Ok(t) = self.tx.lock() {
            let _ = t.send(W::Msg(r));
        }
    }

    pub fn log(&self, l: Log) {
        if let Ok(t) = self.tx.lock() {
            let _ = t.send(W::Log(l));
        }
    }

    pub fn purge_prefix(&self, pfx: &str) {
        if let Ok(t) = self.tx.lock() {
            let _ = t.send(W::Purge(pfx.to_string()));
        }
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
        let Ok(mut s) = c.prepare("select json from messages where chan=?1 order by ts desc, id desc limit ?2") else { return vec![] };
        let Ok(it) = s.query_map(params![chan, n], |r| r.get::<_, String>(0)) else { return vec![] };
        let mut v: Vec<String> = it.flatten().collect();
        v.reverse();
        v
    }

    pub fn logs(&self, chan: &str, id: &str) -> Vec<(String, String, i64)> {
        let Ok(c) = self.rd.lock() else { return vec![] };
        let Ok(mut s) = c.prepare("select kind,body,ts from logs where chan=?1 and id=?2 order by ts") else { return vec![] };
        let Ok(it) = s.query_map(params![chan, id], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?))) else { return vec![] };
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
}
