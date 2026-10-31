use cord_core::*;
use cord_st::{St, login};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let base = std::env::var("ST_BASE").unwrap_or_else(|_| "https://api.stoat.chat".into());
    let tok = match std::env::var("ST_TOKEN") {
        Ok(t) => t,
        Err(_) => {
            let e = std::env::var("ST_EMAIL")?;
            let p = std::env::var("ST_PASS")?;
            login(&base, &e, &p).await?
        }
    };
    let st = St::new(1, base, tok);
    let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();
    st.start(tx).await?;
    let mut first = true;
    while let Some(e) = rx.recv().await {
        match &e {
            Ev::Msg(m) => println!("msg {} <{}> {}", m.chan, m.auth.shown(), m.body),
            Ev::Gld(g) => println!("gld {} {}", g.id, g.name),
            Ev::Chan(c) => println!("chan {} {:?} {}", c.id, c.k, c.name),
            other => println!("{other:?}"),
        }
        if let Ev::Chan(c) = &e {
            if first && c.k == ChK::Txt {
                first = false;
                let h = st.hist(&c.id, None, 5).await?;
                for m in h {
                    println!("hist <{}> {}", m.auth.shown(), m.body);
                }
            }
        }
    }
    Ok(())
}
