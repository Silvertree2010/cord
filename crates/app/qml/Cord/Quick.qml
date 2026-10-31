import QtQuick
import QtQuick.Controls
import Cord

Rectangle {
    id: r
    color: "#88000000"
    MouseArea { anchors.fill: parent; onClicked: S.qk = false }
    readonly property var secs: ({ c: "Channels", d: "Direct messages", g: "Servers" })
    property var ent: {
        const q = fl.text.toLowerCase().replace(/^[#@*]/, "")
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
        if (md !== "*") for (const c of Object.values(S.chans)) {
            if (c.k === "Cat" || c.k === "Thread") continue
            const dm = c.k === "Dm" || (c.k === "Grp" && !c.grid)
            if (md === "#" && dm) continue
            if (md === "@" && !dm) continue
            const n = S.chName(c.rid)
            if (!n) continue
            const s = sc(n)
            if (s) add({ t: dm ? "d" : "c", id: c.rid, n: n, sub: dm ? Th.svcName(S.svc(c.rid)) : c.grid && S.gmap[c.grid] ? S.gmap[c.grid].name : "", g: c.k === "Voice" || c.k === "Stage" ? "voice" : dm ? "at" : "hash", s: s + (S.unr[c.rid] ? 0.5 : 0) })
        }
        if (md !== "#" && md !== "@") for (const g of S.glds) {
            const s = sc(g.name)
            if (s) add({ t: "g", id: g.rid, n: g.name, sub: Th.svcName(S.svc(g.rid)), g: "grid", s: s })
        }
        const ord = { c: 0, d: 1, g: 2 }
        return out.sort((a, b) => ord[a.t] - ord[b.t] || b.s - a.s).slice(0, 14)
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
        width: Math.min(parent.width - 48, 560)
        height: col.implicitHeight + 24
        radius: Th.r
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
                        font.pixelSize: 12
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
                            text: modelData.e ? modelData.e.n : ""
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
                            color: Th.tx3
                            font.pixelSize: Th.fsS
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
        if (x.t === "g") S.pick(x.id)
        else {
            const c = S.chans[x.id]
            if (!c) return
            S.sel = c.grid || "home"
            S.open(x.id)
        }
        S.qk = false
    }
}
