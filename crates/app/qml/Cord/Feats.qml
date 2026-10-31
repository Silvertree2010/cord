import QtQuick
import Cord

QtObject {
    Component.onCompleted: {
        S.hook("send", o => {
            if (!S.f("fakenitro") || !o.body) return o
            const cid = S.selC
            if (S.svc(cid) !== "dc") return o
            const ch = S.chans[cid]
            const here = ch ? ch.grid : ""
            const own = {}
            if (here && S.gmap[here]) for (const e of S.gmap[here].emos || []) if (e.id) own[e.id.raw] = true
            o.body = o.body.replace(/<(a?):(\w+):(\d+)>/g, (m, an, n, id) => {
                if (own[id] && !an) return m
                return "[" + n + "](https://cdn.discordapp.com/emojis/" + id + "." + (an ? "gif" : "webp") + "?size=48&quality=lossless&name=" + n + ")"
            })
            return o
        })
        S.hook("send", o => {
            if (o.body && /^@silent\s/.test(o.body)) { o.body = o.body.replace(/^@silent\s+/, ""); o.silent = true }
            return o
        })
        S.hook("render", m => {
            if (!S.f("fakenitro") || !m.body) return m
            const r = /^\[(\w+)\]\(https:\/\/cdn\.discordapp\.com\/emojis\/(\d+)\.(gif|webp|png)\?[^)]*\)$/
            if (r.test(m.body.trim())) {
                const x = m.body.trim().match(r)
                return Object.assign({}, m, { body: "<" + (x[3] === "gif" ? "a" : "") + ":" + x[1] + ":" + x[2] + ">" })
            }
            const r2 = /https:\/\/cdn\.discordapp\.com\/emojis\/(\d+)\.(gif|webp|png)\?[^\s)]*name=(\w+)[^\s)]*/g
            if (r2.test(m.body)) return Object.assign({}, m, { body: m.body.replace(/\[(\w+)\]\(https:\/\/cdn\.discordapp\.com\/emojis\/(\d+)\.(gif|webp|png)\?[^)]*\)/g, (s, n, id, ex) => "<" + (ex === "gif" ? "a" : "") + ":" + n + ":" + id + ">") })
            return m
        })
    }
}
