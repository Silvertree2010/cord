import QtQuick
import Cord

Item {
    id: r
    property var n: ({})
    readonly property real gap: 2
    function boxes(n, x, y, w, h) {
        if (n.b) return [{ b: n.b, x: x, y: y, w: w, h: h }]
        const ch = n.ch || []
        const out = []
        if (n.t === "row") {
            const fixed = ch.filter(c => !c.grow).reduce((a, c) => a + (c.w || 200), 0)
            const g = ch.filter(c => c.grow).length || 1
            const k = (w - gap * (ch.length - 1)) / (fixed * 1.6 + g * 600)
            let cx = x
            for (const c of ch) {
                const cw = c.grow ? 600 * k : (c.w || 200) * 1.6 * k
                out.push(...boxes(c, cx, y, cw, h))
                cx += cw + gap
            }
        } else {
            const fx = ch.filter(c => !c.grow).length
            const gh = Math.max(10, (h - fx * 14 - gap * (ch.length - 1)) / (ch.length - fx || 1))
            let cy = y
            for (const c of ch) {
                const chh = c.grow ? gh : 12
                out.push(...boxes(c, x, cy, w, chh))
                cy += chh + gap
            }
        }
        return out
    }
    Repeater {
        model: r.width > 0 ? r.boxes(r.n, 0, 0, r.width, r.height) : []
        Rectangle {
            x: modelData.x
            y: modelData.y
            width: modelData.w
            height: modelData.h
            radius: 3
            color: modelData.b === "chat" || modelData.b === "stage" ? Th.bg3 : modelData.b === "rail" ? Th.bg4 : Th.bg2
            border.color: Th.ln
            Rectangle {
                visible: modelData.b === "chat"
                anchors.bottom: parent.bottom
                anchors.margins: 4
                x: 4
                width: parent.width - 8
                height: 5
                radius: 2
                color: Th.bg4
            }
            Column {
                visible: modelData.b === "side" || modelData.b === "mems" || modelData.b === "inbox"
                x: 4
                y: 6
                spacing: 4
                Repeater {
                    model: Math.max(0, Math.min(4, Math.floor((modelData.h - 6) / 7)))
                    Rectangle { width: Math.max(4, modelData.w - 10 - (index % 2) * 6); height: 3; radius: 1.5; color: Th.tx3; opacity: 0.4 }
                }
            }
            Column {
                visible: modelData.b === "rail" && modelData.h > modelData.w
                anchors.horizontalCenter: parent.horizontalCenter
                y: 4
                spacing: 3
                Repeater {
                    model: 4
                    Rectangle { width: Math.max(2, modelData.w - 6); height: width; radius: width / 2; color: Th.tx3; opacity: 0.4 }
                }
            }
        }
    }
}
