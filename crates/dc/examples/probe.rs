use cord_core::{Bk, Ev};
use cord_dc::Dc;

#[tokio::main]
async fn main() {
    let t = std::env::var("CORD_DC_TOKEN").expect("CORD_DC_TOKEN");
    let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();
    let d = Dc::new(1, t).unwrap();
    d.start(tx).await.unwrap();
    let (mut g, mut c, mut h) = (0, 0, 0);
    while let Some(e) = rx.recv().await {
        match &e {
            Ev::Gld(x) => {
                g += 1;
                println!("gld {} {} roles={}", x.id, x.name, x.roles.len());
            }
            Ev::Chan(x) => {
                c += 1;
                h += x.hidden as u32;
                if c % 50 == 1 {
                    println!("chan {} {:?} {} hidden={} (total {c}, hidden {h}, gld {g})", x.id, x.k, x.name, x.hidden);
                }
            }
            Ev::Msg(m) => println!("msg {} <{}> {}", m.chan, m.auth.shown(), m.body),
            _ => println!("{e:?}"),
        }
    }
}
