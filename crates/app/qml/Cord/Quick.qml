import QtQuick
import QtQuick.Controls
import Cord

Rectangle {
    id: r
    color: "#00000088"
    MouseArea { anchors.fill: parent; onClicked: S.qk = false }
    property var res: {
        const q = fl.text.toLowerCase().replace(/^[#@*]/, "")
        const md = fl.text.charAt(0)
        const out = []
        const sc = s => { if (!q) return 1; const i = s.toLowerCase().indexOf(q); return i < 0 ? 0 : i === 0 ? 3 : 2 }
        if (md !== "*") for (const c of Object.values(S.chans)) {
            if (c.k === "Cat") continue
            const dm = c.k === "Dm" || (c.k === "Grp" && !c.grid)
            if (md === "#" && dm) continue
            if (md === "@" && !dm) continue
            const n = S.chName(c.rid)
            const s = sc(n)
            if (s) out.push({ k: "c", id: c.rid, n: n, sub: c.grid && S.gmap[c.grid] ? S.gmap[c.grid].name : "Direct message", s: s + (S.unr[c.rid] ? 0.5 : 0), dm: dm })
        }
        if (md !== "#" && md !== "@") for (const g of S.glds) {
            const s = sc(g.name)
            if (s) out.push({ k: "g", id: g.rid, n: g.name, sub: Th.svcName(S.svc(g.rid)), s: s })
        }
        return out.sort((a, b) => b.s - a.s).slice(0, 12)
    }
    property int cur: 0
    onResChanged: cur = 0
    Rectangle {
        anchors.horizontalCenter: parent.horizontalCenter
        y: parent.height * 0.18
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
            spacing: 6
            Fld {
                id: fl
                width: parent.width
                placeholderText: "Where would you like to go?  # channels  @ people  * servers"
                focus: true
                Component.onCompleted: forceActiveFocus()
                Keys.onDownPressed: r.cur = Math.min(r.cur + 1, r.res.length - 1)
                Keys.onUpPressed: r.cur = Math.max(r.cur - 1, 0)
                Keys.onReturnPressed: r.go(r.res[r.cur])
                Keys.onEscapePressed: S.qk = false
            }
            Repeater {
                model: r.res
                Rectangle {
                    width: col.width
                    height: 40
                    radius: Th.rs
                    color: index === r.cur ? Th.sel : "transparent"
                    Row {
                        x: 10
                        spacing: 10
                        anchors.verticalCenter: parent.verticalCenter
                        Badge { sv: S.svc(modelData.id); sz: 14; ring: "transparent"; anchors.verticalCenter: parent.verticalCenter }
                        Ico { n: modelData.k === "g" ? "grid" : modelData.dm ? "at" : "hash"; sz: 16; col: Th.tx3; anchors.verticalCenter: parent.verticalCenter }
                        Lbl { text: modelData.n; font.weight: Font.Medium; anchors.verticalCenter: parent.verticalCenter }
                        Lbl { text: modelData.sub; color: Th.tx3; font.pixelSize: Th.fsS; anchors.verticalCenter: parent.verticalCenter }
                    }
                    MouseArea { anchors.fill: parent; hoverEnabled: true; onEntered: r.cur = index; onClicked: r.go(modelData) }
                }
            }
        }
    }
    function go(x) {
        if (!x) return
        if (x.k === "g") S.pick(x.id)
        else {
            const c = S.chans[x.id]
            if (c.k === "Voice" || c.k === "Stage") S.c.join(x.id)
            S.sel = c.grid || "home"
            S.open(x.id)
        }
        S.qk = false
    }
}
