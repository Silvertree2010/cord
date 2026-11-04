use crate::Res;

pub struct Pipe {
    pub idx: u32,
    pub size: Option<(i32, i32)>,
}

pub async fn pick(_win: bool) -> Res<Pipe> {
    if !crate::has("avfvideosrc") {
        anyhow::bail!("gstreamer avfvideosrc missing (applemedia plugin)");
    }
    Ok(Pipe { idx: 0, size: None })
}
