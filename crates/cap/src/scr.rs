use crate::Res;
use ashpd::desktop::{
    Session,
    screencast::{CursorMode, Screencast, SelectSourcesOptions, SourceType},
};
use std::{any::Any, os::fd::OwnedFd};

pub struct Pipe {
    pub fd: OwnedFd,
    pub node: u32,
    pub size: Option<(i32, i32)>,
    _keep: Box<dyn Any + Send>,
}

struct Keep {
    _p: Screencast,
    _s: Session<Screencast>,
}

pub async fn pick(win: bool) -> Res<Pipe> {
    let p = Screencast::new().await?;
    let s = p.create_session(Default::default()).await?;
    let src = if win { SourceType::Window.into() } else { SourceType::Monitor | SourceType::Window };
    p.select_sources(&s, SelectSourcesOptions::default().set_cursor_mode(CursorMode::Embedded).set_sources(src).set_multiple(false)).await?;
    let r = p.start(&s, None, Default::default()).await?.response()?;
    let st = r.streams().first().ok_or_else(|| anyhow::anyhow!("no stream selected"))?.to_owned();
    let fd = p.open_pipe_wire_remote(&s, Default::default()).await?;
    Ok(Pipe { fd, node: st.pipe_wire_node_id(), size: st.size(), _keep: Box::new(Keep { _p: p, _s: s }) })
}
