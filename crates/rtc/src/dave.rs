use serde_json::{Value, json};

pub enum Out {
    Bin(Vec<u8>),
    Json(Value),
}

#[cfg(feature = "dave")]
pub const MAXV: u16 = davey::DAVE_PROTOCOL_VERSION;
#[cfg(not(feature = "dave"))]
pub const MAXV: u16 = 0;

#[cfg_attr(not(feature = "dave"), allow(dead_code))]
pub fn rd16(d: &[u8]) -> Option<(u16, &[u8])> {
    if d.len() < 2 {
        return None;
    }
    Some((u16::from_be_bytes([d[0], d[1]]), &d[2..]))
}

#[cfg_attr(not(feature = "dave"), allow(dead_code))]
fn tid(t: u16) -> Out {
    Out::Json(json!({"op": 23, "d": {"transition_id": t}}))
}

#[cfg_attr(not(feature = "dave"), allow(dead_code))]
fn bad(t: u16) -> Out {
    Out::Json(json!({"op": 31, "d": {"transition_id": t}}))
}

#[cfg(feature = "dave")]
pub struct Dv {
    s: davey::DaveSession,
    uid: u64,
    cid: u64,
}

#[cfg(feature = "dave")]
impl Dv {
    pub fn new(ver: u16, uid: u64, cid: u64) -> Option<Dv> {
        let v = std::num::NonZeroU16::new(ver)?;
        let s = davey::DaveSession::new(v, uid, cid, None).ok()?;
        Some(Dv { s, uid, cid })
    }

    pub fn ver(&self) -> u16 {
        self.s.protocol_version().get()
    }

    pub fn ready(&self) -> bool {
        self.s.status() == davey::SessionStatus::ACTIVE && self.s.can_encrypt()
    }

    pub fn code(&self, uid: u64) -> Option<String> {
        self.s.get_verification_code(uid).ok()
    }

    fn reset(&mut self, ver: u16) {
        match std::num::NonZeroU16::new(ver) {
            Some(v) => {
                if let Err(e) = self.s.reinit(v, self.uid, self.cid, None) {
                    tracing::warn!("dave reinit: {e}");
                }
            }
            None => {
                let _ = self.s.reset();
            }
        }
    }

    pub fn bin(&mut self, op: u8, d: &[u8]) -> Vec<Out> {
        let mut o = Vec::new();
        match op {
            25 => match self.s.set_external_sender(d) {
                Ok(()) => match self.s.create_key_package() {
                    Ok(k) => o.push(Out::Bin([&[26u8][..], &k].concat())),
                    Err(e) => tracing::warn!("dave keypkg: {e}"),
                },
                Err(e) => tracing::warn!("dave ext sender: {e}"),
            },
            27 => {
                let Some((&t, r)) = d.split_first() else { return o };
                let t = if t == 0 { davey::ProposalsOperationType::APPEND } else { davey::ProposalsOperationType::REVOKE };
                match self.s.process_proposals(t, r, None) {
                    Ok(Some(cw)) => {
                        let mut m = vec![28u8];
                        m.extend_from_slice(&cw.commit);
                        if let Some(w) = cw.welcome {
                            m.extend_from_slice(&w);
                        }
                        o.push(Out::Bin(m));
                    }
                    Ok(None) => {}
                    Err(e) => tracing::warn!("dave proposals: {e}"),
                }
            }
            29 => {
                let Some((t, c)) = rd16(d) else { return o };
                match self.s.process_commit(c) {
                    Ok(true) => self.prep(t, &mut o),
                    Ok(false) => {}
                    Err(e) => {
                        tracing::warn!("dave commit: {e}");
                        o.push(bad(t));
                        self.reset(self.ver());
                    }
                }
            }
            30 => {
                let Some((t, w)) = rd16(d) else { return o };
                match self.s.process_welcome(w, None) {
                    Ok(()) => self.prep(t, &mut o),
                    Err(e) => {
                        tracing::warn!("dave welcome: {e}");
                        o.push(bad(t));
                        self.reset(self.ver());
                    }
                }
            }
            _ => {}
        }
        o
    }

    fn prep(&mut self, t: u16, o: &mut Vec<Out>) {
        let v = self.ver();
        match self.s.prepare_transition(t, v) {
            Ok(()) => {
                if t != 0 {
                    o.push(tid(t));
                }
            }
            Err(e) => {
                tracing::warn!("dave prepare: {e}");
                o.push(bad(t));
            }
        }
    }

    pub fn json(&mut self, op: u64, d: &Value) -> Vec<Out> {
        let mut o = Vec::new();
        let t = d["transition_id"].as_u64().unwrap_or(0) as u16;
        let v = d["protocol_version"].as_u64().unwrap_or(0) as u16;
        match op {
            21 => match self.s.prepare_transition(t, v) {
                Ok(()) => {
                    if t != 0 {
                        o.push(tid(t));
                    }
                }
                Err(e) => {
                    tracing::warn!("dave prepare: {e}");
                    o.push(bad(t));
                }
            },
            22 => {
                if let Err(e) = self.s.execute_transition(t) {
                    tracing::warn!("dave execute: {e}");
                }
            }
            24 => {
                if d["epoch"].as_u64() == Some(1) {
                    self.reset(v);
                }
            }
            _ => {}
        }
        o
    }

    pub fn enc(&mut self, p: &[u8]) -> Vec<u8> {
        if self.s.can_encrypt() {
            if let Ok(c) = self.s.encrypt_opus(p) {
                return c.into_owned();
            }
        }
        p.to_vec()
    }

    pub fn dec(&mut self, uid: u64, p: &[u8]) -> Option<Vec<u8>> {
        self.s.decrypt(uid, davey::MediaType::AUDIO, p).ok()
    }
}

#[cfg(not(feature = "dave"))]
pub struct Dv;

#[cfg(not(feature = "dave"))]
impl Dv {
    pub fn new(_: u16, _: u64, _: u64) -> Option<Dv> {
        None
    }
    pub fn ver(&self) -> u16 {
        0
    }
    pub fn ready(&self) -> bool {
        false
    }
    pub fn code(&self, _: u64) -> Option<String> {
        None
    }
    pub fn bin(&mut self, _: u8, _: &[u8]) -> Vec<Out> {
        vec![]
    }
    pub fn json(&mut self, _: u64, _: &Value) -> Vec<Out> {
        vec![]
    }
    pub fn enc(&mut self, p: &[u8]) -> Vec<u8> {
        p.to_vec()
    }
    pub fn dec(&mut self, _: u64, p: &[u8]) -> Option<Vec<u8>> {
        Some(p.to_vec())
    }
}

#[cfg(all(test, feature = "dave"))]
mod tests {
    use super::*;

    #[test]
    fn plain_until_ready() {
        let mut d = Dv::new(1, 111, 222).unwrap();
        assert!(!d.ready());
        assert_eq!(d.enc(b"abc"), b"abc");
        assert_eq!(d.ver(), 1);
    }

    #[test]
    fn garbage_is_survivable() {
        let mut d = Dv::new(1, 111, 222).unwrap();
        assert!(d.bin(25, &[1, 2, 3]).is_empty());
        assert!(d.bin(27, &[0, 9, 9]).is_empty());
        assert!(d.bin(29, &[0]).is_empty());
        assert!(d.json(22, &json!({"transition_id": 4})).is_empty());
        assert!(d.json(24, &json!({"protocol_version": 1, "epoch": 1})).is_empty());
        assert!(Dv::new(0, 1, 2).is_none());
    }

    #[test]
    fn rd() {
        assert_eq!(rd16(&[1, 2, 3]), Some((258, &[3u8][..])));
        assert!(rd16(&[1]).is_none());
    }
}
