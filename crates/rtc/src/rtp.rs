use aes_gcm::{
    Aes256Gcm,
    aead::{Aead, KeyInit, Payload},
};
use chacha20poly1305::XChaCha20Poly1305;

pub const PT: u8 = 0x78;
pub const HL: usize = 12;
pub const TAG: usize = 16;
pub const CTR: usize = 4;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Hdr {
    pub cc: u8,
    pub ext: bool,
    pub pt: u8,
    pub seq: u16,
    pub ts: u32,
    pub ssrc: u32,
}

impl Hdr {
    pub fn pack(seq: u16, ts: u32, ssrc: u32) -> [u8; HL] {
        let mut h = [0u8; HL];
        h[0] = 0x80;
        h[1] = PT;
        h[2..4].copy_from_slice(&seq.to_be_bytes());
        h[4..8].copy_from_slice(&ts.to_be_bytes());
        h[8..12].copy_from_slice(&ssrc.to_be_bytes());
        h
    }

    pub fn pack_ex(pt: u8, marker: bool, seq: u16, ts: u32, ssrc: u32) -> [u8; HL] {
        let mut h = Hdr::pack(seq, ts, ssrc);
        h[1] = (pt & 0x7f) | if marker { 0x80 } else { 0 };
        h
    }

    pub fn parse(b: &[u8]) -> Option<(Hdr, usize)> {
        if b.len() < HL || b[0] >> 6 != 2 {
            return None;
        }
        let cc = b[0] & 0xf;
        let ext = b[0] & 0x10 != 0;
        let n = HL + cc as usize * 4 + if ext { 4 } else { 0 };
        if b.len() < n {
            return None;
        }
        let h = Hdr {
            cc,
            ext,
            pt: b[1] & 0x7f,
            seq: u16::from_be_bytes([b[2], b[3]]),
            ts: u32::from_be_bytes([b[4], b[5], b[6], b[7]]),
            ssrc: u32::from_be_bytes([b[8], b[9], b[10], b[11]]),
        };
        Some((h, n))
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Mode {
    Aes,
    Xcha,
}

impl Mode {
    pub fn name(self) -> &'static str {
        match self {
            Mode::Aes => "aead_aes256_gcm_rtpsize",
            Mode::Xcha => "aead_xchacha20_poly1305_rtpsize",
        }
    }

    pub fn from_name(s: &str) -> Option<Mode> {
        match s {
            "aead_aes256_gcm_rtpsize" => Some(Mode::Aes),
            "aead_xchacha20_poly1305_rtpsize" => Some(Mode::Xcha),
            _ => None,
        }
    }

    pub fn pick(avail: &[String]) -> Option<Mode> {
        let has = |m: Mode| avail.iter().any(|s| s == m.name());
        if hw_aes() && has(Mode::Aes) {
            Some(Mode::Aes)
        } else if has(Mode::Xcha) {
            Some(Mode::Xcha)
        } else if has(Mode::Aes) {
            Some(Mode::Aes)
        } else {
            None
        }
    }
}

fn hw_aes() -> bool {
    #[cfg(target_arch = "x86_64")]
    {
        std::arch::is_x86_feature_detected!("aes") && std::arch::is_x86_feature_detected!("pclmulqdq")
    }
    #[cfg(target_arch = "aarch64")]
    {
        std::arch::is_aarch64_feature_detected!("aes")
    }
    #[cfg(not(any(target_arch = "x86_64", target_arch = "aarch64")))]
    {
        false
    }
}

pub struct Cry {
    pub mode: Mode,
    key: [u8; 32],
    ctr: u32,
}

impl Cry {
    pub fn new(mode: Mode, key: &[u8]) -> Option<Cry> {
        Some(Cry { mode, key: key.try_into().ok()?, ctr: 0 })
    }

    fn run(&self, enc: bool, ctr: [u8; CTR], aad: &[u8], msg: &[u8]) -> Option<Vec<u8>> {
        let p = Payload { msg, aad };
        match self.mode {
            Mode::Aes => {
                let c = Aes256Gcm::new((&self.key).into());
                let mut n = [0u8; 12];
                n[..CTR].copy_from_slice(&ctr);
                if enc { c.encrypt((&n).into(), p).ok() } else { c.decrypt((&n).into(), p).ok() }
            }
            Mode::Xcha => {
                let c = XChaCha20Poly1305::new((&self.key).into());
                let mut n = [0u8; 24];
                n[..CTR].copy_from_slice(&ctr);
                if enc { c.encrypt((&n).into(), p).ok() } else { c.decrypt((&n).into(), p).ok() }
            }
        }
    }

    pub fn seal(&mut self, hdr: &[u8; HL], payload: &[u8]) -> Vec<u8> {
        let ctr = self.ctr.to_be_bytes();
        self.ctr = self.ctr.wrapping_add(1);
        let ct = self.run(true, ctr, hdr, payload).unwrap_or_default();
        let mut o = Vec::with_capacity(HL + ct.len() + CTR);
        o.extend_from_slice(hdr);
        o.extend_from_slice(&ct);
        o.extend_from_slice(&ctr);
        o
    }

    pub fn seal_rtcp(&mut self, pkt: &[u8]) -> Vec<u8> {
        if pkt.len() < 8 {
            return vec![];
        }
        let ctr = self.ctr.to_be_bytes();
        self.ctr = self.ctr.wrapping_add(1);
        let ct = self.run(true, ctr, &pkt[..8], &pkt[8..]).unwrap_or_default();
        let mut o = Vec::with_capacity(8 + ct.len() + CTR);
        o.extend_from_slice(&pkt[..8]);
        o.extend_from_slice(&ct);
        o.extend_from_slice(&ctr);
        o
    }

    pub fn open_rtcp(&self, pkt: &[u8]) -> Option<Vec<u8>> {
        if pkt.len() < 8 + TAG + CTR {
            return None;
        }
        let e = pkt.len() - CTR;
        let ctr: [u8; CTR] = pkt[e..].try_into().ok()?;
        let pl = self.run(false, ctr, &pkt[..8], &pkt[8..e])?;
        let mut o = pkt[..8].to_vec();
        o.extend_from_slice(&pl);
        Some(o)
    }

    pub fn open(&self, pkt: &[u8]) -> Option<(Hdr, Vec<u8>)> {
        let (h, n) = Hdr::parse(pkt)?;
        if pkt.len() < n + TAG + CTR {
            return None;
        }
        let e = pkt.len() - CTR;
        let ctr: [u8; CTR] = pkt[e..].try_into().ok()?;
        let mut pl = self.run(false, ctr, &pkt[..n], &pkt[n..e])?;
        if h.ext {
            let w = u16::from_be_bytes([pkt[n - 2], pkt[n - 1]]) as usize * 4;
            if pl.len() < w {
                return None;
            }
            pl.drain(..w);
        }
        Some((h, pl))
    }
}

pub fn disc_req(ssrc: u32) -> [u8; 74] {
    let mut b = [0u8; 74];
    b[0..2].copy_from_slice(&1u16.to_be_bytes());
    b[2..4].copy_from_slice(&70u16.to_be_bytes());
    b[4..8].copy_from_slice(&ssrc.to_be_bytes());
    b
}

pub fn disc_res(b: &[u8]) -> Option<(String, u16)> {
    if b.len() < 74 || u16::from_be_bytes([b[0], b[1]]) != 2 {
        return None;
    }
    let a = &b[8..72];
    let e = a.iter().position(|&x| x == 0).unwrap_or(a.len());
    let ip = std::str::from_utf8(&a[..e]).ok()?.to_string();
    let port = u16::from_be_bytes([b[72], b[73]]);
    Some((ip, port))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hdr_round() {
        let b = Hdr::pack(0xabcd, 0x01020304, 0xdeadbeef);
        let (h, n) = Hdr::parse(&b).unwrap();
        assert_eq!(n, 12);
        assert_eq!((h.seq, h.ts, h.ssrc, h.pt), (0xabcd, 0x01020304, 0xdeadbeef, PT));
        assert_eq!(b[0], 0x80);
        assert_eq!(b[1], 0x78);
        assert!(Hdr::parse(&b[..11]).is_none());
        let mut bad = b;
        bad[0] = 0x40;
        assert!(Hdr::parse(&bad).is_none());
    }

    #[test]
    fn hdr_ext_len() {
        let mut b = Hdr::pack(1, 2, 3).to_vec();
        b[0] |= 0x10 | 2;
        b.extend_from_slice(&[0; 8]);
        b.extend_from_slice(&[0xbe, 0xde, 0, 1]);
        let (h, n) = Hdr::parse(&b).unwrap();
        assert!(h.ext);
        assert_eq!(h.cc, 2);
        assert_eq!(n, 12 + 8 + 4);
    }

    fn rt(m: Mode) {
        let key = [7u8; 32];
        let mut tx = Cry::new(m, &key).unwrap();
        let rx = Cry::new(m, &key).unwrap();
        let hd = Hdr::pack(5, 960, 42);
        for i in 0..3u8 {
            let pl = vec![i; 40 + i as usize];
            let pk = tx.seal(&hd, &pl);
            assert_eq!(pk.len(), 12 + pl.len() + TAG + CTR);
            let (h, out) = rx.open(&pk).unwrap();
            assert_eq!(h.ssrc, 42);
            assert_eq!(out, pl);
            let mut t = pk.clone();
            t[13] ^= 1;
            assert!(rx.open(&t).is_none());
            let mut t = pk.clone();
            t[2] ^= 1;
            assert!(rx.open(&t).is_none());
        }
        let wrong = Cry::new(m, &[8u8; 32]).unwrap();
        assert!(wrong.open(&tx.seal(&hd, b"hi")).is_none());
    }

    #[test]
    fn aes_round() {
        rt(Mode::Aes);
    }

    #[test]
    fn xcha_round() {
        rt(Mode::Xcha);
    }

    #[test]
    fn ctr_incr() {
        let mut c = Cry::new(Mode::Aes, &[1; 32]).unwrap();
        let hd = Hdr::pack(0, 0, 0);
        let a = c.seal(&hd, b"x");
        let b = c.seal(&hd, b"x");
        assert_eq!(&a[a.len() - 4..], &[0, 0, 0, 0]);
        assert_eq!(&b[b.len() - 4..], &[0, 0, 0, 1]);
        assert_ne!(a, b);
    }

    #[test]
    fn ext_stripped() {
        let key = [3u8; 32];
        let mut tx = Cry::new(Mode::Xcha, &key).unwrap();
        let mut hd = Hdr::pack(1, 2, 3).to_vec();
        hd[0] |= 0x10;
        hd.extend_from_slice(&[0xbe, 0xde, 0, 2]);
        let mut pl = vec![9u8; 8];
        pl.extend_from_slice(b"opus");
        let ctr = tx.ctr.to_be_bytes();
        tx.ctr += 1;
        let ct = tx.run(true, ctr, &hd, &pl).unwrap();
        let mut pk = hd.clone();
        pk.extend_from_slice(&ct);
        pk.extend_from_slice(&ctr);
        let (_, out) = tx.open(&pk).unwrap();
        assert_eq!(out, b"opus");
    }

    #[test]
    fn rtcp_round() {
        let mut tx = Cry::new(Mode::Aes, &[4; 32]).unwrap();
        let rx = Cry::new(Mode::Aes, &[4; 32]).unwrap();
        let p = [0x81, 206, 0, 2, 0, 0, 0, 1, 0, 0, 0, 2];
        let s = tx.seal_rtcp(&p);
        assert_eq!(&s[..8], &p[..8]);
        assert_eq!(rx.open_rtcp(&s).unwrap(), p);
        let mut t = s.clone();
        t[3] ^= 1;
        assert!(rx.open_rtcp(&t).is_none());
    }

    #[test]
    fn marker_bit() {
        let h = Hdr::pack_ex(101, true, 1, 2, 3);
        assert_eq!(h[1], 0x80 | 101);
        let (p, _) = Hdr::parse(&h).unwrap();
        assert_eq!(p.pt, 101);
    }

    #[test]
    fn disc() {
        let r = disc_req(9);
        assert_eq!(r.len(), 74);
        assert_eq!(&r[..4], &[0, 1, 0, 70]);
        let mut p = [0u8; 74];
        p[1] = 2;
        p[8..8 + 9].copy_from_slice(b"1.2.3.4\0x");
        p[72..74].copy_from_slice(&5555u16.to_be_bytes());
        assert_eq!(disc_res(&p), Some(("1.2.3.4".into(), 5555)));
    }

    #[test]
    fn pick() {
        let v = |a: &[&str]| a.iter().map(|s| s.to_string()).collect::<Vec<_>>();
        assert_eq!(Mode::pick(&v(&["xsalsa20_poly1305", "aead_xchacha20_poly1305_rtpsize"])), Some(Mode::Xcha));
        assert_eq!(Mode::pick(&v(&["xsalsa20_poly1305"])), None);
        assert!(Mode::pick(&v(&["aead_aes256_gcm_rtpsize", "aead_xchacha20_poly1305_rtpsize"])).is_some());
    }
}
