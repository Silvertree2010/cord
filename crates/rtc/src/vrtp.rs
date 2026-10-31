use serde_json::{Value, json};

pub const PAY: usize = 1100;
pub const CLK: u64 = 90_000;
const MAXAU: usize = 16 << 20;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Codec {
    H264,
    Vp8,
    Av1,
}

impl Codec {
    pub fn name(self) -> &'static str {
        match self {
            Codec::H264 => "H264",
            Codec::Vp8 => "VP8",
            Codec::Av1 => "AV1",
        }
    }

    pub fn pt(self) -> u8 {
        match self {
            Codec::H264 => 101,
            Codec::Vp8 => 105,
            Codec::Av1 => 109,
        }
    }

    pub fn rtx(self) -> u8 {
        self.pt() + 1
    }

    pub fn all() -> [Codec; 3] {
        [Codec::H264, Codec::Vp8, Codec::Av1]
    }

    pub fn by_pt(pt: u8) -> Option<(Codec, bool)> {
        Codec::all().into_iter().find_map(|c| if c.pt() == pt { Some((c, false)) } else if c.rtx() == pt { Some((c, true)) } else { None })
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Q {
    pub w: u32,
    pub h: u32,
    pub fps: u32,
    pub kbps: u32,
}

pub fn ts90(pts_us: u64) -> u32 {
    (pts_us * CLK / 1_000_000) as u32
}

pub fn codecs() -> Value {
    let mut v = vec![];
    for (i, c) in Codec::all().into_iter().enumerate() {
        let mut o = json!({"name": c.name(), "type": "video", "priority": 1000 + i as u32 * 1000, "payload_type": c.pt(), "rtx_payload_type": c.rtx()});
        if c == Codec::H264 {
            o["encode"] = json!(true);
            o["decode"] = json!(true);
        }
        v.push(o);
    }
    v.insert(0, json!({"name": "opus", "type": "audio", "priority": 1000, "payload_type": 120}));
    Value::Array(v)
}

pub fn ident_streams() -> Value {
    json!([{"type": "screen", "rid": "100", "quality": 100}])
}

pub fn video_op(audio: u32, v: u32, rtx: u32, q: Option<Q>) -> Value {
    let s = match q {
        Some(q) => json!([{
            "type": "screen",
            "rid": "100",
            "ssrc": v,
            "rtx_ssrc": rtx,
            "active": true,
            "quality": 100,
            "max_bitrate": q.kbps as u64 * 1000,
            "max_framerate": q.fps,
            "max_resolution": {"type": "fixed", "width": q.w, "height": q.h}
        }]),
        None => json!([]),
    };
    json!({"op": 12, "d": {"audio_ssrc": audio, "video_ssrc": if q.is_some() { v } else { 0 }, "rtx_ssrc": if q.is_some() { rtx } else { 0 }, "streams": s}})
}

pub fn nals(b: &[u8]) -> Vec<&[u8]> {
    let mut idx = vec![];
    let mut i = 0;
    while i + 3 <= b.len() {
        if b[i] == 0 && b[i + 1] == 0 && b[i + 2] == 1 {
            idx.push(i + 3);
            i += 3;
        } else {
            i += 1;
        }
    }
    let mut o = vec![];
    for (k, &s) in idx.iter().enumerate() {
        let mut e = idx.get(k + 1).map(|n| n - 3).unwrap_or(b.len());
        while e > s && b[e - 1] == 0 {
            e -= 1;
        }
        if e > s {
            o.push(&b[s..e]);
        }
    }
    o
}

pub fn norm4(au: &[u8]) -> Vec<u8> {
    let mut o = Vec::with_capacity(au.len() + 8);
    for n in nals(au) {
        o.extend_from_slice(&[0, 0, 0, 1]);
        o.extend_from_slice(n);
    }
    o
}

pub fn is_key(au: &[u8]) -> bool {
    nals(au).iter().any(|n| n[0] & 31 == 5)
}

fn stap(g: &[&[u8]]) -> Vec<u8> {
    if g.len() == 1 {
        return g[0].to_vec();
    }
    let nri = g.iter().map(|n| n[0] & 0x60).max().unwrap_or(0);
    let mut o = vec![nri | 24];
    for n in g {
        o.extend_from_slice(&(n.len() as u16).to_be_bytes());
        o.extend_from_slice(n);
    }
    o
}

pub fn pack_h264(au: &[u8]) -> Vec<Vec<u8>> {
    let mut o = vec![];
    let mut g: Vec<&[u8]> = vec![];
    let mut sz = 1;
    for n in nals(au) {
        if n.len() <= PAY {
            if sz + 2 + n.len() > PAY && !g.is_empty() {
                o.push(stap(&g));
                g.clear();
                sz = 1;
            }
            sz += 2 + n.len();
            g.push(n);
            continue;
        }
        if !g.is_empty() {
            o.push(stap(&g));
            g.clear();
            sz = 1;
        }
        let ind = (n[0] & 0xe0) | 28;
        let ty = n[0] & 31;
        let body = &n[1..];
        let cs = PAY - 2;
        let cnt = body.len().div_ceil(cs);
        for (i, c) in body.chunks(cs).enumerate() {
            let mut h = ty;
            if i == 0 {
                h |= 0x80;
            }
            if i + 1 == cnt {
                h |= 0x40;
            }
            let mut p = vec![ind, h];
            p.extend_from_slice(c);
            o.push(p);
        }
    }
    if !g.is_empty() {
        o.push(stap(&g));
    }
    o
}

#[derive(Default)]
pub struct Pu {
    pub au: Option<(Vec<u8>, bool)>,
    pub lost: bool,
}

#[derive(Default)]
struct Base {
    last: Option<u16>,
    ts: Option<u32>,
    buf: Vec<u8>,
    bad: bool,
    key: bool,
}

impl Base {
    fn gap(&mut self, seq: u16, ts: u32) -> bool {
        let mut lost = false;
        let same = self.ts == Some(ts);
        if let Some(l) = self.last
            && seq != l.wrapping_add(1)
        {
            lost = true;
            if same {
                self.bad = true;
            }
        }
        self.last = Some(seq);
        if !same {
            if !self.buf.is_empty() {
                lost = true;
            }
            self.buf.clear();
            self.key = false;
            self.bad = false;
            self.ts = Some(ts);
        }
        lost
    }

    fn fin(&mut self, marker: bool, lost: bool) -> Pu {
        if self.buf.len() > MAXAU {
            self.buf.clear();
            self.bad = true;
        }
        if marker && !self.bad && !self.buf.is_empty() {
            let b = std::mem::take(&mut self.buf);
            self.ts = None;
            return Pu { au: Some((b, self.key)), lost };
        }
        if marker {
            self.buf.clear();
            self.ts = None;
        }
        Pu { au: None, lost }
    }
}

#[derive(Default)]
pub struct H264u {
    b: Base,
    fu: bool,
}

const SC: [u8; 4] = [0, 0, 0, 1];

impl H264u {
    pub fn push(&mut self, seq: u16, ts: u32, marker: bool, p: &[u8]) -> Pu {
        let lost = self.b.gap(seq, ts);
        if lost {
            self.fu = false;
        }
        if p.is_empty() || self.b.bad {
            return self.b.fin(marker, lost);
        }
        match p[0] & 31 {
            24 => {
                let mut i = 1;
                while i + 2 <= p.len() {
                    let n = u16::from_be_bytes([p[i], p[i + 1]]) as usize;
                    i += 2;
                    if n == 0 || i + n > p.len() {
                        break;
                    }
                    self.nal(&p[i..i + n]);
                    i += n;
                }
            }
            28 => {
                if p.len() >= 3 {
                    let (s, e, ty) = (p[1] & 0x80 != 0, p[1] & 0x40 != 0, p[1] & 31);
                    if s {
                        self.b.buf.extend_from_slice(&SC);
                        self.b.buf.push((p[0] & 0xe0) | ty);
                        self.fu = true;
                        if ty == 5 {
                            self.b.key = true;
                        }
                    }
                    if self.fu {
                        self.b.buf.extend_from_slice(&p[2..]);
                    } else {
                        self.b.bad = true;
                    }
                    if e {
                        self.fu = false;
                    }
                }
            }
            1..=23 => self.nal(p),
            _ => {}
        }
        self.b.fin(marker, lost)
    }

    fn nal(&mut self, n: &[u8]) {
        if n[0] & 31 == 5 {
            self.b.key = true;
        }
        self.b.buf.extend_from_slice(&SC);
        self.b.buf.extend_from_slice(n);
    }
}

#[derive(Default)]
pub struct Vp8u {
    b: Base,
}

impl Vp8u {
    pub fn push(&mut self, seq: u16, ts: u32, marker: bool, p: &[u8]) -> Pu {
        let lost = self.b.gap(seq, ts);
        if p.is_empty() || self.b.bad {
            return self.b.fin(marker, lost);
        }
        let mut i = 1;
        if p[0] & 0x80 != 0 && p.len() > 1 {
            let x = p[1];
            i = 2;
            if x & 0x80 != 0 {
                if p.get(i).is_some_and(|b| b & 0x80 != 0) {
                    i += 1;
                }
                i += 1;
            }
            if x & 0x40 != 0 {
                i += 1;
            }
            if x & 0x30 != 0 {
                i += 1;
            }
        }
        if i > p.len() {
            return self.b.fin(marker, lost);
        }
        if p[0] & 0x10 != 0 && p[0] & 7 == 0 {
            self.b.buf.clear();
            self.b.key = p.get(i).is_some_and(|b| b & 1 == 0);
        }
        self.b.buf.extend_from_slice(&p[i..]);
        self.b.fin(marker, lost)
    }
}

fn leb(mut v: usize) -> Vec<u8> {
    let mut o = vec![];
    loop {
        let b = (v & 0x7f) as u8;
        v >>= 7;
        if v == 0 {
            o.push(b);
            return o;
        }
        o.push(b | 0x80);
    }
}

fn rleb(b: &[u8]) -> Option<(usize, usize)> {
    let mut v = 0usize;
    for (i, &x) in b.iter().take(8).enumerate() {
        v |= ((x & 0x7f) as usize) << (7 * i);
        if x & 0x80 == 0 {
            return Some((v, i + 1));
        }
    }
    None
}

fn obu(o: &[u8]) -> Vec<u8> {
    if o.is_empty() || o[0] & 2 != 0 {
        return o.to_vec();
    }
    let h = if o[0] & 4 != 0 { 2 } else { 1 };
    if o.len() < h {
        return vec![];
    }
    let mut r = vec![o[0] | 2];
    r.extend_from_slice(&o[1..h]);
    r.extend(leb(o.len() - h));
    r.extend_from_slice(&o[h..]);
    r
}

#[derive(Default)]
pub struct Av1u {
    b: Base,
    part: Vec<u8>,
}

impl Av1u {
    pub fn push(&mut self, seq: u16, ts: u32, marker: bool, p: &[u8]) -> Pu {
        let lost = self.b.gap(seq, ts);
        if lost {
            self.part.clear();
        }
        if p.len() < 2 || self.b.bad {
            return self.b.fin(marker, lost);
        }
        let (z, y, w, n) = (p[0] & 0x80 != 0, p[0] & 0x40 != 0, (p[0] >> 4) & 3, p[0] & 8 != 0);
        if n {
            self.b.key = true;
        }
        let mut els: Vec<&[u8]> = vec![];
        let mut i = 1;
        while i < p.len() {
            if w != 0 && els.len() + 1 == w as usize {
                els.push(&p[i..]);
                break;
            }
            let Some((l, k)) = rleb(&p[i..]) else { break };
            i += k;
            if i + l > p.len() {
                break;
            }
            els.push(&p[i..i + l]);
            i += l;
        }
        let cnt = els.len();
        if self.part.len() > MAXAU {
            self.part.clear();
            self.b.bad = true;
        }
        for (k, e) in els.into_iter().enumerate() {
            let first = k == 0 && z;
            let last = k + 1 == cnt && y;
            if first {
                self.part.extend_from_slice(e);
            } else {
                self.part = e.to_vec();
            }
            if !last {
                let o = obu(&std::mem::take(&mut self.part));
                self.b.buf.extend(o);
            }
        }
        self.b.fin(marker, lost)
    }
}

pub enum Unp {
    H(H264u),
    V(Vp8u),
    A(Av1u),
}

impl Unp {
    pub fn new(c: Codec) -> Unp {
        match c {
            Codec::H264 => Unp::H(Default::default()),
            Codec::Vp8 => Unp::V(Default::default()),
            Codec::Av1 => Unp::A(Default::default()),
        }
    }

    pub fn push(&mut self, seq: u16, ts: u32, marker: bool, p: &[u8]) -> Pu {
        match self {
            Unp::H(u) => u.push(seq, ts, marker, p),
            Unp::V(u) => u.push(seq, ts, marker, p),
            Unp::A(u) => u.push(seq, ts, marker, p),
        }
    }
}

pub fn pli(me: u32, media: u32) -> Vec<u8> {
    let mut o = vec![0x81, 206, 0, 2];
    o.extend_from_slice(&me.to_be_bytes());
    o.extend_from_slice(&media.to_be_bytes());
    o
}

pub fn fir(me: u32, media: u32, n: u8) -> Vec<u8> {
    let mut o = vec![0x84, 206, 0, 4];
    o.extend_from_slice(&me.to_be_bytes());
    o.extend_from_slice(&0u32.to_be_bytes());
    o.extend_from_slice(&media.to_be_bytes());
    o.extend_from_slice(&[n, 0, 0, 0]);
    o
}

pub fn nack(me: u32, media: u32, seqs: &[u16]) -> Vec<u8> {
    let mut f: Vec<(u16, u16)> = vec![];
    for &s in seqs {
        match f.last_mut() {
            Some((p, b)) if s.wrapping_sub(*p) >= 1 && s.wrapping_sub(*p) <= 16 => *b |= 1 << (s.wrapping_sub(*p) - 1),
            _ => f.push((s, 0)),
        }
    }
    let mut o = vec![0x81, 205];
    o.extend_from_slice(&((2 + f.len()) as u16).to_be_bytes());
    o.extend_from_slice(&me.to_be_bytes());
    o.extend_from_slice(&media.to_be_bytes());
    for (p, b) in f {
        o.extend_from_slice(&p.to_be_bytes());
        o.extend_from_slice(&b.to_be_bytes());
    }
    o
}

#[derive(Debug, PartialEq, Eq)]
pub enum Fb {
    Pli(u32),
    Fir(u32),
    Nack(u32, Vec<u16>),
}

pub fn parse_fb(mut b: &[u8]) -> Vec<Fb> {
    let mut o = vec![];
    while b.len() >= 8 {
        let n = (u16::from_be_bytes([b[2], b[3]]) as usize + 1) * 4;
        if n > b.len() {
            break;
        }
        let (fmt, pt, p) = (b[0] & 31, b[1], &b[..n]);
        let u = |i: usize| u32::from_be_bytes([p[i], p[i + 1], p[i + 2], p[i + 3]]);
        match (pt, fmt) {
            (206, 1) if n >= 12 => o.push(Fb::Pli(u(8))),
            (206, 4) if n >= 16 => o.push(Fb::Fir(u(12))),
            (205, 1) if n >= 16 => {
                let mut s = vec![];
                let mut i = 12;
                while i + 4 <= n {
                    let pid = u16::from_be_bytes([p[i], p[i + 1]]);
                    let blp = u16::from_be_bytes([p[i + 2], p[i + 3]]);
                    s.push(pid);
                    for k in 0..16 {
                        if blp & (1 << k) != 0 {
                            s.push(pid.wrapping_add(k + 1));
                        }
                    }
                    i += 4;
                }
                o.push(Fb::Nack(u(8), s));
            }
            _ => {}
        }
        b = &b[n..];
    }
    o
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn oversize_au_dropped() {
        let mut u = H264u::default();
        let big = vec![0x41u8; 1000];
        for i in 0..(MAXAU / 1000 + 5) {
            assert!(u.push(i as u16, 1, false, &big).au.is_none());
        }
        assert!(u.push(60000, 1, true, &big).au.is_none());
    }

    fn au(sizes: &[usize]) -> Vec<u8> {
        let mut a = vec![];
        for (i, &s) in sizes.iter().enumerate() {
            let ty = [7u8, 8, 5, 1][i.min(3)];
            a.extend_from_slice(&[0, 0, 0, 1, 0x60 | ty]);
            a.extend((0..s).map(|k| (k % 250) as u8 + 1));
        }
        a
    }

    fn rt(a: &[u8]) -> (Vec<Vec<u8>>, Option<(Vec<u8>, bool)>) {
        let ps = pack_h264(a);
        let mut u = H264u::default();
        let mut r = None;
        for (i, p) in ps.iter().enumerate() {
            assert!(p.len() <= PAY + 2);
            let o = u.push(i as u16, 9000, i + 1 == ps.len(), p);
            assert!(!o.lost);
            if o.au.is_some() {
                assert_eq!(i + 1, ps.len());
                r = o.au;
            }
        }
        (ps, r)
    }

    #[test]
    fn small_stap() {
        let a = au(&[10, 4, 300, 50]);
        let (ps, r) = rt(&a);
        assert_eq!(ps.len(), 1);
        assert_eq!(ps[0][0] & 31, 24);
        assert_eq!(ps[0][0] & 0x60, 0x60);
        assert_eq!(r.unwrap(), (a, true));
    }

    #[test]
    fn single_nal() {
        let a = au(&[0x1f - 1]);
        let (ps, r) = rt(&a);
        assert_eq!(ps.len(), 1);
        assert_eq!(ps[0][0] & 31, 7);
        assert_eq!(r.unwrap().0, a);
    }

    #[test]
    fn fua_roundtrip() {
        let a = au(&[10, 4, 5000, 3000]);
        let (ps, r) = rt(&a);
        assert!(ps.len() > 5);
        let fu: Vec<_> = ps.iter().filter(|p| p[0] & 31 == 28).collect();
        assert!(fu[0][1] & 0x80 != 0);
        assert!(fu.iter().any(|p| p[1] & 0x40 != 0));
        assert_eq!(fu[0][0] & 0xe0, 0x60);
        let (b, k) = r.unwrap();
        assert!(k);
        assert_eq!(b, a);
    }

    #[test]
    fn loss_detected() {
        let a = au(&[10, 4, 5000]);
        let ps = pack_h264(&a);
        let mut u = H264u::default();
        let mut lost = false;
        let mut got = false;
        for (i, p) in ps.iter().enumerate() {
            if i == 2 {
                continue;
            }
            let o = u.push(i as u16, 1, i + 1 == ps.len(), p);
            lost |= o.lost;
            got |= o.au.is_some();
        }
        assert!(lost);
        assert!(!got);
        let b = au(&[10, 4, 20]);
        let o = u.push(100, 2, true, &pack_h264(&b)[0]);
        assert!(o.lost);
        let o = u.push(101, 3, true, &pack_h264(&b)[0]);
        assert!(o.au.is_some());
    }

    #[test]
    fn nal_split_with_zeros() {
        let a = [0, 0, 1, 0x65, 1, 0, 0, 0, 1, 0x41, 2];
        assert_eq!(nals(&a), vec![&[0x65, 1][..], &[0x41, 2][..]]);
        assert!(is_key(&a));
    }

    #[test]
    fn vp8_frame() {
        let mut u = Vp8u::default();
        let o = u.push(1, 5, false, &[0x10, 0x00, 1, 2]);
        assert!(o.au.is_none());
        let o = u.push(2, 5, true, &[0x00, 3, 4]);
        assert_eq!(o.au.unwrap(), (vec![0, 1, 2, 3, 4], true));
        let mut u = Vp8u::default();
        let o = u.push(1, 5, true, &[0x90, 0x80, 0x12, 0x01, 1, 2]);
        assert_eq!(o.au.unwrap(), (vec![1, 1, 2], false));
    }

    #[test]
    fn av1_frame() {
        let mut u = Av1u::default();
        let o = u.push(1, 1, true, &[0x28, 1, 0x10, 0x30, 7, 7]);
        let (b, k) = o.au.unwrap();
        assert!(k);
        assert_eq!(b, vec![0x12, 0, 0x32, 2, 7, 7]);
    }

    #[test]
    fn rtcp_fb() {
        assert_eq!(parse_fb(&pli(1, 2)), vec![Fb::Pli(2)]);
        assert_eq!(parse_fb(&fir(1, 2, 7)), vec![Fb::Fir(2)]);
        let n = nack(1, 2, &[10, 11, 14, 100]);
        assert_eq!(parse_fb(&n), vec![Fb::Nack(2, vec![10, 11, 14, 100])]);
        let mut c = pli(1, 2);
        c.extend(nack(1, 2, &[5]));
        assert_eq!(parse_fb(&c).len(), 2);
    }

    #[test]
    fn descriptors() {
        let v = video_op(10, 11, 12, Some(Q { w: 1920, h: 1080, fps: 60, kbps: 8000 }));
        assert_eq!(v["op"], 12);
        let s = &v["d"]["streams"][0];
        assert_eq!(s["max_resolution"]["width"], 1920);
        assert_eq!(s["max_framerate"], 60);
        assert_eq!(s["max_bitrate"], 8_000_000);
        assert_eq!(s["rtx_ssrc"], 12);
        assert_eq!(v["d"]["video_ssrc"], 11);
        let z = video_op(10, 0, 0, None);
        assert_eq!(z["d"]["streams"].as_array().unwrap().len(), 0);
        let c = codecs();
        assert!(c.as_array().unwrap().iter().any(|x| x["name"] == "H264" && x["payload_type"] == 101 && x["rtx_payload_type"] == 102));
        assert_eq!(Codec::by_pt(106), Some((Codec::Vp8, true)));
        assert_eq!(ts90(1_000_000), 90_000);
    }
}
