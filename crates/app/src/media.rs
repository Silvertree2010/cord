use crate::out::Mp;
use std::{
    io::Write,
    process::{Command, Stdio},
    sync::atomic::{AtomicBool, Ordering},
};

pub fn esc(s: &str) -> String {
    s.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;")
}

pub fn copy(t: String) {
    std::thread::spawn(move || {
        let tools: [(&str, &[&str]); 3] = [("wl-copy", &[]), ("xclip", &["-selection", "clipboard"]), ("xsel", &["--clipboard", "--input"])];
        for (c, a) in tools {
            let Ok(mut ch) = Command::new(c).args(a).stdin(Stdio::piped()).stdout(Stdio::null()).stderr(Stdio::null()).spawn() else { continue };
            if let Some(mut i) = ch.stdin.take() {
                if i.write_all(t.as_bytes()).is_err() {
                    let _ = ch.wait();
                    continue;
                }
            }
            if ch.wait().map(|s| s.success()).unwrap_or(false) {
                return;
            }
        }
        tracing_warn("no clipboard tool succeeded");
    });
}

fn tracing_warn(m: &str) {
    eprintln!("cord: {m}");
}

pub fn notify(t: String, b: String) {
    std::thread::spawn(move || {
        let _ = notify_rust::Notification::new().appname("cord").summary(&esc(&t)).body(&esc(&b)).icon("cord").show();
    });
}

pub fn media_cmd(cmd: &str) {
    let c = match cmd {
        "prev" => "previous",
        "next" => "next",
        _ => "play-pause",
    };
    std::thread::spawn(move || {
        if let Ok(mut ch) = Command::new("playerctl").arg(c).stdout(Stdio::null()).stderr(Stdio::null()).spawn() {
            let _ = ch.wait();
        }
    });
}

static BUSY: AtomicBool = AtomicBool::new(false);

pub fn parse_mpris(t: &str) -> Option<Mp> {
    let p: Vec<&str> = t.trim().split('\u{1f}').collect();
    if p.len() < 7 || p[1].is_empty() {
        return None;
    }
    let n = |s: &str| s.parse::<f64>().unwrap_or(0.0);
    Some(Mp { playing: p[0] == "Playing", title: p[1].into(), artist: p[2].into(), art: p[3].into(), url: p[4].into(), pos: n(p[5]), len: n(p[6]) })
}

pub fn mpris(done: impl FnOnce(Option<Mp>) + Send + 'static) {
    if BUSY.swap(true, Ordering::SeqCst) {
        return;
    }
    std::thread::spawn(move || {
        let o = Command::new("playerctl")
            .args(["metadata", "--format", "{{status}}\u{1f}{{title}}\u{1f}{{artist}}\u{1f}{{mpris:artUrl}}\u{1f}{{xesam:url}}\u{1f}{{position}}\u{1f}{{mpris:length}}"])
            .stderr(Stdio::null())
            .output();
        BUSY.store(false, Ordering::SeqCst);
        let r = o.ok().and_then(|o| parse_mpris(&String::from_utf8_lossy(&o.stdout)));
        done(r);
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn escapes_markup() {
        assert_eq!(esc("a<b>&c"), "a&lt;b&gt;&amp;c");
    }

    #[test]
    fn mpris_parse() {
        let m = parse_mpris("Playing\u{1f}Song\u{1f}Me\u{1f}\u{1f}u\u{1f}5\u{1f}10\n").unwrap();
        assert!(m.playing && m.title == "Song" && m.len == 10.0);
        assert!(parse_mpris("").is_none());
        assert!(parse_mpris("Paused\u{1f}\u{1f}\u{1f}\u{1f}\u{1f}0\u{1f}0").is_none());
    }
}
