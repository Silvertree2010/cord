use cord_rtc::Eng;
use std::time::Duration;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let secs: u64 = std::env::args().nth(1).and_then(|s| s.parse().ok()).unwrap_or(15);
    let e = Eng::new();
    e.on_speak(|s| println!("speak {:?}", s));
    e.vad(Some(-50.0));
    let mut rx = e.tx_frames().ok_or_else(|| anyhow::anyhow!("no frames"))?;
    e.start()?;
    println!("loopback for {secs}s, talk into the mic (use headphones)");
    let e2 = e.clone();
    let t = tokio::spawn(async move {
        let mut seq = 0u16;
        while let Some(p) = rx.recv().await {
            e2.rx(1, seq, p.data);
            seq = seq.wrapping_add(1);
        }
    });
    let lv = e.clone();
    let m = tokio::spawn(async move {
        loop {
            tokio::time::sleep(Duration::from_millis(500)).await;
            println!("level {:.1} dB", lv.level());
        }
    });
    tokio::time::sleep(Duration::from_secs(secs)).await;
    m.abort();
    e.stop();
    t.abort();
    Ok(())
}
