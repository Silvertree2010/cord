use crate::{Cod, Frame, Res, init};
use gstreamer as gst;
use gst::prelude::*;
use gstreamer_app as app;
use gstreamer_video::{self as gv, VideoFrameExt};
use std::sync::Arc;

pub struct Dec {
    pl: gst::Pipeline,
    src: app::AppSrc,
    pub name: &'static str,
}

fn cands(c: Cod, sw: bool) -> Vec<(&'static str, &'static str, &'static str)> {
    let v: &[(&str, &str, &str)] = match c {
        Cod::H264 => &[("nvh264dec", "video/x-h264,stream-format=byte-stream,alignment=au", "h264parse"), ("avdec_h264", "video/x-h264,stream-format=byte-stream,alignment=au", "h264parse")],
        Cod::Vp8 => &[("nvvp8dec", "video/x-vp8", "identity"), ("avdec_vp8", "video/x-vp8", "identity"), ("vp8dec", "video/x-vp8", "identity")],
        Cod::Av1 => &[("nvav1dec", "video/x-av1,stream-format=obu-stream,alignment=tu", "av1parse"), ("vulkanav1dec", "video/x-av1,stream-format=obu-stream,alignment=tu", "av1parse"), ("av1dec", "video/x-av1,stream-format=obu-stream,alignment=tu", "av1parse"), ("avdec_av1", "video/x-av1,stream-format=obu-stream,alignment=tu", "av1parse")],
    };
    v.iter().copied().filter(|(n, _, _)| !(sw && n.starts_with("nv") || sw && n.starts_with("vulkan"))).filter(|(n, _, p)| crate::has(n) && crate::has(p)).collect()
}

pub fn desc(dec: &str, caps: &str, parse: &str) -> String {
    format!("appsrc name=in is-live=true format=time do-timestamp=true caps=\"{caps}\" ! {parse} ! {dec} ! videoconvert ! video/x-raw,format=RGBA ! appsink name=out sync=false drop=true max-buffers=2")
}

impl Dec {
    pub fn new(c: Cod, sw: bool, cb: impl Fn(Frame) + Send + Sync + 'static) -> Res<Dec> {
        init()?;
        let cb = Arc::new(cb);
        let mut last = anyhow::anyhow!("no decoder available for {c:?}");
        for (n, caps, p) in cands(c, sw) {
            match Self::build(n, caps, p, cb.clone()) {
                Ok(d) => return Ok(d),
                Err(e) => last = e,
            }
        }
        Err(last)
    }

    fn build(name: &'static str, caps: &str, parse: &str, cb: Arc<dyn Fn(Frame) + Send + Sync>) -> Res<Dec> {
        let pl = gst::parse::launch(&desc(name, caps, parse))?.downcast::<gst::Pipeline>().map_err(|_| anyhow::anyhow!("not a pipeline"))?;
        let src = pl.by_name("in").and_then(|e| e.downcast::<app::AppSrc>().ok()).ok_or_else(|| anyhow::anyhow!("no appsrc"))?;
        let sink = pl.by_name("out").and_then(|e| e.downcast::<app::AppSink>().ok()).ok_or_else(|| anyhow::anyhow!("no appsink"))?;
        sink.set_callbacks(
            app::AppSinkCallbacks::builder()
                .new_sample(move |s| {
                    let smp = s.pull_sample().map_err(|_| gst::FlowError::Eos)?;
                    let buf = smp.buffer().ok_or(gst::FlowError::Error)?;
                    let info = gv::VideoInfo::from_caps(smp.caps().ok_or(gst::FlowError::Error)?).map_err(|_| gst::FlowError::Error)?;
                    let f = gv::VideoFrameRef::from_buffer_ref_readable(buf, &info).map_err(|_| gst::FlowError::Error)?;
                    let (w, h) = (info.width() as usize, info.height() as usize);
                    let st = f.plane_stride()[0] as usize;
                    let d = f.plane_data(0).map_err(|_| gst::FlowError::Error)?;
                    let mut rgba = Vec::with_capacity(w * h * 4);
                    for y in 0..h {
                        rgba.extend_from_slice(&d[y * st..y * st + w * 4]);
                    }
                    cb(Frame { w: w as u32, h: h as u32, rgba });
                    Ok(gst::FlowSuccess::Ok)
                })
                .build(),
        );
        src.set_property("block", false);
        src.set_max_bytes(8 << 20);
        pl.set_state(gst::State::Playing).map_err(|e| anyhow::anyhow!("decoder {name}: {e}"))?;
        Ok(Dec { pl, src, name })
    }

    pub fn push(&self, data: Vec<u8>, pts_us: u64) {
        let mut b = gst::Buffer::from_mut_slice(data);
        b.get_mut().unwrap().set_pts(gst::ClockTime::from_useconds(pts_us));
        let _ = self.src.push_buffer(b);
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

impl Drop for Dec {
    fn drop(&mut self) {
        let _ = self.pl.set_state(gst::State::Null);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn desc_shape() {
        let d = desc("avdec_h264", "video/x-h264", "h264parse");
        assert!(d.contains("! avdec_h264 ! videoconvert ! video/x-raw,format=RGBA"));
    }

    #[test]
    fn sw_skips_hw() {
        assert!(cands(Cod::H264, true).iter().all(|c| !c.0.starts_with("nv")));
    }
}
