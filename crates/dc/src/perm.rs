use cord_core::Gld;
use serde_json::Value;

pub const ADM: u64 = 1 << 3;
pub const VIEW: u64 = 1 << 10;

pub fn pu(v: &Value) -> u64 {
    match v {
        Value::String(s) => s.parse().unwrap_or(0),
        Value::Number(n) => n.as_u64().unwrap_or(0),
        _ => 0,
    }
}

pub struct Ow {
    pub id: String,
    pub m: bool,
    pub al: u64,
    pub dn: u64,
}

pub fn ows(c: &Value) -> Vec<Ow> {
    c.get("permission_overwrites")
        .and_then(|x| x.as_array())
        .map(|a| {
            a.iter()
                .map(|o| Ow {
                    id: o["id"].as_str().unwrap_or("").to_string(),
                    m: match &o["type"] {
                        Value::Number(n) => n.as_u64() == Some(1),
                        Value::String(s) => s == "member" || s == "1",
                        _ => false,
                    },
                    al: pu(&o["allow"]),
                    dn: pu(&o["deny"]),
                })
                .collect()
        })
        .unwrap_or_default()
}

pub fn base(g: &Gld, my: &[String], uid: &str) -> u64 {
    if g.owner.as_ref().map(|o| &*o.raw) == Some(uid) {
        return u64::MAX;
    }
    let mut p = 0u64;
    for r in &g.roles {
        if r.id.raw == g.id.raw || my.iter().any(|m| **m == *r.id.raw) {
            p |= r.perm;
        }
    }
    if p & ADM != 0 { u64::MAX } else { p }
}

pub fn chan(b: u64, gid: &str, my: &[String], uid: &str, o: &[Ow]) -> u64 {
    if b & ADM != 0 {
        return u64::MAX;
    }
    let mut p = b;
    if let Some(e) = o.iter().find(|e| !e.m && e.id == gid) {
        p &= !e.dn;
        p |= e.al;
    }
    let (mut a, mut d) = (0u64, 0u64);
    for e in o.iter().filter(|e| !e.m && my.contains(&e.id)) {
        a |= e.al;
        d |= e.dn;
    }
    p &= !d;
    p |= a;
    if let Some(e) = o.iter().find(|e| e.m && e.id == uid) {
        p &= !e.dn;
        p |= e.al;
    }
    p
}

pub fn view(g: &Gld, my: &[String], uid: &str, c: &Value) -> bool {
    let b = base(g, my, uid);
    chan(b, &g.id.raw, my, uid, &ows(c)) & VIEW != 0
}
