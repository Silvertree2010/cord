use crate::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Ev {
    Ready(Acc),
    Gld(Gld),
    GldDel(Rid),
    Chan(Chan),
    ChanDel(Rid),
    Msg(Msg),
    MsgUpd(Msg),
    MsgDel { chan: Rid, id: Rid },
    React { chan: Rid, id: Rid, e: Emo, me: bool, add: bool },
    Typing { chan: Rid, usr: Usr },
    Pres { usr: Rid, p: Pres },
    Rel(Rel),
    RelDel(Rid),
    Mem(Mem),
    Mems(Vec<Mem>),
    Unread { chan: Rid, n: u32, ment: u32 },
    Ack { chan: Rid },
    Vs(Vs),
    Speak { chan: Rid, usr: Rid, on: bool },
    Call(CallSt),
    Conn { acc: u32, up: bool },
    Err { acc: u32, msg: String },
    Folds { acc: u32, list: Vec<Fold> },
}
