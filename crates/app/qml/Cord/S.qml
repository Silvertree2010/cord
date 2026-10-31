pragma Singleton
import QtQuick
import cord.core
import "Lay.js" as L

QtObject {
    id: s

    property Core c: Core {
        onDl: d => s.on(JSON.parse(d))
    }

    property var accs: ({})
    property var up: []
    property var glds: []
    property var gmap: ({})
    property var chans: ({})
    property var rels: []
    property var pres: ({})
    property var unr: ({})
    property var vs: []
    property var call: ({})
    property var names: ({})
    property var typing: ({})
    property var cfgAccs: []
    property int ver: 0
    property int uver: 0

    property string sel: "home"
    property string selC: ""
    property string selC2: ""
    property var lastC: ({})
    property string reply: ""
    property string editing: ""
    property string qr: ""
    property string qrUsr: ""
    property var errs: []

    property var ui: ({})
    property var lay: L.cp(L.presets.classic)
    property bool edit: false
    property bool setup: false
    property bool showSet: false
    property string setPage: "accounts"
    property var feats: ({})
    property var blks: ({})
    property var hooks: ({ render: [], send: [], ev: [] })
    property var mods: []

    signal ev(var d)
    signal msg(string chan, var m, bool isNew)
    signal msgDel(string chan, string id, bool gone)
    signal hist(string chan, int n, bool more)
    signal fs(string path)

    readonly property var featDefs: [
        { k: "msglog", n: "Message logger", d: "Keep deleted messages and edit history visible", on: true },
        { k: "hidden", n: "Show hidden channels", d: "List channels you cannot open, marked with a lock", on: true },
        { k: "fakenitro", n: "Emoji and sticker passthrough", d: "Send any emoji or sticker as a link when you cannot use it natively", on: true },
        { k: "streamq", n: "Stream quality unlock", d: "Pick any resolution and frame rate for screen share", on: true },
        { k: "silenttyping", n: "Silent typing", d: "Do not send typing indicators", on: false },
        { k: "noread", n: "Do not send read receipts", d: "Read messages without marking them as read for others", on: false },
        { k: "platform", n: "Platform indicators", d: "Show desktop, mobile or web next to names", on: true },
        { k: "spoilers", n: "Reveal spoilers", d: "Show spoiler text without clicking", on: false },
        { k: "notrack", n: "Block tracking", d: "Drop analytics and telemetry requests", on: true },
        { k: "translate", n: "Translate", d: "Translate messages from the context menu", on: true },
        { k: "spotify", n: "Media controls", d: "Player bar for Spotify and other MPRIS players", on: true },
        { k: "relnotif", n: "Relationship notifier", d: "Tell you when someone removes you as a friend or a server removes you", on: true },
        { k: "typingtweaks", n: "Typing indicators everywhere", d: "Show who types in the channel list and server bar", on: true },
        { k: "callTimer", n: "Call timer", d: "Show how long you are in a call", on: true },
        { k: "copyraw", n: "Copy raw message", d: "Context menu entry to copy the unformatted message", on: true },
        { k: "readall", n: "Read all", d: "One click to mark every channel as read", on: true },
        { k: "pinDms", n: "Pin DMs", d: "Keep chosen DMs on top of the list", on: true },
        { k: "noreplyping", n: "No reply mention", d: "Replies do not ping by default", on: false },
        { k: "anim", n: "Animations", d: "Interface motion", on: true }
    ]

    function f(k) {
        if (k in feats) return feats[k]
        const d = featDefs.find(x => x.k === k)
        return d ? d.on : false
    }
    function setF(k, on) {
        const o = Object.assign({}, feats)
        o[k] = on
        feats = o
        c.feat(k, on)
    }

    function svc(id) { return id ? id.split(":")[0] : "dc" }
    function acc(id) { return id ? parseInt(id.split(":")[1]) : 0 }
    function raw(id) { return id ? id.split(":").slice(2).join(":") : "" }
    function rid(sv, a, r) { return sv + ":" + a + ":" + r }
    function ridOf(o) { return o ? (o.svc === "Mx" ? "mx" : o.svc === "St" ? "st" : "dc") + ":" + o.acc + ":" + o.raw : "" }
    function tag(sv) { return sv === "Mx" ? "mx" : sv === "St" ? "st" : "dc" }

    function boot() {
        const u = JSON.parse(c.cget("ui.json"))
        ui = u || {}
        if (ui.lay) lay = ui.lay
        const cf = JSON.parse(c.cget("cfg.json")) || {}
        feats = cf.feat || {}
        cfgAccs = JSON.parse(c.accs())
        setup = !u || !u.done
        loadTheme()
        c.boot()
        refresh()
        scanMods()
    }

    function saveUi() {
        const u = Object.assign({}, ui)
        u.lay = lay
        ui = u
        c.cset("ui.json", JSON.stringify(u))
    }
    function setUi(k, v) {
        const u = Object.assign({}, ui)
        u[k] = v
        ui = u
        c.cset("ui.json", JSON.stringify(u))
    }
    function setLay(l) {
        lay = L.norm(L.cp(l))
        saveUi()
    }

    function themes() {
        const b = JSON.parse(c.ls("themes", "json"))
        const q = c.qml() + "/Cord/themes/"
        return ["graphite", "paper", "mono", "caelestia"].map(n => ({ id: n, path: n === "caelestia" ? "" : q + n + ".json", user: false }))
            .concat(b.map(p => ({ id: p.split("/").pop().replace(".json", ""), path: p, user: true })))
    }
    function loadTheme() {
        const n = ui.theme || "graphite"
        const t = themes().find(x => x.id === n) || themes()[0]
        const T = Th
        T.apply(JSON.parse(c.rdf(c.qml() + "/Cord/themes/graphite.json") || "{}"))
        if (t.id === "caelestia") {
            const h = c.home().replace(/\/\.config\/cord$/, "")
            const j = c.rdf(h + "/.local/state/caelestia/scheme.json")
            if (j) T.fromScheme(JSON.parse(j))
        } else if (t.path) {
            const j = c.rdf(t.path)
            if (j) T.apply(JSON.parse(j))
        }
        if (ui.fs) T.fs = ui.fs
        if (ui.dens) T.dens = ui.dens
        if (ui.ac) T.ac = ui.ac
    }

    function refresh() {
        const j = JSON.parse(c.snap())
        const am = {}
        for (const a of j.accs || []) am[a.id] = a
        accs = am
        up = j.up || []
        const nm = {}
        const gm = {}
        for (const g of j.glds || []) {
            const id = ridOf(g.id)
            g.rid = id
            gm[id] = g
            for (const r of g.roles || []) nm[raw(ridOf(r.id))] = r.name
        }
        const order = ui.gorder || []
        const gl = Object.values(gm).sort((a, b) => {
            const ia = order.indexOf(a.rid), ib = order.indexOf(b.rid)
            if (ia >= 0 || ib >= 0) return (ia < 0 ? 1e9 : ia) - (ib < 0 ? 1e9 : ib)
            return a.pos - b.pos || a.rid.localeCompare(b.rid)
        })
        const cm = {}
        for (const ch of j.chans || []) {
            const id = ridOf(ch.id)
            ch.rid = id
            ch.grid = ch.gld ? ridOf(ch.gld) : ""
            ch.prid = ch.par ? ridOf(ch.par) : ""
            cm[id] = ch
            nm[raw(id)] = ch.name
        }
        for (const r of j.rels || []) if (r.usr.id) nm[raw(ridOf(r.usr.id))] = r.usr.disp || r.usr.name
        names = nm
        gmap = gm
        glds = gl
        chans = cm
        rels = j.rels || []
        pres = j.pres || {}
        unr = j.unr || {}
        vs = j.vs || []
        call = j.call || {}
        ver++
        if (sel !== "home" && !selC && gm[sel]) Qt.callLater(() => { if (!selC) pick(sel) })
    }

    property Timer deb: Timer {
        interval: 60
        onTriggered: s.refresh()
    }
    property Timer typT: Timer {
        interval: 1000
        repeat: true
        running: true
        onTriggered: {
            const n = Date.now()
            let ch = false
            const t = Object.assign({}, s.typing)
            for (const k in t) {
                const o = {}
                for (const u in t[k]) if (t[k][u].until > n) o[u] = t[k][u]; else ch = true
                t[k] = o
            }
            if (ch) s.typing = t
        }
    }

    function on(d) {
        switch (d.t) {
        case "Msg": {
            const ck = d.d.chan
            const m = d.d.m
            run("render", m)
            if (s.typing[ck]) {
                const t = Object.assign({}, s.typing)
                const o = Object.assign({}, t[ck])
                delete o[m.auth.name]
                t[ck] = o
                s.typing = t
            }
            s.msg(ck, m, d.d.new)
            if (d.d.new && !isMe(m.auth, acc(ck)) && ui.notif !== false && ui.st !== "Dnd") {
                const ch = chans[ck]
                const dm = ch && (ch.k === "Dm" || ch.k === "Grp")
                const me_ = me(acc(ck))
                const men = me_ && me_.id && (m.body || "").indexOf(me_.id.raw) >= 0
                const focused = Qt.application.state === Qt.ApplicationActive && (selC === ck || selC2 === ck)
                if ((dm || men) && !focused) c.notify((m.auth.disp || m.auth.name) + (ch && ch.grid && gmap[ch.grid] ? " (#" + ch.name + ", " + gmap[ch.grid].name + ")" : ""), (m.body || (m.atts && m.atts.length ? "Sent an attachment" : "")).slice(0, 200))
            }
            if (d.d.new && chans[ck]) {
                const c2 = Object.assign({}, chans)
                c2[ck] = Object.assign({}, c2[ck], { last: m.id })
                chans = c2
            }
            break
        }
        case "MsgDel": s.msgDel(d.d.chan, d.d.id, d.d.gone); break
        case "Typing": {
            const t = Object.assign({}, s.typing)
            const o = Object.assign({}, t[d.d.chan] || {})
            o[d.d.usr.name] = { n: d.d.usr.disp || d.d.usr.name, until: Date.now() + 9000 }
            t[d.d.chan] = o
            s.typing = t
            break
        }
        case "Unr": {
            const u = Object.assign({}, unr)
            if (d.d.u.n || d.d.u.ment) u[d.d.chan] = d.d.u
            else delete u[d.d.chan]
            unr = u
            uver++
            if (d.d.chan === selC && d.d.u.n && !f("noread")) c.ack(selC)
            break
        }
        case "Pres": {
            const p = Object.assign({}, pres)
            p[d.d.usr] = d.d.p
            pres = p
            break
        }
        case "Hist": s.hist(d.d.chan, d.d.n, d.d.more); break
        case "Qr": qr = d.d; break
        case "QrUsr": qrUsr = d.d.name; break
        case "Accs": cfgAccs = JSON.parse(c.accs()); qr = ""; qrUsr = ""; deb.restart(); break
        case "Err": {
            const e = errs.slice(-4)
            e.push({ msg: d.d.msg, at: Date.now() })
            errs = e
            console.warn("cord:", d.d.msg)
            break
        }
        case "Fs": onFs(d.d); break
        case "Speak": {
            const t = Object.assign({}, speaking)
            if (d.d.on) t[d.d.usr] = true
            else delete t[d.d.usr]
            speaking = t
            break
        }
        default: deb.restart()
        }
        s.ev(d)
        for (const h of hooks.ev) try { h(d) } catch (e) { console.warn(e) }
    }
    property var speaking: ({})

    property Timer fsT: Timer {
        interval: 150
        property var ps: ({})
        onTriggered: {
            const p = ps
            ps = {}
            for (const k in p) {
                if (k.indexOf("/themes/") >= 0 || k.endsWith("ui.json")) s.loadTheme()
                if (k.indexOf("/mods/") >= 0 || k.indexOf("/blocks/") >= 0) s.scanMods()
                s.fs(k)
            }
        }
    }
    function onFs(p) {
        if (p.endsWith(".tmp")) return
        const o = fsT.ps
        o[p] = true
        fsT.ps = o
        fsT.restart()
    }

    function run(k, v) {
        for (const h of hooks[k] || []) try { const r = h(v); if (r !== undefined) v = r } catch (e) { console.warn(e) }
        return v
    }
    function hook(k, fn) {
        const h = Object.assign({}, hooks)
        h[k] = (h[k] || []).concat([fn])
        hooks = h
    }
    function regBlk(id, url, name, opt) {
        const b = Object.assign({}, blks)
        b[id] = { url: url, name: name || id, opt: opt || {} }
        blks = b
    }
    function blkUrl(id) {
        if (blks[id]) return blks[id].url
        const o = c.home() + "/blocks/" + id + ".qml"
        if (c.rdf(o).length) return "file://" + o
        return Qt.resolvedUrl("blocks/" + id.charAt(0).toUpperCase() + id.slice(1) + ".qml")
    }
    readonly property var coreBlks: [
        { id: "rail", name: "Server bar" }, { id: "side", name: "Channels and DMs" }, { id: "chat", name: "Chat" },
        { id: "mems", name: "Member list" }, { id: "me", name: "Account panel" }, { id: "call", name: "Call panel" },
        { id: "stage", name: "Call stage" }, { id: "inbox", name: "Inbox" }, { id: "player", name: "Media player" }
    ]
    function allBlks() {
        return coreBlks.concat(Object.keys(blks).map(k => ({ id: k, name: blks[k].name })))
    }
    function scanMods() {
        const ps = JSON.parse(c.ls("mods", ""))
        mods = ps.filter(p => c.rdf(p + "/main.qml").length).map(p => ({ id: p.split("/").pop(), url: "file://" + p + "/main.qml", on: (ui.mods || {})[p.split("/").pop()] !== false }))
    }

    function pick(g) {
        if (sel !== g) {
            const l = Object.assign({}, lastC)
            l[sel] = selC
            lastC = l
        }
        sel = g
        if (g === "home") { open(lastC.home || ""); return }
        const last = lastC[g]
        if (last && chans[last]) { open(last); return }
        const cs = chList(g).filter(x => x.k === "Txt" || x.k === "Ann" || x.k === "Forum")
        open(cs.length ? cs[0].rid : "")
    }
    function open(cid, slot) {
        if (slot === 1) { selC2 = cid; return }
        selC = cid
        reply = ""
        editing = ""
        if (cid && unr[cid] && !f("noread")) c.ack(cid)
    }
    function chList(g) {
        void ver
        const all = Object.values(chans).filter(x => x.grid === g)
        const pos = x => x.pos * 10 + (x.k === "Voice" || x.k === "Stage" ? 5 : 0)
        const cats = all.filter(x => x.k === "Cat").sort((a, b) => a.pos - b.pos)
        const top = all.filter(x => x.k !== "Cat" && !x.prid && x.k !== "Thread").sort((a, b) => pos(a) - pos(b))
        const out = top.slice()
        for (const ct of cats) {
            out.push(ct)
            out.push(...all.filter(x => x.prid === ct.rid && x.k !== "Thread").sort((a, b) => pos(a) - pos(b)))
        }
        return out
    }
    function dms() {
        void ver
        const pinned = ui.pins || []
        return Object.values(chans).filter(x => x.k === "Dm" || (x.k === "Grp" && !x.grid))
            .sort((a, b) => {
                const pa = pinned.indexOf(a.rid) >= 0, pb = pinned.indexOf(b.rid) >= 0
                if (pa !== pb) return pa ? -1 : 1
                return (b.last ? lastTs(b) : 0) - (a.last ? lastTs(a) : 0)
            })
    }
    function lastTs(ch) {
        const r = ch.last && ch.last.raw
        if (!r) return 0
        if (ch.rid.startsWith("dc") && /^\d+$/.test(r)) return Math.floor(Number(r) / 4194304) + 1420070400000
        return parseInt(r.replace(/\D/g, "")) || 0
    }
    function gUnr(g) {
        void uver
        let n = 0, m = 0
        for (const k in unr) {
            const ch = chans[k]
            if (ch && ch.grid === g) { n += unr[k].n ? 1 : 0; m += unr[k].ment }
        }
        return { n: n, m: m }
    }
    function dmUnr() {
        void uver
        return dms().filter(x => unr[x.rid] && (unr[x.rid].n || unr[x.rid].ment))
    }
    function chName(cid) {
        const ch = chans[cid]
        if (!ch) return ""
        if (ch.k === "Dm") return ch.rcp && ch.rcp.length ? (ch.rcp[0].disp || ch.rcp[0].name) : ch.name
        if (ch.k === "Grp" && !ch.name) return (ch.rcp || []).map(u => u.disp || u.name).join(", ")
        return ch.name
    }
    function me(a) { return accs[a] ? accs[a].me : null }
    function presOf(u) { return u ? pres[typeof u === "string" ? u : ridOf(u)] || null : null }
    function vsIn(cid) {
        return vs.filter(v => v.chan && ridOf(v.chan) === cid)
    }
    function readAll() {
        for (const k in unr) c.ack(k)
    }
    function send(cid, txt, files) {
        if (!cid) return
        let o = { body: txt, files: files || [] }
        o = run("send", o)
        if (!o || (!o.body && !(o.files || []).length)) return
        if (editing) { c.edit(cid, editing, o.body); editing = ""; return }
        c.send(cid, o.body, reply, JSON.stringify(o.files || []), !!o.silent)
        reply = ""
    }
    function setW(p, v, k) {
        const l = L.cp(lay)
        L.at(l, p)[k] = v
        lay = l
        saveUi()
    }
    property var drafts: ({})
    signal profReq(var u, var at)
    signal emoReq(var at, var cb)
    signal viewReq(string url, bool vid)
    function prof(u, at) { profReq(u, at) }
    function emoji(at, cb) { emoReq(at, cb) }
    function view(u, v) { viewReq(u, !!v) }
    property var media: null
    function goLive() {
        const h = ui.sh || 1080
        const q = f("streamq") ? { w: Math.round(h * 16 / 9), h: h, fps: ui.sfps || 60, kbps: h >= 1440 ? 16000 : 8000, snd: ui.ssnd !== false } : { w: 1280, h: 720, fps: 30, kbps: 4000, snd: ui.ssnd !== false }
        c.golive(true, JSON.stringify(q))
    }
    function u(x) {
        if (!x) return ""
        if (x.startsWith("mxc://") || x.startsWith("mxe:")) return "image://cordm/0/" + x
        return x
    }
    property Timer medT: Timer {
        interval: 2000
        repeat: true
        running: s.f("spotify")
        onTriggered: { const j = s.c.mpris(); s.media = j ? JSON.parse(j) : null }
    }
    property bool qk: false
    function quick() { qk = true }
    function copy(t) { c.copy(t) }
    function isMe(u, a) {
        const m = me(a)
        return !!(m && m.id && u && u.id && m.id.raw === u.id.raw)
    }
    function toggleBlk(b, on) { setLay(L.toggle(lay, b, on)) }
    function sideBlk(b, sd) { setLay(L.side(lay, b, sd)) }
    function hasBlk(b) { return L.has(lay, b) }
    function sideOf(b) { return L.sideOf(lay, b) }
    function preset(k) { setLay(L.presets[k]) }
}
