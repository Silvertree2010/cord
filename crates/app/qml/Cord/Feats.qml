import QtQuick
import Cord
import "Fmt.js" as F

QtObject {
    Component.onCompleted: {
        S.hookCore("send", o => {
            if (!S.f("fakenitro") || !o.body) return o
            const cid = o.cid
            if (!cid || !S.caps(cid).fakeEmoji) return o
            const ch = S.chans[cid]
            const here = ch ? ch.grid : ""
            const own = {}
            if (here && S.gmap[here]) for (const e of S.gmap[here].emos || []) if (e.id) own[e.id.raw] = true
            o.body = F.emoEncode(o.body, own)
            return o
        }, "fakenitro")
        S.hookCore("send", o => {
            if (o.body && /^@silent\s/.test(o.body)) { o.body = o.body.replace(/^@silent\s+/, ""); o.silent = true }
            return o
        }, "silent")
        S.hookCore("render", m => {
            if (!S.f("fakenitro") || !m.body) return m
            const b = F.emoDecode(m.body)
            return b === m.body ? m : Object.assign({}, m, { body: b })
        }, "fakenitro")
    }
}
