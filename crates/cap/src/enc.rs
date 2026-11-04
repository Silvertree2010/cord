use crate::{Au, Res, first, init, scr::Pipe};
use gstreamer as gst;
use gst::prelude::*;
use gstreamer_app as app;
#[cfg(target_os = "linux")]
use std::os::fd::AsRawFd;
use tokio::sync::mpsc::{UnboundedReceiver, unbounded_channel};

#[derive(Clone, Copy, Debug)]
pub struct Cfg {
    pub w: u32,
    pub h: u32,
    pub fps: u32,
    pub kbps: u32,
}

pub enum Src {
    Test,
    Pw(Pipe),
}

pub const ENCS: &[&str] = &["vtenc_h264_hw", "vtenc_h264", "nvh264enc", "nvautogpuh264enc", "vah264enc", "vah264lpenc", "vaapih264enc", "x264enc", "openh264enc"];

pub fn pick() -> Option<&'static str> {
    first(ENCS)
}

pub fn set(e: &gst::Element, k: &str, v: &str) {
    if e.find_property(k).is_some() {
        e.set_property_from_str(k, v);
    }
}

fn tune(e: &gst::Element, name: &str, c: &Cfg) {
    let b = c.kbps.to_string();
    let g = (c.fps * 4).to_string();
    match name {
        "nvh264enc" | "nvautogpuh264enc" => {
            set(e, "bitrate", &b);
            set(e, "max-bitrate", &(c.kbps * 3 / 2).to_string());
            set(e, "rc-mode", "cbr");
            set(e, "preset", "p4");
            set(e, "tune", "low-latency");
            set(e, "zerolatency", "true");
            set(e, "bframes", "0");
            set(e, "gop-size", &g);
            set(e, "repeat-sequence-header", "true");
            set(e, "aud", "false");
        }
        "vtenc_h264_hw" | "vtenc_h264" => {
            set(e, "bitrate", &b);
            set(e, "max-keyframe-interval", &g);
            set(e, "realtime", "true");
            set(e, "allow-frame-reordering", "false");
        }
        "x264enc" => {
            set(e, "bitrate", &b);
            set(e, "speed-preset", "veryfast");
            set(e, "tune", "zerolatency");
            set(e, "key-int-max", &g);
            set(e, "bframes", "0");
            set(e, "byte-stream", "true");
            set(e, "threads", "0");
        }
        "vah264enc" | "vah264lpenc" => {
            set(e, "bitrate", &b);
            set(e, "rate-control", "cbr");
            set(e, "key-int-max", &g);
            set(e, "b-frames", "0");
        }
        "vaapih264enc" => {
            set(e, "bitrate", &b);
            set(e, "rate-control", "cbr");
            set(e, "keyframe-period", &g);
            set(e, "max-bframes", "0");
        }
        "openh264enc" => {
            set(e, "bitrate", &(c.kbps * 1000).to_string());
            set(e, "gop-size", &g);
            set(e, "rate-control", "bitrate");
        }
        _ => {}
    }
}

pub fn src_str(s: &Src, fps: u32) -> String {
    match s {
        Src::Test => format!("videotestsrc is-live=true pattern=ball ! video/x-raw,framerate={fps}/1"),
        #[cfg(target_os = "linux")]
        Src::Pw(p) => format!("pipewiresrc fd={} path={} always-copy=true do-timestamp=true", p.fd.as_raw_fd(), p.node),
        #[cfg(not(target_os = "linux"))]
        Src::Pw(p) => format!("avfvideosrc capture-screen=true capture-screen-cursor=true device-index={} do-timestamp=true", p.idx),
    }
}

pub fn desc(src: &str, enc: &str, c: &Cfg) -> String {
    format!(
        "{src} ! queue max-size-buffers=4 leaky=downstream ! videoconvert ! videoscale add-borders=true ! videorate ! video/x-raw,format=NV12,width={w},height={h},framerate={f}/1 ! {enc} name=e ! video/x-h264,profile=main ! h264parse config-interval=-1 ! video/x-h264,stream-format=byte-stream,alignment=au ! appsink name=out sync=false max-buffers=16",
        w = c.w & !1,
        h = c.h & !1,
        f = c.fps,
    )
}

pub struct Enc {
    pl: gst::Pipeline,
    sink: gst::Element,
    pub name: &'static str,
    _src: Src,
}

impl Enc {
    pub fn start(c: Cfg, s: Src) -> Res<(Enc, UnboundedReceiver<Au>)> {
        init()?;
        let name = pick().ok_or_else(|| anyhow::anyhow!("no H.264 encoder (nvh264enc, va, x264) available"))?;
        if cfg!(target_os = "linux") && matches!(s, Src::Pw(_)) && !crate::has("pipewiresrc") {
            anyhow::bail!("gstreamer pipewiresrc missing (install gst-plugin-pipewire)");
        }
        let d = desc(&src_str(&s, c.fps), name, &c);
        let pl = gst::parse::launch(&d)?.downcast::<gst::Pipeline>().map_err(|_| anyhow::anyhow!("not a pipeline"))?;
        let e = pl.by_name("e").ok_or_else(|| anyhow::anyhow!("no encoder element"))?;
        tune(&e, name, &c);
        let sink = pl.by_name("out").ok_or_else(|| anyhow::anyhow!("no sink"))?;
        let (tx, rx) = unbounded_channel();
        let a = sink.clone().downcast::<app::AppSink>().map_err(|_| anyhow::anyhow!("not appsink"))?;
        a.set_callbacks(
            app::AppSinkCallbacks::builder()
                .new_sample(move |s| {
                    let smp = s.pull_sample().map_err(|_| gst::FlowError::Eos)?;
                    let b = smp.buffer().ok_or(gst::FlowError::Error)?;
                    let m = b.map_readable().map_err(|_| gst::FlowError::Error)?;
                    let au = Au { data: m.as_slice().to_vec(), pts_us: b.pts().map(|t| t.useconds()).unwrap_or(0), key: !b.flags().contains(gst::BufferFlags::DELTA_UNIT) };
                    let _ = tx.send(au);
                    Ok(gst::FlowSuccess::Ok)
                })
                .build(),
        );
        pl.set_state(gst::State::Playing).map_err(|e| anyhow::anyhow!("encoder start: {e}"))?;
        Ok((Enc { pl, sink, name, _src: s }, rx))
    }

    pub fn key(&self) {
        self.keyer().key();
    }

    pub fn keyer(&self) -> Keyer {
        Keyer(self.sink.clone())
    }

    pub fn bitrate(&self, kbps: u32) {
        if let Some(e) = self.pl.by_name("e") {
            set(&e, "bitrate", &kbps.to_string());
        }
    }

    pub fn err(&self) -> Option<String> {
        let b = self.pl.bus()?;
        while let Some(m) = b.pop() {
            if let gst::MessageView::Error(e) = m.view() {
                return Some(format!("{} ({})", e.error(), e.debug().unwrap_or_default()));
            }
        }
        None
    }
}

#[derive(Clone)]
pub struct Keyer(gst::Element);

impl Keyer {
    pub fn key(&self) {
        let ev = gstreamer_video::UpstreamForceKeyUnitEvent::builder().all_headers(true).build();
        if let Some(p) = self.0.static_pad("sink") {
            p.push_event(ev);
        }
    }
}

impl Drop for Enc {
    fn drop(&mut self) {
        let _ = self.pl.set_state(gst::State::Null);
    }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn split() {
        let b = [0, 0, 0, 1, 0x67, 1, 2, 0, 0, 1, 0x68, 3, 0, 0, 0, 1, 0x65, 9, 9];
        let n = nals(&b);
        assert_eq!(n, vec![&[0x67, 1, 2][..], &[0x68, 3][..], &[0x65, 9, 9][..]]);
    }

    #[test]
    fn pipeline_string() {
        let c = Cfg { w: 1921, h: 1081, fps: 60, kbps: 8000 };
        let d = desc("videotestsrc", "nvh264enc", &c);
        assert!(d.contains("width=1920,height=1080,framerate=60/1"));
        assert!(d.contains("appsink name=out"));
    }

    #[test]
    fn encode_test_source() {
        if pick().is_none() {
            return;
        }
        let c = Cfg { w: 1280, h: 720, fps: 30, kbps: 3000 };
        let (e, mut rx) = Enc::start(c, Src::Test).unwrap();
        let t = std::time::Instant::now();
        let mut v = vec![];
        while t.elapsed() < std::time::Duration::from_secs(3) {
            while let Ok(a) = rx.try_recv() {
                v.push(a);
            }
            std::thread::sleep(std::time::Duration::from_millis(50));
        }
        e.key();
        std::thread::sleep(std::time::Duration::from_millis(300));
        while let Ok(a) = rx.try_recv() {
            v.push(a);
        }
        let err = e.err();
        drop(e);
        assert!(err.is_none(), "{err:?}");
        assert!(v.len() > 30, "{}", v.len());
        assert!(v[0].key);
        let n = nals(&v[0].data);
        assert!(n.iter().any(|x| x[0] & 31 == 7));
        assert!(n.iter().any(|x| x[0] & 31 == 5));
        assert!(v.iter().filter(|a| a.key).count() >= 2);
        assert!(v.windows(2).all(|w| w[1].pts_us >= w[0].pts_us));
    }
}
