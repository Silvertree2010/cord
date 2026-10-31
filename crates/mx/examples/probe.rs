use cord_core::*;
use cord_mx::Mx;
use std::path::PathBuf;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let dir = PathBuf::from(std::env::var("MX_DIR").unwrap_or_else(|_| "mx-probe".into()));
    std::fs::create_dir_all(&dir)?;
    let sess = dir.join("session.json");
    let store = dir.join("store");
    let mx = match std::fs::read_to_string(&sess) {
        Ok(s) => Mx::restore(1, &store, &s).await?,
        Err(_) => {
            let hs = std::env::var("MX_HS")?;
            let u = std::env::var("MX_USER")?;
            let p = std::env::var("MX_PASS")?;
            let mx = Mx::login(1, &hs, &u, &p, &store).await?;
            std::fs::write(&sess, mx.session_json()?)?;
            mx
        }
    };
    if let Ok(k) = std::env::var("MX_RECOVERY") {
        mx.recover(&k).await?;
    }
    let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();
    mx.start(tx).await?;
    while let Some(e) = rx.recv().await {
        match &e {
            Ev::Msg(m) => println!("msg {} <{}> {}", m.chan, m.auth.shown(), m.body),
            Ev::Gld(g) => println!("gld {} {}", g.id, g.name),
            Ev::Chan(c) => println!("chan {} {:?} {}", c.id, c.k, c.name),
            other => println!("{other:?}"),
        }
    }
    Ok(())
}
