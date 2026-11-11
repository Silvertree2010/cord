import QtQuick
import QtQuick.Controls
import Cord

Rectangle {
    id: r
    color: "#80000000"
    NumberAnimation on opacity { from: 0; to: 1; duration: Th.mF + 20; easing.type: Easing.OutCubic }
    MouseArea { anchors.fill: parent; onClicked: S.qk = false }
    readonly property var secs: ({ r: "Recent and unread", c: "Channels", d: "Direct messages", g: "Servers" })
    readonly property string q: fl.text.toLowerCase().replace(/^[#@*]/, "")
    property var ent: {
        void S.ver
        void S.uver
        const q = r.q
        const md = fl.text.charAt(0)
        const out = []
        const seen = {}
        const sc = s => { if (!q) return 1; const i = s.toLowerCase().indexOf(q); return i < 0 ? 0 : i === 0 ? 3 : 2 }
        const add = o => {
            const k = o.t + "|" + o.n.toLowerCase() + "|" + o.sub
            if (seen[k] || seen[o.id]) return
            seen[k] = true
            seen[o.id] = true
            out.push(o)
        }
        const isDm = c => c.k === "Dm" || (c.k === "Grp" && !c.grid)
        const chE = (c, t, s) => ({ t: t, id: c.rid, n: S.chName(c.rid), sub: isDm(c) ? Th.svcName(S.svc(c.rid)) : c.grid && S.gmap[c.grid] ? S.gmap[c.grid].name : "", g: c.k === "Voice" || c.k === "Stage" ? "voice" : isDm(c) ? "at" : "hash", s: s })
        const cap = { r: 6, c: 8, d: 6, g: 6 }
        const cnt = { r: 0, c: 0, d: 0, g: 0 }
        const addC = o => { if (cnt[o.t] >= cap[o.t]) return; const n0 = out.length; add(o); if (out.length > n0) cnt[o.t]++ }
        if (!q && !md) {
            const un = Object.values(S.chans).filter(c => c.k !== "Cat" && c.k !== "Thread" && S.unr[c.rid] && (S.unr[c.rid].n || S.unr[c.rid].ment) && S.chName(c.rid))
                .sort((a, b) => (S.unr[b.rid].ment || 0) - (S.unr[a.rid].ment || 0))
            for (const c of un) addC(chE(c, "r", 1))
            for (const id of S.ui.rec || []) { const c = S.chans[id]; if (c && c.k !== "Cat" && S.chName(id)) addC(chE(c, "r", 1)) }
            for (const c of S.dms()) addC(chE(c, "d", 1))
            for (const g of S.glds) addC({ t: "g", id: g.rid, n: g.name, sub: Th.svcName(S.svc(g.rid)), g: "grid", s: 1 })
        } else {
            if (md !== "*") for (const c of Object.values(S.chans)) {
                if (c.k === "Cat" || c.k === "Thread") continue
                const dm = isDm(c)
                if (md === "#" && dm) continue
                if (md === "@" && !dm) continue
                const n = S.chName(c.rid)
                if (!n) continue
                const s = sc(n)
                if (s) add(chE(c, dm ? "d" : "c", s + (S.unr[c.rid] ? 0.5 : 0)))
            }
            if (md === "@") {
                const have = {}
                for (const c of S.dms()) if (c.k === "Dm" && c.rcp && c.rcp[0] && c.rcp[0].id) have[S.ridOf(c.rcp[0].id)] = true
                for (const x of S.rels) {
                    if (x.k !== "Friend" || !x.usr.id) continue
                    const id = S.ridOf(x.usr.id)
                    if (have[id]) continue
                    const n = x.usr.disp || x.usr.name
                    const s = sc(n)
                    if (s) add({ t: "d", id: "u:" + id, n: n, sub: Th.svcName(S.svc(id)) + " friend", g: "at", s: s, usr: x.usr })
                }
            }
            if (md !== "#" && md !== "@") for (const g of S.glds) {
                const s = sc(g.name)
                if (s) add({ t: "g", id: g.rid, n: g.name, sub: Th.svcName(S.svc(g.rid)), g: "grid", s: s })
            }
        }
        const ord = { r: 0, c: 1, d: 2, g: 3 }
        const res = out.sort((a, b) => ord[a.t] - ord[b.t] || b.s - a.s)
        if (!q && !md) return res
        const k = { c: 0, d: 0, g: 0 }
        return res.filter(e => ++k[e.t] <= cap[e.t]).slice(0, 14)
    }
    function esc(t) { return String(t).replace(/&/g, "&amp;").replace(/</g, "&lt;").replace(/>/g, "&gt;") }
    function hl(t) {
        const q = r.q
        const i = q ? t.toLowerCase().indexOf(q) : -1
        if (i < 0) return esc(t)
        return esc(t.slice(0, i)) + '<span style="font-weight:600;color:' + Th.tx + '">' + esc(t.slice(i, i + q.length)) + "</span>" + esc(t.slice(i + q.length))
    }
    property var rows: {
        const o = []
        let last = ""
        let n = 0
        for (const e of ent) {
            if (e.t !== last) { o.push({ h: secs[e.t] }); last = e.t }
            o.push({ e: e, i: n++ })
        }
        return o
    }
    property int cur: 0
    onEntChanged: cur = 0
    Rectangle {
        anchors.horizontalCenter: parent.horizontalCenter
        y: parent.height * 0.15
        transformOrigin: Item.Top
        NumberAnimation on scale { from: 0.95; to: 1; duration: Th.mM; easing.type: Easing.OutCubic }
        width: Math.min(parent.width - 48, 560)
        height: col.implicitHeight + 24
        radius: 6
        color: Th.bg2
        border.color: Th.ln
        MouseArea { anchors.fill: parent }
        Column {
            id: col
            x: 12
            y: 12
            width: parent.width - 24
            spacing: 8
            Fld {
                id: fl
                width: parent.width
                placeholderText: "Jump to…"
                focus: true
                Component.onCompleted: forceActiveFocus()
                Keys.onDownPressed: { r.cur = Math.min(r.cur + 1, r.ent.length - 1); r.vis() }
                Keys.onUpPressed: { r.cur = Math.max(r.cur - 1, 0); r.vis() }
                Keys.onReturnPressed: r.go(r.ent[r.cur])
                Keys.onEnterPressed: r.go(r.ent[r.cur])
                Keys.onEscapePressed: S.qk = false
            }
            ListView {
                Smooth {}
                id: lv
                width: parent.width
                height: Math.min(contentHeight, r.height * 0.55)
                visible: r.ent.length > 0
                clip: true
                interactive: contentHeight > height
                model: r.rows
                ScrollBar.vertical: Scr {}
                delegate: Item {
                    width: lv.width
                    height: modelData.h ? 28 : 36
                    Lbl {
                        visible: !!modelData.h
                        x: 10
                        y: 8
                        text: modelData.h || ""
                        color: Th.tx3
                        font.pixelSize: Th.fsXS
                        font.weight: Font.DemiBold
                    }
                    Rectangle {
                        visible: !modelData.h
                        anchors.fill: parent
                        radius: Th.rs
                        color: modelData.i === r.cur ? Th.sel : "transparent"
                        Ico { id: gi; x: 10; n: modelData.e ? modelData.e.g : "hash"; sz: 16; col: Th.tx3; anchors.verticalCenter: parent.verticalCenter }
                        Lbl {
                            id: nl
                            anchors.left: gi.right
                            anchors.leftMargin: 10
                            anchors.right: sb.left
                            anchors.rightMargin: 12
                            anchors.verticalCenter: parent.verticalCenter
                            textFormat: Text.RichText
                            text: modelData.e ? r.hl(modelData.e.n) : ""
                            color: r.q ? Th.tx2 : Th.tx
                            font.weight: Font.Medium
                        }
                        Lbl {
                            id: sb
                            anchors.right: parent.right
                            anchors.rightMargin: 10
                            anchors.verticalCenter: parent.verticalCenter
                            width: Math.min(implicitWidth, 160)
                            horizontalAlignment: Text.AlignRight
                            text: modelData.e ? modelData.e.sub : ""
                            color: Th.tx2
                            font.pixelSize: Th.fsXS
                        }
                        MouseArea { anchors.fill: parent; hoverEnabled: true; onEntered: r.cur = modelData.i; onClicked: r.go(modelData.e) }
                    }
                }
            }
            Lbl {
                visible: r.ent.length === 0
                width: parent.width
                height: 36
                verticalAlignment: Text.AlignVCenter
                leftPadding: 10
                text: "No matches"
                color: Th.tx3
            }
        }
    }
    function vis() {
        const i = rows.findIndex(x => x.i === cur)
        if (i >= 0) lv.positionViewAtIndex(i, ListView.Contain)
    }
    function go(x) {
        if (!x) return
        if (x.usr) S.openDm(x.usr)
        else if (x.t === "g") S.pick(x.id)
        else {
            U.recent(x.id)
            const c = S.chans[x.id]
            if (!c) return
            S.sel = c.grid || "home"
            S.open(x.id)
        }
        S.qk = false
    }
}
