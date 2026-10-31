pragma Singleton
import QtQuick
import Cord
import "Lay.js" as L

QtObject {
    id: u

    property var pop: null
    property var toasts: []
    property var snap: null
    property var undo: []
    property string tipText: ""
    property var tipItem: null
    property var tipBox: ({ x: 0, y: 0, w: 0, h: 0 })
    property string tipSide: "b"
    readonly property bool demo: S.cfgAccs.some(a => a.base === "demo")
    property real callT0: 0
    property string callEl: ""
    readonly property bool inCall: !!(S.call && S.call.chan)
    readonly property bool tipOk: !S.edit && !S.showSet && !S.setup
    signal askReq(var o)

    onInCallChanged: {
        callT0 = Date.now()
        callEl = ""
    }

    property Timer callTm: Timer {
        interval: 1000
        repeat: true
        running: u.inCall
        triggeredOnStart: true
        onTriggered: {
            const s = Math.floor((Date.now() - u.callT0) / 1000)
            u.callEl = (s >= 3600 ? Math.floor(s / 3600) + ":" : "") + String(Math.floor(s / 60) % 60).padStart(2, "0") + ":" + String(s % 60).padStart(2, "0")
        }
    }

    property Timer toastTm: Timer {
        interval: 1000
        repeat: true
        running: u.toasts.length > 0
        onTriggered: {
            const n = Date.now()
            const k = u.toasts.filter(t => t.keep || n - t.at < 8000)
            if (k.length !== u.toasts.length) u.toasts = k
        }
    }

    function reg(p) {
        hideTip()
        if (pop && pop !== p) pop.close()
        pop = p
    }
    function unreg(p) { if (pop === p) pop = null }
    function closePop() { if (pop) pop.close() }

    function showTip(item, text, side) {
        if (!tipOk || !item || !text) return
        tipSide = side || "b"
        const p = item.mapToItem(null, 0, 0)
        tipBox = { x: p.x, y: p.y, w: item.width, h: item.height }
        tipItem = item
        tipText = text
    }
    function hideTip(item) {
        if (item && tipItem !== item) return
        tipText = ""
        tipItem = null
    }

    function ask(o) { askReq(o) }
    function rmDemo() { S.cfgAccs.filter(a => a.base === "demo").forEach(a => S.c.rm_acc(a.id)) }
    function exitDemo() {
        ask({ title: "Exit demo", body: "Demo accounts and their data are removed.", ok: "Exit demo", danger: true, run: rmDemo })
    }
    function guardDemo(fn) {
        if (!demo) { fn(); return }
        ask({ title: "Exit demo first", body: "Demo accounts are removed before a real account is added.", ok: "Exit demo and continue", danger: true, run: () => { rmDemo(); fn() } })
    }
    function pushEmo(e) {
        const k = e.id ? "c" + e.id.raw : "u" + e.name
        const l = [e].concat((S.ui.emor || []).filter(x => (x.id ? "c" + x.id.raw : "u" + x.name) !== k)).slice(0, 12)
        S.setUi("emor", l)
    }

    function err(d) {
        const a = d.acc ? S.cfgAccs.find(x => x.id === d.acc) : null
        const me = d.acc && S.accs[d.acc] ? S.accs[d.acc].me : null
        const pre = a ? Th.svcName(S.tag(a.svc)) + (me ? " " + (me.disp || me.name) : "") + ": " : ""
        const msg = pre + d.msg
        const k = toasts.filter(t => t.msg !== msg)
        k.push({ msg: msg, at: Date.now(), keep: d.ctx === "send", id: Date.now() + Math.random(), retry: d.retry || null })
        toasts = k.slice(-5)
    }
    function dismiss(id) { toasts = toasts.filter(t => t.id !== id) }

    function cnt(b, n) {
        n = n || S.lay
        if (n.b) return n.b === b ? 1 : 0
        let c = 0
        for (const x of n.ch || []) c += cnt(b, x)
        return c
    }

    function onEdit() {
        if (S.edit) {
            snap = L.cp(S.lay)
            undo = []
        }
        hideTip()
    }
    function push() {
        const k = undo.slice(-30)
        k.push(L.cp(S.lay))
        undo = k
    }
    function setLay(l) {
        const before = L.cp(S.lay)
        const bj = JSON.stringify(before)
        const ret = S.setLay(l)
        const ch = JSON.stringify(S.lay) !== bj
        const ok = ret === undefined ? ch || JSON.stringify(l) === bj : ret !== false
        if (ch) {
            const k = undo.slice(-30)
            k.push(before)
            undo = k
        }
        return ok
    }
    function undoLay() {
        if (!undo.length) return
        const k = undo.slice()
        const l = k.pop()
        undo = k
        S.setLay(l)
    }
    function resetLay() { setLay(L.cp(L.presets.classic)) }
    function cancelEdit() {
        if (snap) S.setLay(snap)
        S.edit = false
    }
    function addBlk(b, side) {
        const c = L.find(S.lay, "chat")
        const nd = b === "chat" ? { b: "chat", grow: true, o: { slot: 1 } } : { b: b, w: 248 }
        setLay(L.ins(S.lay, c || [], side, nd))
    }
    function addItems() {
        const out = []
        for (const b of S.allBlks()) {
            const id = b.id
            if (id === "chat") {
                if (JSON.stringify(S.lay).indexOf('"slot":1') >= 0) continue
                out.push({ text: "Second chat", items: [{ text: "Left of chat", run: () => addBlk("chat", "l") }, { text: "Right of chat", run: () => addBlk("chat", "r") }] })
                continue
            }
            if (cnt(id) > 0) continue
            out.push({ text: b.name, items: [{ text: "Left of chat", run: () => addBlk(id, "l") }, { text: "Right of chat", run: () => addBlk(id, "r") }] })
        }
        return out
    }
}
