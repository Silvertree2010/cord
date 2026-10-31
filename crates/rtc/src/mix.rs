use crate::jb::{Jb, Pop};
use opus::{Channels, Decoder};
use std::{collections::HashMap, sync::Arc};

pub const SR: u32 = 48000;
pub const FR: usize = 960;
pub const FS: usize = FR * 2;
pub const SIL: [u8; 3] = [0xF8, 0xFF, 0xFE];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Spk {
    pub ssrc: Option<u32>,
    pub on: bool,
}

pub type Cb = Arc<dyn Fn(Spk) + Send + Sync>;

struct Us {
    jb: Jb,
    dec: Decoder,
    act: bool,
    idle: u32,
}

pub struct Mix {
    us: HashMap<u32, Us>,
    vol: HashMap<u32, f32>,
    pub deaf: bool,
    pub cb: Option<Cb>,
    buf: Vec<f32>,
    depth: usize,
}

impl Default for Mix {
    fn default() -> Self {
        Self::new()
    }
}

impl Mix {
    pub fn new() -> Mix {
        Mix { us: HashMap::new(), vol: HashMap::new(), deaf: false, cb: None, buf: vec![0.0; FS], depth: 3 }
    }

    pub fn set_depth(&mut self, d: usize) {
        self.depth = d.max(1);
    }

    pub fn set_vol(&mut self, ssrc: u32, v: f32) {
        self.vol.insert(ssrc, v.clamp(0.0, 5.0));
    }

    pub fn vol(&self, ssrc: u32) -> f32 {
        self.vol.get(&ssrc).copied().unwrap_or(1.0)
    }

    pub fn drop_ssrc(&mut self, ssrc: u32) {
        if let Some(u) = self.us.remove(&ssrc)
            && u.act
        {
            self.emit(ssrc, false);
        }
    }

    pub fn clear(&mut self) {
        let k: Vec<u32> = self.us.keys().copied().collect();
        for s in k {
            self.drop_ssrc(s);
        }
    }

    pub fn push(&mut self, ssrc: u32, seq: u16, d: Vec<u8>) {
        if self.deaf {
            return;
        }
        if !self.us.contains_key(&ssrc) {
            let Ok(dec) = Decoder::new(SR, Channels::Stereo) else { return };
            self.us.insert(ssrc, Us { jb: Jb::new(self.depth, 50), dec, act: false, idle: 0 });
        }
        if let Some(u) = self.us.get_mut(&ssrc) {
            u.jb.push(seq, d);
        }
    }

    fn emit(&self, ssrc: u32, on: bool) {
        if let Some(c) = &self.cb {
            c(Spk { ssrc: Some(ssrc), on });
        }
    }

    pub fn pull(&mut self, out: &mut [f32]) {
        out.fill(0.0);
        let mut ev = Vec::new();
        let mut dead = Vec::new();
        for (&ssrc, u) in self.us.iter_mut() {
            let v = self.vol.get(&ssrc).copied().unwrap_or(1.0);
            let r = match u.jb.pop() {
                Pop::Pkt(d) => {
                    u.idle = 0;
                    let on = d != SIL;
                    if on != u.act {
                        u.act = on;
                        ev.push((ssrc, on));
                    }
                    u.dec.decode_float(&d, &mut self.buf, false)
                }
                Pop::Lost => u.dec.decode_float(&[], &mut self.buf, false),
                Pop::Empty => {
                    u.idle += 1;
                    if u.idle == 10 && u.act {
                        u.act = false;
                        ev.push((ssrc, false));
                    }
                    if u.idle > 500 {
                        dead.push(ssrc);
                    }
                    continue;
                }
            };
            if r.is_ok() {
                for (o, s) in out.iter_mut().zip(self.buf.iter()) {
                    *o += s * v;
                }
            }
        }
        for s in dead {
            self.us.remove(&s);
        }
        for (s, on) in ev {
            self.emit(s, on);
        }
        for o in out.iter_mut() {
            *o = o.clamp(-1.0, 1.0);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use opus::{Application, Encoder};

    fn tone(a: f32) -> Vec<f32> {
        (0..FR).flat_map(|i| {
            let s = (i as f32 * 440.0 * std::f32::consts::TAU / SR as f32).sin() * a;
            [s, s]
        }).collect()
    }

    fn en() -> Encoder {
        Encoder::new(SR, Channels::Stereo, Application::Voip).unwrap()
    }

    fn rms(a: &[f32]) -> f32 {
        (a.iter().map(|x| x * x).sum::<f32>() / a.len() as f32).sqrt()
    }

    #[test]
    fn decode_and_vol() {
        let mut e = en();
        let mut m = Mix::new();
        for s in 0..8u16 {
            let p = e.encode_vec_float(&tone(0.5), 1000).unwrap();
            m.push(7, s, p);
        }
        let mut o = vec![0.0; FS];
        let mut peak = 0.0f32;
        for _ in 0..8 {
            m.pull(&mut o);
            peak = peak.max(rms(&o));
        }
        assert!(peak > 0.1, "{peak}");
        m.set_vol(7, 0.0);
        m.pull(&mut o);
        assert_eq!(rms(&o), 0.0);
    }

    #[test]
    fn speak_events() {
        let log = Arc::new(std::sync::Mutex::new(vec![]));
        let l2 = log.clone();
        let mut m = Mix::new();
        m.cb = Some(Arc::new(move |s| l2.lock().unwrap().push(s)));
        let mut e = en();
        for s in 0..4u16 {
            m.push(9, s, e.encode_vec_float(&tone(0.3), 1000).unwrap());
        }
        m.push(9, 4, SIL.to_vec());
        let mut o = vec![0.0; FS];
        for _ in 0..6 {
            m.pull(&mut o);
        }
        let l = log.lock().unwrap();
        assert_eq!(*l, vec![Spk { ssrc: Some(9), on: true }, Spk { ssrc: Some(9), on: false }]);
    }

    #[test]
    fn loss_conceals() {
        let mut e = en();
        let mut m = Mix::new();
        for s in [0u16, 1, 3, 4, 5] {
            m.push(1, s, e.encode_vec_float(&tone(0.5), 1000).unwrap());
        }
        let mut o = vec![0.0; FS];
        let mut n = 0;
        for _ in 0..6 {
            m.pull(&mut o);
            if rms(&o) > 0.01 {
                n += 1;
            }
        }
        assert!(n >= 5);
    }

    #[test]
    fn deaf_drops() {
        let mut m = Mix::new();
        m.deaf = true;
        m.push(1, 0, vec![1, 2, 3]);
        assert!(m.us.is_empty());
    }
}
