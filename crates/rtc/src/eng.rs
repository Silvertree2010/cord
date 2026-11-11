use crate::mix::{Cb, FR, FS, Mix, SIL, SR, Spk};
use anyhow::{Result, anyhow};
use cpal::{
    FromSample, SizedSample, StreamConfig,
    traits::{DeviceTrait, HostTrait, StreamTrait},
};
use deep_filter_rt::tract::DfTract;
use ndarray::Array2;
use nnnoiseless::DenoiseState;
use sonora::{AudioProcessing, Config as ApmCfg, StreamConfig as ApmSc};
use sonora::config::EchoCanceller;
use opus::{Application, Bitrate, Channels, Encoder};
use std::{
    collections::VecDeque,
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, AtomicU8, AtomicU32, Ordering::Relaxed},
        mpsc,
    },
    thread::JoinHandle,
};
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender, unbounded_channel};

#[derive(Clone, Debug)]
pub struct Pkt {
    pub data: Vec<u8>,
    pub skip: u32,
    pub last: bool,
}

#[derive(Default)]
struct Dev {
    inp: Option<String>,
    out: Option<String>,
}

struct Sh {
    mix: Mutex<Mix>,
    mute: AtomicBool,
    deaf: AtomicBool,
    ptt_mode: AtomicBool,
    ptt_down: AtomicBool,
    ns: AtomicBool,
    agc: AtomicBool,
    nsm: AtomicU8,
    aec: AtomicBool,
    refq: Mutex<VecDeque<f32>>,
    thr: AtomicU32,
    lvl: AtomicU32,
    cb: Mutex<Option<Cb>>,
    tx: UnboundedSender<Pkt>,
    rx: Mutex<Option<UnboundedReceiver<Pkt>>>,
    dev: Mutex<Dev>,
    th: Mutex<Option<(mpsc::Sender<()>, JoinHandle<()>)>>,
}

#[derive(Clone)]
pub struct Eng(Arc<Sh>);

impl Default for Eng {
    fn default() -> Self {
        Self::new()
    }
}

impl Eng {
    pub fn new() -> Eng {
        let (tx, rx) = unbounded_channel();
        Eng(Arc::new(Sh {
            mix: Mutex::new(Mix::new()),
            mute: AtomicBool::new(false),
            deaf: AtomicBool::new(false),
            ptt_mode: AtomicBool::new(false),
            ptt_down: AtomicBool::new(false),
            ns: AtomicBool::new(true),
            agc: AtomicBool::new(false),
            nsm: AtomicU8::new(0),
            aec: AtomicBool::new(false),
            refq: Mutex::new(VecDeque::new()),
            thr: AtomicU32::new((-45.0f32).to_bits()),
            lvl: AtomicU32::new((-120.0f32).to_bits()),
            cb: Mutex::new(None),
            tx,
            rx: Mutex::new(Some(rx)),
            dev: Mutex::new(Dev::default()),
            th: Mutex::new(None),
        }))
    }

    pub fn tx_frames(&self) -> Option<UnboundedReceiver<Pkt>> {
        self.0.rx.lock().unwrap().take()
    }

    pub fn give_frames(&self, rx: UnboundedReceiver<Pkt>) {
        *self.0.rx.lock().unwrap() = Some(rx);
    }

    pub fn rx(&self, ssrc: u32, seq: u16, pkt: Vec<u8>) {
        self.0.mix.lock().unwrap().push(ssrc, seq, pkt);
    }

    pub fn set_vol(&self, ssrc: u32, v: f32) {
        self.0.mix.lock().unwrap().set_vol(ssrc, v);
    }

    pub fn drop_ssrc(&self, ssrc: u32) {
        self.0.mix.lock().unwrap().drop_ssrc(ssrc);
    }

    pub fn reset(&self) {
        self.0.mix.lock().unwrap().clear();
    }

    pub fn mute(&self, on: bool) {
        self.0.mute.store(on, Relaxed);
    }

    pub fn deaf(&self, on: bool) {
        self.0.deaf.store(on, Relaxed);
        self.0.mix.lock().unwrap().deaf = on;
        if on {
            self.reset();
        }
    }

    pub fn ptt_mode(&self, on: bool) {
        self.0.ptt_mode.store(on, Relaxed);
    }

    pub fn ptt(&self, down: bool) {
        self.0.ptt_down.store(down, Relaxed);
    }

    pub fn ns(&self, on: bool) {
        self.0.ns.store(on, Relaxed);
    }

    pub fn ns_mode(&self, deep: bool) {
        self.0.nsm.store(deep as u8, Relaxed);
    }

    pub fn aec(&self, on: bool) {
        self.0.aec.store(on, Relaxed);
        if !on {
            self.0.refq.lock().unwrap().clear();
        }
    }

    pub fn agc(&self, on: bool) {
        self.0.agc.store(on, Relaxed);
    }

    pub fn vad(&self, db: Option<f32>) {
        self.0.thr.store(db.unwrap_or(f32::NEG_INFINITY).to_bits(), Relaxed);
    }

    pub fn level(&self) -> f32 {
        f32::from_bits(self.0.lvl.load(Relaxed))
    }

    pub fn jitter(&self, frames: usize) {
        self.0.mix.lock().unwrap().set_depth(frames);
    }

    pub fn on_speak(&self, f: impl Fn(Spk) + Send + Sync + 'static) {
        let c: Cb = Arc::new(f);
        *self.0.cb.lock().unwrap() = Some(c.clone());
        self.0.mix.lock().unwrap().cb = Some(c);
    }

    pub fn dev_in(&self, id: Option<String>) {
        self.0.dev.lock().unwrap().inp = id;
    }

    pub fn dev_out(&self, id: Option<String>) {
        self.0.dev.lock().unwrap().out = id;
    }

    pub fn inputs(&self) -> Vec<String> {
        let h = cpal::default_host();
        h.input_devices().map(|d| d.filter_map(|d| d.id().ok().map(|i| i.to_string())).collect()).unwrap_or_default()
    }

    pub fn outputs(&self) -> Vec<String> {
        let h = cpal::default_host();
        h.output_devices().map(|d| d.filter_map(|d| d.id().ok().map(|i| i.to_string())).collect()).unwrap_or_default()
    }

    pub fn start(&self) -> Result<()> {
        let mut g = self.0.th.lock().unwrap();
        if g.is_some() {
            return Ok(());
        }
        let (st, sr) = mpsc::channel::<()>();
        let (rt, rr) = mpsc::channel::<Result<()>>();
        let sh = self.0.clone();
        let h = std::thread::spawn(move || match open(&sh) {
            Ok(s) => {
                let _ = rt.send(Ok(()));
                let _ = sr.recv();
                drop(s);
            }
            Err(e) => {
                let _ = rt.send(Err(e));
            }
        });
        match rr.recv() {
            Ok(Ok(())) => {
                *g = Some((st, h));
                Ok(())
            }
            Ok(Err(e)) => Err(e),
            Err(_) => Err(anyhow!("audio thread died")),
        }
    }

    pub fn stop(&self) {
        if let Some((st, h)) = self.0.th.lock().unwrap().take() {
            let _ = st.send(());
            let _ = h.join();
        }
        self.0.lvl.store((-120.0f32).to_bits(), Relaxed);
    }

    pub fn pull(&self, out: &mut [f32]) {
        self.0.mix.lock().unwrap().pull(out);
    }

    pub fn cap(&self) -> Cap {
        Cap::new(self.0.clone())
    }
}

pub fn list_devs() -> serde_json::Value {
    let h = cpal::default_host();
    fn one<I: Iterator<Item = cpal::Device>>(it: Result<I, cpal::Error>) -> Vec<serde_json::Value> {
        it.map(|d| {
            d.filter_map(|d| {
                let id = d.id().ok()?.to_string();
                let name = d.description().map(|x| x.name().to_string()).unwrap_or_else(|_| id.clone());
                Some(serde_json::json!({ "id": id, "name": name }))
            })
            .collect()
        })
        .unwrap_or_default()
    }
    serde_json::json!({ "in": one(h.input_devices()), "out": one(h.output_devices()) })
}

fn pick(h: &cpal::Host, id: &Option<String>, inp: bool) -> Result<cpal::Device> {
    if let Some(s) = id
        && let Ok(did) = s.parse::<cpal::DeviceId>()
        && let Some(d) = h.device_by_id(&did)
    {
        return Ok(d);
    }
    let d = if inp { h.default_input_device() } else { h.default_output_device() };
    d.ok_or_else(|| anyhow!("no {} device", if inp { "input" } else { "output" }))
}

fn err(e: cpal::Error) {
    tracing::warn!("audio: {e}");
}

fn bin<T>(d: &cpal::Device, cfg: StreamConfig, mut f: impl FnMut(&[f32]) + Send + 'static) -> Result<cpal::Stream>
where
    T: SizedSample + Send + 'static,
    f32: FromSample<T>,
{
    let mut b: Vec<f32> = Vec::new();
    d.build_input_stream::<T, _, _>(
        cfg,
        move |x: &[T], _| {
            b.clear();
            b.extend(x.iter().map(|s| s.to_sample::<f32>()));
            f(&b);
        },
        err,
        None,
    )
    .map_err(|e| anyhow!("{e}"))
}

fn bout<T>(d: &cpal::Device, cfg: StreamConfig, mut f: impl FnMut(&mut [f32]) + Send + 'static) -> Result<cpal::Stream>
where
    T: SizedSample + FromSample<f32> + Send + 'static,
{
    let mut b: Vec<f32> = Vec::new();
    d.build_output_stream::<T, _, _>(
        cfg,
        move |x: &mut [T], _| {
            b.resize(x.len(), 0.0);
            f(&mut b);
            for (o, s) in x.iter_mut().zip(b.iter()) {
                *o = T::from_sample(*s);
            }
        },
        err,
        None,
    )
    .map_err(|e| anyhow!("{e}"))
}

fn cfgs() -> [StreamConfig; 2] {
    [2, 1].map(|c| StreamConfig { channels: c, sample_rate: SR, buffer_size: cpal::BufferSize::Default })
}

fn omk(sh: Arc<Sh>, ch: usize) -> impl FnMut(&mut [f32]) + Send + 'static {
    let mut q: VecDeque<f32> = VecDeque::new();
    let mut fr = vec![0.0f32; FS];
    move |o: &mut [f32]| {
        let n = o.len();
        while q.len() < n {
            sh.mix.lock().unwrap().pull(&mut fr);
            if sh.aec.load(Relaxed) {
                let mut g = sh.refq.lock().unwrap();
                g.extend(fr.chunks(2).map(|p| (p[0] + p[1]) * 0.5));
                let l = g.len();
                if l > 48000 {
                    g.drain(..l - 24000);
                }
            }
            if ch == 2 {
                q.extend(fr.iter());
            } else {
                q.extend(fr.chunks(2).map(|p| (p[0] + p[1]) * 0.5));
            }
        }
        for x in o.iter_mut() {
            *x = q.pop_front().unwrap_or(0.0);
        }
    }
}

fn imk(t: mpsc::Sender<(Vec<f32>, usize)>, ch: usize) -> impl FnMut(&[f32]) + Send + 'static {
    move |x: &[f32]| {
        let _ = t.send((x.to_vec(), ch));
    }
}

fn open(sh: &Arc<Sh>) -> Result<Vec<cpal::Stream>> {
    let h = cpal::default_host();
    let (ii, oi) = {
        let d = sh.dev.lock().unwrap();
        (d.inp.clone(), d.out.clone())
    };
    let mut v = Vec::new();
    let od = pick(&h, &oi, false)?;
    let mut last = anyhow!("no output config");
    for c in cfgs() {
        let ch = c.channels as usize;
        let r = bout::<f32>(&od, c.clone(), omk(sh.clone(), ch)).or_else(|_| bout::<i16>(&od, c.clone(), omk(sh.clone(), ch)));
        match r {
            Ok(s) => {
                s.play().map_err(|e| anyhow!("{e}"))?;
                v.push(s);
                break;
            }
            Err(e) => last = e,
        }
    }
    if v.is_empty() {
        return Err(last);
    }
    let idev = pick(&h, &ii, true)?;
    let mut last = anyhow!("no input config");
    let (ptx, prx) = mpsc::channel::<(Vec<f32>, usize)>();
    let sh2 = sh.clone();
    std::thread::spawn(move || {
        let mut cap = Cap::new(sh2);
        while let Ok((d, ch)) = prx.recv() {
            cap.push(&d, ch);
        }
    });
    for c in cfgs() {
        let ch = c.channels as usize;
        let r = bin::<f32>(&idev, c.clone(), imk(ptx.clone(), ch)).or_else(|_| bin::<i16>(&idev, c.clone(), imk(ptx.clone(), ch)));
        match r {
            Ok(s) => {
                s.play().map_err(|e| anyhow!("{e}"))?;
                v.push(s);
                return Ok(v);
            }
            Err(e) => last = e,
        }
    }
    Err(last)
}

pub struct Cap {
    sh: Arc<Sh>,
    enc: Encoder,
    ns: Box<DenoiseState<'static>>,
    acc: Vec<f32>,
    hang: u32,
    on: bool,
    skip: u32,
    sil: u32,
    ag: f32,
    df: Option<Box<DfTract>>,
    apm: Option<Box<AudioProcessing>>,
}

impl Cap {
    fn new(sh: Arc<Sh>) -> Cap {
        let mut enc = Encoder::new(SR, Channels::Stereo, Application::Voip).expect("opus enc");
        let _ = enc.set_bitrate(Bitrate::Bits(64000));
        let _ = enc.set_inband_fec(true);
        let _ = enc.set_packet_loss_perc(5);
        Cap { sh, enc, ns: DenoiseState::new(), acc: Vec::new(), hang: 0, on: false, skip: 0, sil: 0, ag: 1.0, df: None, apm: None }
    }

    pub fn push(&mut self, pcm: &[f32], ch: usize) {
        if ch == 2 {
            self.acc.extend(pcm.chunks(2).map(|p| (p[0] + p[1]) * 0.5));
        } else {
            self.acc.extend_from_slice(pcm);
        }
        while self.acc.len() >= FR {
            let f: Vec<f32> = self.acc.drain(..FR).collect();
            self.frame(&f);
        }
    }

    fn frame(&mut self, f: &[f32]) {
        let sh = self.sh.clone();
        let mut m: Vec<f32> = f.to_vec();
        if sh.aec.load(Relaxed) {
            let apm = self.apm.get_or_insert_with(|| {
                let cfg = ApmCfg { echo_canceller: Some(EchoCanceller::default()), ..Default::default() };
                Box::new(AudioProcessing::builder().config(cfg).capture_config(ApmSc::new(48000, 1)).render_config(ApmSc::new(48000, 1)).build())
            });
            for c in m.chunks_mut(480) {
                let mut r = [0.0f32; 480];
                {
                    let mut g = sh.refq.lock().unwrap();
                    for x in r.iter_mut() {
                        *x = g.pop_front().unwrap_or(0.0);
                    }
                }
                let mut sink = [0.0f32; 480];
                let _ = apm.process_render_f32(&[&r], &mut [&mut sink]);
                let src: Vec<f32> = c.to_vec();
                let mut out = [0.0f32; 480];
                if apm.process_capture_f32(&[&src], &mut [&mut out]).is_ok() {
                    c.copy_from_slice(&out);
                }
            }
        } else {
            self.apm = None;
        }
        if sh.ns.load(Relaxed) && sh.nsm.load(Relaxed) == 1 {
            let df = self.df.get_or_insert_with(|| Box::new(DfTract::default()));
            for c in m.chunks_mut(480) {
                let i = Array2::from_shape_vec((1, 480), c.to_vec()).unwrap();
                let mut o = Array2::<f32>::zeros((1, 480));
                if df.process(i.view(), o.view_mut()).is_ok() {
                    for (d, s) in c.iter_mut().zip(o.iter()) {
                        *d = *s;
                    }
                }
            }
        } else if sh.ns.load(Relaxed) {
            let mut o = [0.0f32; DenoiseState::FRAME_SIZE];
            for c in m.chunks_mut(DenoiseState::FRAME_SIZE) {
                let i: Vec<f32> = c.iter().map(|x| x * 32768.0).collect();
                self.ns.process_frame(&mut o, &i);
                for (d, s) in c.iter_mut().zip(o.iter()) {
                    *d = s / 32768.0;
                }
            }
        }
        let rms = (m.iter().map(|x| x * x).sum::<f32>() / m.len() as f32).sqrt();
        let db = 20.0 * rms.max(1e-6).log10();
        if sh.agc.load(Relaxed) {
            if rms > 0.003 {
                let want = (0.1 / rms).clamp(0.5, 8.0);
                let k = if want < self.ag { 0.3 } else { 0.02 };
                self.ag += (want - self.ag) * k;
            }
            for x in m.iter_mut() {
                *x = (*x * self.ag).clamp(-0.98, 0.98);
            }
        } else {
            self.ag = 1.0;
        }
        sh.lvl.store(db.to_bits(), Relaxed);
        let talk = if sh.mute.load(Relaxed) {
            false
        } else if sh.ptt_mode.load(Relaxed) {
            sh.ptt_down.load(Relaxed)
        } else {
            let thr = f32::from_bits(sh.thr.load(Relaxed));
            if db > thr {
                self.hang = 10;
                true
            } else if self.hang > 0 {
                self.hang -= 1;
                true
            } else {
                false
            }
        };
        if talk {
            if !self.on {
                self.on = true;
                self.say(true);
            }
            self.sil = 0;
            let st: Vec<f32> = m.iter().flat_map(|&x| [x, x]).collect();
            if let Ok(d) = self.enc.encode_vec_float(&st, 1275) {
                let skip = std::mem::take(&mut self.skip);
                let _ = sh.tx.send(Pkt { data: d, skip, last: false });
            }
        } else if self.on {
            self.sil += 1;
            let last = self.sil >= 5;
            let skip = std::mem::take(&mut self.skip);
            let _ = sh.tx.send(Pkt { data: SIL.to_vec(), skip, last });
            if last {
                self.on = false;
                self.sil = 0;
                let _ = self.enc.reset_state();
                self.say(false);
            }
        } else {
            self.skip += 1;
        }
    }

    fn say(&self, on: bool) {
        if let Some(c) = &*self.sh.cb.lock().unwrap() {
            c(Spk { ssrc: None, on });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tone(n: usize, a: f32) -> Vec<f32> {
        (0..n).map(|i| (i as f32 * 300.0 * std::f32::consts::TAU / SR as f32).sin() * a).collect()
    }

    fn drain(rx: &mut UnboundedReceiver<Pkt>) -> Vec<Pkt> {
        let mut v = vec![];
        while let Ok(p) = rx.try_recv() {
            v.push(p);
        }
        v
    }

    #[test]
    fn vad_gates_and_ends() {
        let e = Eng::new();
        e.ns(false);
        let mut rx = e.tx_frames().unwrap();
        let mut c = e.cap();
        c.push(&tone(FR * 3, 0.0), 1);
        assert!(drain(&mut rx).is_empty());
        c.push(&tone(FR * 5, 0.3), 1);
        let v = drain(&mut rx);
        assert_eq!(v.len(), 5);
        assert!(v[0].skip == 3 && v[1].skip == 0);
        c.push(&tone(FR * 20, 0.0), 1);
        let v = drain(&mut rx);
        assert!(v.last().unwrap().last);
        assert!(v.iter().filter(|p| p.data == SIL).count() == 5);
    }

    #[test]
    fn mute_and_ptt() {
        let e = Eng::new();
        e.ns(false);
        let mut rx = e.tx_frames().unwrap();
        let mut c = e.cap();
        e.mute(true);
        c.push(&tone(FR * 5, 0.3), 1);
        assert!(drain(&mut rx).is_empty());
        e.mute(false);
        e.ptt_mode(true);
        c.push(&tone(FR * 5, 0.3), 1);
        assert!(drain(&mut rx).is_empty());
        e.ptt(true);
        c.push(&tone(FR * 2, 0.0), 1);
        assert_eq!(drain(&mut rx).len(), 2);
    }

    #[test]
    fn loopback() {
        let e = Eng::new();
        e.ns(false);
        let mut rx = e.tx_frames().unwrap();
        let mut c = e.cap();
        c.push(&tone(FR * 10, 0.4), 1);
        let v = drain(&mut rx);
        for (i, p) in v.iter().enumerate() {
            e.rx(5, i as u16, p.data.clone());
        }
        let mut o = vec![0.0f32; FS];
        let mut peak = 0.0f32;
        for _ in 0..10 {
            e.0.mix.lock().unwrap().pull(&mut o);
            peak = peak.max(o.iter().fold(0.0, |a, &b| a.max(b.abs())));
        }
        assert!(peak > 0.1, "{peak}");
    }

    #[test]
    fn speak_cb() {
        let e = Eng::new();
        e.ns(false);
        let log = Arc::new(Mutex::new(vec![]));
        let l = log.clone();
        e.on_speak(move |s| l.lock().unwrap().push(s));
        let _rx = e.tx_frames().unwrap();
        let mut c = e.cap();
        c.push(&tone(FR * 3, 0.3), 1);
        c.push(&tone(FR * 20, 0.0), 1);
        assert_eq!(*log.lock().unwrap(), vec![Spk { ssrc: None, on: true }, Spk { ssrc: None, on: false }]);
    }
}
