pragma Singleton
import QtQuick
import cord.core
import "Lay.js" as L

QtObject {
    id: s

    readonly property int apiVer: 1

    property Core c: Core {
        onDl: d => s.onBatch(d)
        onFrameTick: (k, n) => s.tick(k, n)
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
    property var frameSeq: ({})
    property int fver: 0
    property int pver: 0
    property string uiBad: ""
    property var cfgAccs: []
    readonly property bool multi: new Set(Object.values(accs).map(a => a.svc)).size > 1
    property var services: []
    property string secretBackend: ""
    property int pinReq: 0
    property string insReq: ""
    property var profs: ({})
    property var profT: ({})
    property var sheets: ({})
    property var apiCb: ({})
    property int apiN: 0
    function api(acc, op, args, cb) {
        const t = "a" + (++apiN)
        if (cb) apiCb[t] = cb
        c.api(acc, op, JSON.stringify(args || {}), t)
    }
    property var sheetQ: ({})
    function reqProf(uid, gld) {
        if (!uid || svc(uid) !== "dc") return
        const k = uid + "|" + (gld || "")
        const now = Date.now()
        if (profT[k] && now - profT[k] < 300000) return
        profT[k] = now
        c.profile(uid, gld || "")
    }
    function profOf(uid, gld) {
        const p = profs[uid + "|" + (gld || "")] || profs[uid + "|"]
        return p || null
    }
    function roleIcon(gld, uid, roles) {
        if (!gld) return null
        const g = gmap[gld]
        if (!g || !g.roles) return null
        let rs = roles
        if (!rs) { const m = (U.memC[gld] || {})[uid]; rs = m ? m.roles : null }
        if (!rs || !rs.length) return null
        const ids = rs.map(x => typeof x === "string" ? x : ridOf(x))
        let best = null
        for (const ro of g.roles) if (ro.icon && ids.indexOf(ridOf(ro.id)) >= 0 && (!best || ro.pos > best.pos)) best = ro
        return best
    }
    function decId(s) {
        const a = String(s).split("")
        let i = a.length - 1
        while (i >= 0) {
            if (a[i] !== "0") { a[i] = String(Number(a[i]) - 1); break }
            a[i] = "9"
            i--
        }
        return a.join("").replace(/^0+(?=\d)/, "")
    }
    function unproxy(u) {
        const m = /^https:\/\/(?:images-ext-\d+|media)\.discordapp\.net\/external\/[^/]+\/(https?)\/(.+)$/.exec(u || "")
        return m ? m[1] + "://" + m[2] : u
    }
    function pow2(n) { let p = 16; while (p < n && p < 512) p *= 2; return p }
    function decoUrl(a, sz, anim) {
        return a ? "https://cdn.discordapp.com/avatar-decoration-presets/" + a + ".png?size=" + pow2(sz || 96) + "&passthrough=" + (anim ? "true" : "false") : ""
    }
    function sheetOf(url, max) {
        if (!url) return null
        const k = url + "|" + (max || 0)
        if (sheets[k]) return sheets[k]
        if (!sheetQ[k]) {
            sheetQ[k] = true
            Qt.callLater(() => c.sheet(url, max || 0))
        }
        return null
    }
    property int ver: 0
    property int uver: 0
    property int dver: 0
    property int nver: 0
    property real snapSeq: 0
    property real lastSeq: 0
    property var hookFails: ({})
    property var hookOff: ({})

    property string sel: "home"
    property string selC: ""
    property string selC2: ""
    property var lastC: ({})
    property int focusSlot: 0
    property var slotSt: ({})
    property string reply: ""
    property string editing: ""
    property string qr: ""
    property string qrUsr: ""
    property var cap: null
    property var folds: []
    property var foldOpen: ({})
    readonly property var rail: {
        const used = {}, out = []
        const byId = {}
        for (const g of glds) byId[g.rid] = g
        for (const f of folds) {
            const gs = f.glds.map(x => byId[ridOf(x)]).filter(g => g && !used[g.rid])
            if (!gs.length) continue
            for (const g of gs) used[g.rid] = true
            if (!f.id) { for (const g of gs) out.push({ g: g }); continue }
            out.push({ f: f, gs: gs })
            if (foldOpen[f.id]) gs.forEach((g, i) => out.push({ g: g, inF: f, last: i === gs.length - 1 }))
        }
        for (const g of glds) if (!used[g.rid]) out.push({ g: g })
        return out
    }
    property string lastFold: ""
    property real foldT: 0
    function toggleFold(id) {
        lastFold = id
        foldT = Date.now()
        const o = Object.assign({}, foldOpen)
        o[id] = !o[id]
        foldOpen = o
        setUi("foldOpen", o)
    }
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
    property var featDefs: []

    signal ev(var d)
    signal msg(string chan, var m, bool isNew)
    signal msgDel(string chan, string id, bool gone)
    signal hist(string chan, int n, bool more, bool ok, var list, string err, bool noop)
    signal loaded(string chan, var list, bool synced)
    signal frameTick(string key, double seq)
    signal mems(string gld, var list)
    signal memList(string gld, var rows)
    signal sent(string chan, bool ok, string err)
    signal dmOpened(string usr, string chan, bool ok, string err)
    signal fs(string path)
    signal resync()

    function f(k) {
        if (k in feats) return feats[k]
        const d = featDefs.find(x => x.k === k)
        return d ? d.on : false
    }
    function setF(k, on) {
        const o = Object.assign({}, feats)
        o[k] = on
        feats = o
        if (k === "anim") Th.motion = on
        c.feat(k, on)
    }

    function svc(id) { return id ? id.split(":")[0] : "dc" }
    function acc(id) { return id ? parseInt(id.split(":")[1]) : 0 }
    function raw(id) { return id ? id.split(":").slice(2).join(":") : "" }
    function rid(sv, a, r) { return sv + ":" + a + ":" + r }
    function ridOf(o) { return o ? (o.rid || tag(o.svc) + ":" + o.acc + ":" + o.raw) : "" }
    function tag(sv) { const t = String(sv || "").toLowerCase(); return t === "mx" || t === "st" ? t : "dc" }

    readonly property var noCaps: ({ call: false, permalink: "", fakeEmoji: false, threads: false })
    function caps(id) {
        const ch = chans[id]
        return ch && ch.caps ? ch.caps : noCaps
    }

    function pj(txt, d) {
        if (txt === undefined || txt === null || txt === "") return d
        try {
            const v = JSON.parse(txt)
            return v === null || v === undefined ? d : v
        } catch (e) {
            console.warn("cord: bad json", e)
            return d
        }
    }

    function blkIds() {
        const ids = coreBlks.map(b => b.id)
        for (const k in blks) ids.push(k)
        for (const k of ui.kb || []) ids.push(k)
        for (const p of pj(c.ls("blocks", "qml"), [])) ids.push(p.split("/").pop().replace(/\.qml$/, ""))
        for (const p of pj(c.ls("mods", ""), [])) ids.push(p.split("/").pop())
        return ids
    }

    function boot() {
        featDefs = pj(c.features(), [])
        services = pj(c.services(), [])
        const u = pj(c.cget("ui.json"), null)
        uiBad = c.bad("ui.json")
        ui = u && typeof u === "object" ? u : {}
        if (ui.lay) lay = L.norm(L.safe(L.cp(ui.lay), blkIds()))
        if (ui.lay && ui.lay.name === "Classic" && !L.has(lay, "bar")) { lay = L.cp(L.presets.classic); saveUi() }
        const cf = pj(c.cget("cfg.json"), {})
        feats = cf.feat || {}
        cfgAccs = pj(c.accs(), [])
        setup = !uiBad && (!u || !u.done)
        loadTheme()
        c.boot()
        if (ui.st && ui.st !== "On") c.set_st(ui.st.toLowerCase(), "")
        bootSnap()
        secretBackend = c.secretBackend()
        scanMods()
    }

    function secretOf(a) { return c.secretOf(a) }

    function reloadUi() {
        const u = pj(c.cget("ui.json"), null)
        uiBad = c.bad("ui.json")
        if (!u || typeof u !== "object") return
        ui = u
        if (u.lay) lay = L.norm(L.safe(L.cp(u.lay), blkIds()))
    }

    function writeUi(u) {
        if (uiBad) {
            pushErr("ui.json", "ui.json on disk is not valid JSON, settings are not saved until it is fixed: " + uiBad, 0)
            return false
        }
        c.cset("ui.json", JSON.stringify(u))
        return true
    }
    function saveUi() {
        const u = Object.assign({}, ui)
        u.lay = lay
        ui = u
        writeUi(u)
    }
    function setUi(k, v) {
        const u = Object.assign({}, ui)
        u[k] = v
        ui = u
        writeUi(u)
    }
    function setLay(l) {
        const n = L.norm(L.cp(l))
        if (!L.valid(n, blkIds())) {
            console.warn("cord: rejected invalid layout")
            return
        }
        lay = n
        saveUi()
    }

    function themes() {
        const b = pj(c.ls("themes", "json"), [])
        const q = c.qml() + "/Cord/themes/"
        return ["midnight", "graphite", "paper", "mono"].concat(c.scheme() ? ["caelestia"] : []).map(n => ({ id: n, path: n === "caelestia" ? "" : q + n + ".json", user: false }))
            .concat(b.map(p => ({ id: p.split("/").pop().replace(".json", ""), path: p, user: true })))
    }
    property Connections schemeWatch: Connections {
        target: Qt.styleHints
        function onColorSchemeChanged() { if (s.ui.sys) s.loadTheme() }
    }
    function loadTheme() {
        const T = Th
        try {
            let n = ui.theme || "midnight"
            if (ui.sys) n = Qt.styleHints.colorScheme === Qt.ColorScheme.Light ? "paper" : (n === "paper" ? "midnight" : n)
            const t = themes().find(x => x.id === n) || themes()[0]
            T.apply(pj(c.rdf(c.qml() + "/Cord/themes/graphite.json"), {}))
            if (t.id === "caelestia") {
                const j = c.scheme()
                if (j) T.fromScheme(pj(j, {}))
            } else if (t.path) {
                const j = c.rdf(t.path)
                if (j) T.apply(pj(j, {}))
            }
            if (ui.fs) T.fs = ui.fs
            if (ui.dens) T.dens = ui.dens
            if (ui.ac) T.ac = ui.ac
            T.motion = f("anim")
        } catch (e) {
            console.warn("cord: theme failed", e)
        }
    }

    property var nm: ({})
    property bool dirtyC: false
    property bool dirtyG: false
    property bool dirtyN: false

    function sortG() {
        const order = ui.gorder || []
        glds = Object.values(gmap).sort((a, b) => {
            const ia = order.indexOf(a.rid), ib = order.indexOf(b.rid)
            if (ia >= 0 || ib >= 0) return (ia < 0 ? 1e9 : ia) - (ib < 0 ? 1e9 : ib)
            return a.pos - b.pos || a.rid.localeCompare(b.rid)
        })
    }

    property Timer flushT: Timer {
        interval: 40
        onTriggered: s.flush()
    }
    function flush() {
        if (dirtyC) { chans = Object.assign({}, chans); dirtyC = false; ver++ }
        if (dirtyG) { gmap = Object.assign({}, gmap); sortG(); dirtyG = false; ver++ }
        if (dirtyN) { names = Object.assign({}, nm); dirtyN = false; nver++ }
        if (sel !== "home" && !selC && gmap[sel]) pick(sel)
    }
    function mark(c_, g_, n_) {
        if (c_) dirtyC = true
        if (g_) dirtyG = true
        if (n_) dirtyN = true
        flushT.restart()
    }

    function putGld(g) {
        gmap[g.rid] = g
        for (const r of g.roles || []) nm[raw(ridOf(r.id))] = r.name
        mark(false, true, true)
    }
    function putChan(ch) {
        chans[ch.rid] = ch
        nm[raw(ch.rid)] = ch.name
        mark(true, false, true)
    }
    function dropChanState(k) {
        if (selC === k) selC = ""
        if (selC2 === k) selC2 = ""
        if (typing[k]) { delete typing[k]; typingChanged() }
        if (unr[k]) { delete unr[k]; unrChanged(); uver++ }
    }
    function delChan(id) {
        dropChanState(id)
        delete chans[id]
        mark(true, false, false)
    }

    function bootSnap() {
        const j = pj(c.snap(), {})
        snapSeq = j.seq || 0
        lastSeq = snapSeq
        const am = {}
        for (const a of j.accs || []) am[a.id] = a
        accs = am
        up = j.up || []
        gmap = {}
        chans = {}
        nm = {}
        for (const g of j.glds || []) putGld(g)
        for (const ch of j.chans || []) putChan(ch)
        for (const r of j.rels || []) if (r.usr.id) nm[raw(ridOf(r.usr.id))] = r.usr.disp || r.usr.name
        rels = j.rels || []
        folds = j.folds || []
        foldOpen = ui.foldOpen || {}
        pres = j.pres || {}
        unr = j.unr || {}
        vs = j.vs || []
        call = j.call || {}
        flushT.stop()
        flush()
        dver++
        uver++
        if (sel !== "home" && !selC && gmap[sel]) Qt.callLater(() => { if (!selC) pick(sel) })
    }
    function refresh() { bootSnap() }

    property Timer typT: Timer {
        interval: 1000
        repeat: true
        running: true
        onTriggered: {
            const n = Date.now()
            let ch = false
            const t = s.typing
            for (const k in t) {
                for (const u in t[k]) if (t[k][u].until <= n) { delete t[k][u]; ch = true }
                if (!Object.keys(t[k]).length) delete t[k]
            }
            if (ch) s.typingChanged()
        }
    }

    property var rcache: ({})
    property int rcount: 0
    function rsig(m) {
        if (m.rev !== undefined) return "r" + m.rev + "|" + rsig0(m, 0) + "|" + (m.auth && m.auth.deco ? m.auth.deco : "")
        let n = 0
        const a = m.auth || {}
        const ea = JSON.stringify(m.embs || []).length + ":" + JSON.stringify(m.atts || []).length
        return (a.id ? a.id.raw : "") + "|" + a.name + "|" + (a.disp || "") + "|" + (a.av || "") + "|" + ea + "|" + rsig0(m, n)
    }
    function rsig0(m, n) {
        for (const r of m.reacts || []) n += r.n + (r.me ? 1000 : 0)
        return (m.ed || 0) + "|" + (m.body || "").length + "|" + n + "|" + (m.atts || []).length + "|" + (m.embs || []).length + "|" + (m.hist || []).length + "|" + (m.del ? 1 : 0) + (m.pin ? 1 : 0)
    }
    function learn(us) {
        let ch = false
        for (const u of us) {
            if (!u || !u.id) continue
            const k = raw(ridOf(u.id)), n = u.disp || u.name
            if (n && nm[k] !== n) { nm[k] = n; ch = true }
        }
        if (ch) mark(false, false, true)
    }
    function rm(m) {
        if (!m || m._r) return m
        learn((m.ments || []).concat(m.auth ? [m.auth] : []))
        const id = ridOf(m.id)
        const sg = rsig(m)
        const hit = rcache[id]
        if (hit && hit.sg === sg) return hit.v
        const r = run("render", m) || m
        r._r = true
        if (rcount > 3000) { rcache = {}; rcount = 0 }
        rcache[id] = { sg: sg, v: r }
        rcount++
        return r
    }
    function rml(l) { return (l || []).map(rm) }

    function visible(cid) {
        if (Qt.application.state !== Qt.ApplicationActive) return false
        if (cid === selC && L.has(lay, "chat")) return true
        return cid === selC2 && slotShown(lay, 1)
    }
    function slotShown(n, sl) {
        if (n.b === "chat" && n.o && n.o.slot === sl) return true
        return (n.ch || []).some(x => slotShown(x, sl))
    }

    function handleMsg(d) {
        const ck = d.chan
        const m = rm(d.m)
        if (typing[ck] && typing[ck][m.auth.name]) {
            delete typing[ck][m.auth.name]
            typingChanged()
        }
        const ch = chans[ck]
        if (d.new && ch) {
            ch.last = m.id
            ch.lastTs = m.ts
            if (ch.k === "Dm" || ch.k === "Grp") dver++
        }
        msg(ck, m, d.new)
        if (d.new && !isMe(m.auth, acc(ck)) && ui.notif !== false && ui.st !== "Dnd") {
            const dm = ch && (ch.k === "Dm" || ch.k === "Grp")
            const me_ = me(acc(ck))
            const men = me_ && me_.id && (m.body || "").indexOf(me_.id.raw) >= 0
            const focused = Qt.application.state === Qt.ApplicationActive && (selC === ck || selC2 === ck)
            if ((dm || men) && !focused) c.notify((m.auth.disp || m.auth.name) + (ch && ch.grid && gmap[ch.grid] ? " (#" + ch.name + ", " + gmap[ch.grid].name + ")" : ""), (m.body || (m.atts && m.atts.length ? "Sent an attachment" : "")).slice(0, 200))
        }
    }

    function relDiff(nr) {
        if (!f("relnotif") || !rels.length) return
        const ids = new Set(nr.filter(x => x.k === "Friend" && x.usr.id).map(x => ridOf(x.usr.id)))
        for (const o of rels) if (o.k === "Friend" && o.usr.id && !ids.has(ridOf(o.usr.id)) && accs[o.usr.id.acc]) c.notify("Friend removed", (o.usr.disp || o.usr.name) + " is no longer on your friends list")
    }

    function purgeAcc(a) {
        const keep = (obj, key) => {
            const o = {}
            for (const k in obj) if (key(k) !== a) o[k] = obj[k]
            return o
        }
        const am = Object.assign({}, accs)
        delete am[a]
        accs = am
        gmap = keep(gmap, k => acc(k))
        chans = keep(chans, k => acc(k))
        pres = keep(pres, k => acc(k))
        unr = keep(unr, k => acc(k))
        typing = keep(typing, k => acc(k))
        rels = rels.filter(r => !r.usr.id || r.usr.id.acc !== a)
        vs = vs.filter(v => !v.usr.id || v.usr.id.acc !== a)
        up = up.filter(x => x !== a)
        if (call.chan && call.chan.acc === a) call = {}
        if (acc(selC) === a) selC = ""
        if (acc(selC2) === a) selC2 = ""
        if (sel !== "home" && acc(sel) === a) sel = "home"
        sortG()
        ver++
        uver++
        dver++
    }

    function pushErr(ctx, msg, a) {
        const e = errs.slice(-4)
        e.push({ msg: msg, ctx: ctx, acc: a, at: Date.now() })
        errs = e
        console.warn("cord:", ctx, msg)
    }

    function tick(k, n) {
        frameSeq[k] = n
        fver++
        frameTick(k, n)
    }

    property var ackQ: ({})
    property Timer ackT: Timer {
        interval: 400
        onTriggered: {
            if (Qt.application.state !== Qt.ApplicationActive) {
                if (Object.keys(s.ackQ).length) restart()
                return
            }
            const q = s.ackQ
            s.ackQ = {}
            for (const k in q) if (s.unr[k]) s.c.ack(k)
        }
    }
    property Connections appCon: Connections {
        target: Qt.application
        function onStateChanged() {
            if (Qt.application.state !== Qt.ApplicationActive) return
            for (const cid of [s.selC, s.selC2]) if (cid && s.unr[cid] && s.visible(cid)) s.ackReq(cid)
            if (Object.keys(s.ackQ).length) s.ackT.restart()
        }
    }
    function ackReq(cid) {
        if (!cid) return
        ackQ[cid] = true
        ackT.restart()
    }

    function onBatch(txt) {
        let v
        try { v = JSON.parse(txt) } catch (e) { console.warn("cord: bad batch", e); return }
        if (Array.isArray(v)) for (const d of v) feed(d)
        else feed(v)
    }
    function feed(d) {
        const q = d.seq || 0
        if (q) {
            if (q <= snapSeq) return
            if ((d.from || q) !== lastSeq + 1) {
                console.warn("cord: stream gap, resnapshot", lastSeq, d.from || q)
                bootSnap()
                resync()
                return
            }
            lastSeq = q
        }
        on(d)
    }

    function on(d) {
        const p = d.d
        switch (d.t) {
        case "Msg": handleMsg(p); break
        case "MsgDel": msgDel(p.chan, p.id, p.gone); break
        case "Msgs": loaded(p.chan, rml(p.list), !!p.synced); break
        case "Hist": hist(p.chan, p.n, p.more, p.ok, rml(p.list), p.err || "", !!p.noop); break
        case "Sent":
            if (!p.ok) pushErr("send", p.err, acc(p.chan))
            sent(p.chan, p.ok, p.err)
            break
        case "Dm": {
            dmOpened(p.usr, p.chan, p.ok, p.err)
            if (p.ok) { pick("home"); open(p.chan) }
            if (p.ok && dmPend[p.usr]) { const t = dmPend[p.usr]; delete dmPend[p.usr]; Qt.callLater(() => send(p.chan, t, [], 0)) }
            else pushErr("dm", p.err, acc(p.usr))
            break
        }
        case "Acc": {
            const am = Object.assign({}, accs)
            am[p.id] = p
            accs = am
            break
        }
        case "Conn": {
            const u = up.filter(x => x !== p.acc)
            if (p.up) u.push(p.acc)
            up = u
            break
        }
        case "Gld": putGld(p); break
        case "GldDel": {
            const g = gmap[p]
            if (g && f("relnotif") && accs[acc(p)]) c.notify("Server removed", "You are no longer in " + g.name)
            delete gmap[p]
            for (const k in chans) if (chans[k].grid === p) { dropChanState(k); delete chans[k] }
            mark(true, true, false)
            if (sel === p) pick("home")
            break
        }
        case "Chan": putChan(p); break
        case "ChanDel": delChan(p); break
        case "Chans": {
            const o = {}
            for (const ch of p) { o[ch.rid] = ch; nm[raw(ch.rid)] = ch.name }
            chans = o
            ver++
            mark(false, false, true)
            break
        }
        case "Typing": {
            if (!typing[p.chan]) typing[p.chan] = {}
            typing[p.chan][p.usr.name] = { n: p.usr.disp || p.usr.name, until: Date.now() + 9000 }
            typingChanged()
            break
        }
        case "Unr": {
            if (p.u.n || p.u.ment) unr[p.chan] = p.u
            else delete unr[p.chan]
            unrChanged()
            uver++
            if (p.u.n && visible(p.chan)) ackReq(p.chan)
            break
        }
        case "PresBatch": {
            for (const e of p) pres[e.usr] = e.p
            pver++
            presChanged()
            break
        }
        case "Rels": {
            relDiff(p)
            for (const r of p) if (r.usr.id) nm[raw(ridOf(r.usr.id))] = r.usr.disp || r.usr.name
            rels = p
            mark(false, false, true)
            break
        }
        case "Mems": learn(p.list.map(x => x.nick ? Object.assign({}, x.usr, { disp: x.nick }) : x.usr)); mems(p.gld, p.list); break
        case "Vs": {
            const id = p.usr.id ? ridOf(p.usr.id) : ""
            const v = vs.filter(x => !x.usr.id || ridOf(x.usr.id) !== id)
            if (p.chan) v.push(p)
            vs = v
            break
        }
        case "Call": call = p; break
        case "Qr": qr = p; break
        case "QrUsr": qrUsr = p.name; break
        case "Cap": cap = p; break
        case "Folds": folds = p; break
        case "Prof": {
            if (!p.ok) break
            const o = Object.assign({}, profs)
            o[p.usr + "|" + (p.gld || "")] = p.p
            profs = o
            break
        }
        case "Api": {
            const cb = apiCb[p.tag]
            delete apiCb[p.tag]
            if (cb) cb(p.ok, p.v, p.err)
            break
        }
        case "Sheet": {
            const o = Object.assign({}, sheets)
            o[p.url + "|" + p.max] = p.ok ? p.s : { err: p.err }
            sheets = o
            break
        }
        case "MemList": memList(p.gld, p.rows); break
        case "Accs": cfgAccs = pj(c.accs(), []); secretBackend = c.secretBackend(); qr = ""; qrUsr = ""; cap = null; break
        case "AccGone": purgeAcc(p.acc); cfgAccs = pj(c.accs(), []); break
        case "Mpris": media = p; break
        case "Err": pushErr(p.ctx, p.msg, p.acc); break
        case "Fs": onFs(p); break
        case "Speak": {
            const t = Object.assign({}, speaking)
            if (p.on) t[p.usr] = true
            else delete t[p.usr]
            speaking = t
            break
        }
        }
        ev(d)
        for (const h of hooks.ev) {
            if (hookOff[h.owner]) continue
            try { h.fn(d) } catch (e) { hookErr(h, e) }
        }
    }
    function hookErr(h, e) {
        console.warn("cord: hook failed", h.owner, e)
        if (h.owner === "core") return
        const n = (hookFails[h.owner] || 0) + 1
        hookFails[h.owner] = n
        if (n >= 5) {
            hookOff[h.owner] = true
            unhook(h.owner)
            pushErr("mod", "Mod " + h.owner + " was disabled after " + n + " failed hook calls: " + e, 0)
        }
    }
    property var speaking: ({})

    property Timer fsT: Timer {
        interval: 150
        property var ps: ({})
        onTriggered: {
            const p = ps
            ps = {}
            for (const k in p) {
                if (k.endsWith("ui.json")) s.reloadUi()
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
        if (k === "render" && v && v._r) return v
        for (const h of hooks[k] || []) {
            if (hookOff[h.owner]) continue
            try { const r = h.fn(v); if (r !== undefined) v = r } catch (e) { hookErr(h, e) }
        }
        return v
    }
    function hook(k, fn, owner, name) {
        owner = owner || "anon"
        if (owner === "core") {
            console.warn("cord: owner core is reserved")
            owner = "mod-core"
        }
        addHook(k, fn, owner, name)
    }
    function hookCore(k, fn, name) { addHook(k, fn, "core", name) }
    function addHook(k, fn, owner, name) {
        const key = name || fn.toString()
        const cur = (hooks[k] || []).filter(h => !(h.owner === owner && h.name === key))
        const h = Object.assign({}, hooks)
        h[k] = cur.concat([{ fn: fn, owner: owner, name: key }])
        hooks = h
        rcache = {}
        rcount = 0
    }
    function unhook(owner) {
        const h = {}
        for (const k in hooks) h[k] = hooks[k].filter(x => x.owner !== owner)
        hooks = h
    }
    function unhookMods() {
        const h = {}
        for (const k in hooks) h[k] = hooks[k].filter(x => x.owner === "core")
        hooks = h
        blks = {}
        rcache = {}
        rcount = 0
        hookFails = {}
        hookOff = {}
    }
    function regBlk(id, url, name, opt) {
        const b = Object.assign({}, blks)
        b[id] = { url: url, name: name || id, opt: opt || {} }
        blks = b
        const kb = ui.kb || []
        if (kb.indexOf(id) < 0) setUi("kb", kb.concat([id]))
    }
    function blkUrl(id) {
        if (blks[id]) return blks[id].url
        const o = c.home() + "/blocks/" + id + ".qml"
        if (c.exists(o)) return "file://" + o
        return Qt.resolvedUrl("blocks/" + id.charAt(0).toUpperCase() + id.slice(1) + ".qml")
    }
    readonly property var coreBlks: [
        { id: "rail", name: "Server bar" }, { id: "side", name: "Channels and DMs" }, { id: "chat", name: "Chat" },
        { id: "mems", name: "Member list" }, { id: "me", name: "Account panel" }, { id: "call", name: "Call panel" },
        { id: "stage", name: "Call stage" }, { id: "inbox", name: "Inbox" }, { id: "player", name: "Media player" }, { id: "bar", name: "Title bar" }
    ]
    function allBlks() {
        return coreBlks.concat(Object.keys(blks).map(k => ({ id: k, name: blks[k].name })))
    }
    function scanMods() {
        const ps = pj(c.ls("mods", ""), [])
        const nmods = ps.filter(p => p.split("/").pop() !== "core" && c.exists(p + "/main.qml")).map(p => ({ id: p.split("/").pop(), url: "file://" + p + "/main.qml", on: (ui.mods || {})[p.split("/").pop()] !== false }))
        unhookMods()
        mods = nmods
    }

    function slot(sl) {
        const k = sl || 0
        return slotSt[k] || {}
    }
    function setSlot(sl, key, v) {
        const o = Object.assign({}, slotSt)
        o[sl] = Object.assign({}, o[sl] || {})
        o[sl][key] = v
        slotSt = o
    }
    property bool _sw: false
    onReplyChanged: {
        if (_sw) return
        _sw = true
        setSlot(focusSlot, "reply", reply)
        if (reply) setSlot(focusSlot, "editing", "")
        reply = ""
        _sw = false
    }
    onEditingChanged: {
        if (_sw) return
        _sw = true
        setSlot(focusSlot, "editing", editing)
        if (editing) setSlot(focusSlot, "reply", "")
        editing = ""
        _sw = false
    }

    function pick(g) {
        if (sel !== g) {
            const l = Object.assign({}, lastC)
            l[sel] = selC
            lastC = l
        }
        sel = g
        if (g === "home") { open(lastC.home && chans[lastC.home] ? lastC.home : ""); return }
        const last = lastC[g]
        if (last && chans[last]) { open(last); return }
        const cs = chList(g).filter(x => x.k === "Txt" || x.k === "Ann" || x.k === "Forum")
        open(cs.length ? cs[0].rid : "")
    }
    function open(cid, sl) {
        if (sl === 1) {
            selC2 = cid
            setSlot(1, "reply", "")
            setSlot(1, "editing", "")
            if (cid && unr[cid]) ackReq(cid)
            return
        }
        selC = cid
        setSlot(0, "reply", "")
        setSlot(0, "editing", "")
        if (cid && unr[cid]) ackReq(cid)
    }
    function chList(g) {
        void ver
        const all = Object.values(chans).filter(x => x.grid === g)
        const pos = x => x.pos * 10 + (x.k === "Voice" || x.k === "Stage" ? 5 : 0)
        const cats = all.filter(x => x.k === "Cat").sort((a, b) => a.pos - b.pos)
        const thr = {}
        for (const t of all) if (t.k === "Thread" && t.prid) (thr[t.prid] = thr[t.prid] || []).push(t)
        const withT = l => {
            const o = []
            for (const c of l) {
                o.push(c)
                for (const t of (thr[c.rid] || []).sort((a, b) => lastTs(b) - lastTs(a)).slice(0, 8)) o.push(Object.assign({}, t, { sub: true }))
            }
            return o
        }
        const top = all.filter(x => x.k !== "Cat" && !x.prid && x.k !== "Thread").sort((a, b) => pos(a) - pos(b))
        const out = withT(top)
        for (const ct of cats) {
            out.push(ct)
            out.push(...withT(all.filter(x => x.prid === ct.rid && x.k !== "Thread").sort((a, b) => pos(a) - pos(b))))
        }
        return out
    }
    function dms() {
        void ver
        void dver
        const pinned = ui.pins || []
        const hid = ui.hidDm || {}
        return Object.values(chans).filter(x => (x.k === "Dm" || (x.k === "Grp" && !x.grid)) && !(x.rid in hid && hid[x.rid] >= lastTs(x) && x.rid !== selC))
            .sort((a, b) => {
                const pa = pinned.indexOf(a.rid) >= 0, pb = pinned.indexOf(b.rid) >= 0
                if (pa !== pb) return pa ? -1 : 1
                return lastTs(b) - lastTs(a)
            })
    }
    function hideDm(rid) {
        const h = Object.assign({}, ui.hidDm || {})
        h[rid] = lastTs(chans[rid])
        setUi("hidDm", h)
        if (selC === rid) open("")
        ver++
    }
    function lastTs(ch) { return ch && ch.lastTs ? ch.lastTs : 0 }
    property var dmPend: ({})
    function dmSend(u, text) {
        const id = typeof u === "string" ? u : ridOf(u && u.id ? u.id : u)
        if (!id || !text.trim()) return
        const d = dms().find(x => x.k === "Dm" && x.rcp && x.rcp[0] && x.rcp[0].id && ridOf(x.rcp[0].id) === id)
        if (d) { pick("home"); open(d.rid); send(d.rid, text, [], 0); return }
        dmPend[id] = text
        c.open_dm(id)
    }
    function openDm(u) {
        const id = typeof u === "string" ? u : ridOf(u && u.id ? u.id : u)
        if (!id) return
        const d = dms().find(x => x.k === "Dm" && x.rcp && x.rcp[0] && x.rcp[0].id && ridOf(x.rcp[0].id) === id)
        if (d) { pick("home"); open(d.rid); return }
        c.open_dm(id)
    }
    readonly property var gu: {
        void uver
        void ver
        const o = {}
        for (const k in unr) {
            const ch = chans[k]
            if (!ch || !ch.grid) continue
            const x = o[ch.grid] || (o[ch.grid] = { n: 0, m: 0 })
            x.n += unr[k].n ? 1 : 0
            x.m += unr[k].ment
        }
        return o
    }
    readonly property var gu0: ({ n: 0, m: 0 })
    function gUnr(g) { return gu[g] || gu0 }
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
    function presOf(u) { void pver; return u ? pres[typeof u === "string" ? u : ridOf(u)] || null : null }
    function vsIn(cid) {
        return vs.filter(v => v.chan && ridOf(v.chan) === cid)
    }
    function readAll() {
        for (const k in unr) c.ack(k)
    }
    function send(cid, txt, files, sl) {
        if (!cid) return false
        sl = sl || 0
        let o = { body: txt, files: files || [], cid: cid }
        o = run("send", o)
        if (!o || (!o.body && !(o.files || []).length)) return false
        const st = slot(sl)
        if (st.editing) c.edit(cid, st.editing, o.body)
        else c.send(cid, o.body, st.reply || "", JSON.stringify(o.files || []), !!o.silent)
        return true
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
        const q = f("streamq") ? { w: Math.round(h * 16 / 9), h: h, fps: ui.sfps || 60, kbps: h >= 1440 ? 16000 : 8000, snd: ui.ssnd !== false && Qt.platform.os !== "osx" } : { w: 1280, h: 720, fps: 30, kbps: 4000, snd: ui.ssnd !== false && Qt.platform.os !== "osx" }
        c.golive(true, JSON.stringify(q))
    }
    function u(x, a) {
        if (!x) return ""
        if (x.startsWith("mxc://") || x.startsWith("mxe:")) return "image://cordm/" + (a || 0) + "/" + x
        return x
    }
    function u2(x, r) { return u(x, acc(r)) }
    property int tick: 0
    property Timer tickT: Timer { interval: 30000; repeat: true; running: true; onTriggered: s.tick++ }
    function plain(t) {
        void nver
        return (t || "").replace(/\n+/g, " ")
            .replace(/<@!?(\d+)>/g, (m, i) => "@" + (names[i] || i))
            .replace(/<@&(\d+)>/g, (m, i) => "@" + (names[i] || "role"))
            .replace(/<#(\d+)>/g, (m, i) => "#" + (names[i] || i))
            .replace(/<a?:(\w+):\d+>/g, ":$1:")
            .replace(/<t:(-?\d+)(?::\w)?>/g, (m, x) => new Date(parseInt(x) * 1000).toLocaleDateString())
            .replace(/\[([^\]]+)\]\((https?:[^)]+)\)/g, "$1")
            .replace(/(\*\*|__|~~|\|\||`)/g, "")
    }
    function thumb(x, w, h, still) {
        if (!x || x.indexOf("cdn.discordapp.com/attachments/") < 0) return still ? "" : u(x)
        const q = "format=webp" + (w > 0 ? "&width=" + Math.round(w) : "") + (h > 0 ? "&height=" + Math.round(h) : "")
        return x.replace("cdn.discordapp.com", "media.discordapp.net") + (x.indexOf("?") >= 0 ? "&" : "?") + q
    }
    function frame(k) { void fver; return "image://cordv/" + k + "?" + (frameSeq[k] !== undefined ? frameSeq[k] : c.frameSeq(k)) }
    property Timer medT: Timer {
        interval: 2000
        repeat: true
        running: s.f("spotify")
        onTriggered: s.c.mpris()
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
