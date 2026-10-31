use crate::{Res, first, init};
use gstreamer as gst;
use gst::prelude::*;
use gstreamer_app as app;
use tokio::sync::mpsc::{UnboundedReceiver, unbounded_channel};

#[derive(Clone, Debug)]
pub struct Op {
    pub data: Vec<u8>,
    pub pts_us: u64,
}

#[derive(Clone, Debug, Default)]
pub struct Cfg {
    pub kbps: u32,
    pub node: Option<String>,
    pub test: bool,
}

pub fn src_str(c: &Cfg) -> Res<String> {
    if c.test {
        return Ok("audiotestsrc is-live=true wave=sine freq=440".into());
    }
    match first(&["pipewiresrc", "pulsesrc", "alsasrc"]) {
        Some("pipewiresrc") => Ok(match &c.node {
            Some(n) => format!("pipewiresrc target-object=\"{n}\" do-timestamp=true"),
            None => "pipewiresrc stream-properties=\"props,stream.capture.sink=true\" do-timestamp=true".into(),
        }),
        Some("pulsesrc") => Ok(match &c.node {
            Some(n) => format!("pulsesrc device=\"{n}\" do-timestamp=true"),
            None => "pulsesrc device=@DEFAULT_MONITOR@ do-timestamp=true".into(),
        }),
        Some(_) => Ok("alsasrc do-timestamp=true".into()),
        None => anyhow::bail!("no audio source element"),
    }
}

pub fn desc(src: &str, kbps: u32) -> String {
    format!(
        "{src} ! queue max-size-time=200000000 leaky=downstream ! audioconvert ! audioresample ! audio/x-raw,rate=48000,channels=2 ! opusenc name=e bitrate={} frame-size=20 audio-type=generic ! appsink name=out sync=false max-buffers=32",
        kbps.max(8) * 1000
    )
}

pub struct Aenc {
    pl: gst::Pipeline,
}

impl Aenc {
    pub fn start(c: Cfg) -> Res<(Aenc, UnboundedReceiver<Op>)> {
        init()?;
        let d = desc(&src_str(&c)?, if c.kbps == 0 { 128 } else { c.kbps });
        let pl = gst::parse::launch(&d)?.downcast::<gst::Pipeline>().map_err(|_| anyhow::anyhow!("not a pipeline"))?;
        let a = pl.by_name("out").and_then(|e| e.downcast::<app::AppSink>().ok()).ok_or_else(|| anyhow::anyhow!("no appsink"))?;
        let (tx, rx) = unbounded_channel();
        a.set_callbacks(
            app::AppSinkCallbacks::builder()
                .new_sample(move |s| {
                    let smp = s.pull_sample().map_err(|_| gst::FlowError::Eos)?;
                    let b = smp.buffer().ok_or(gst::FlowError::Error)?;
                    let m = b.map_readable().map_err(|_| gst::FlowError::Error)?;
                    let _ = tx.send(Op { data: m.as_slice().to_vec(), pts_us: b.pts().map(|t| t.useconds()).unwrap_or(0) });
                    Ok(gst::FlowSuccess::Ok)
                })
                .build(),
        );
        pl.set_state(gst::State::Playing).map_err(|e| anyhow::anyhow!("audio start: {e}"))?;
        Ok((Aenc { pl }, rx))
    }
}

impl Drop for Aenc {
    fn drop(&mut self) {
        let _ = self.pl.set_state(gst::State::Null);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn opus_test_tone() {
        let (a, mut rx) = Aenc::start(Cfg { kbps: 96, node: None, test: true }).unwrap();
        std::thread::sleep(std::time::Duration::from_millis(700));
        drop(a);
        let mut n = 0;
        while let Ok(p) = rx.try_recv() {
            assert!(!p.data.is_empty() && p.data.len() < 1500);
            n += 1;
        }
        assert!(n >= 20, "{n}");
    }

    #[test]
    fn desc_has_opus() {
        assert!(desc("x", 128).contains("bitrate=128000"));
    }
}
