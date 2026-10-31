import QtQuick
import QtQuick.Controls
import Cord
import "emo.js" as E

Popup {
    id: r
    property var cb: null
    width: 380
    height: 420
    padding: 12
    closePolicy: Popup.CloseOnEscape | Popup.CloseOnPressOutside
    onAboutToShow: { U.reg(r); ef.text = "" }
    onOpened: ef.forceActiveFocus()
    onClosed: U.unreg(r)
    background: Rectangle { color: Th.bg3; radius: Th.r; border.color: Th.ln; border.width: 1 }
    property string q: ""
    property var cust: {
        const out = []
        for (const g of S.glds) for (const e of g.emos || []) out.push({ e: e, g: g.rid })
        return out
    }
    property var items: {
        const q = r.q.trim()
        const cu = r.cust.filter(x => !q || x.e.name.toLowerCase().indexOf(q) >= 0).map(x => ({ c: x.e, g: x.g }))
        const un = E.all.filter(x => !q || x[1].indexOf(q) >= 0).map(x => ({ u: x[0] }))
        return cu.concat(un)
    }
    property var recent: S.ui.emor || []
    function pick(e, keep) {
        if (r.cb) r.cb(e)
        U.pushEmo(e)
        if (!keep) r.close()
    }
    function pickItem(x, keep) {
        pick(x.c ? x.c : { id: null, name: x.u, url: null, anim: false }, keep)
    }
    Column {
        anchors.fill: parent
        spacing: 8
        Fld {
            id: ef
            width: parent.width
            placeholderText: "Search emoji"
            onTextChanged: r.q = text.toLowerCase()
            Keys.onReturnPressed: e => { if (r.items.length) r.pickItem(r.items[0], !!(e.modifiers & Qt.ShiftModifier)) }
        }
        Column {
            visible: !r.q && r.recent.length > 0
            width: parent.width
            spacing: 4
            Lbl { text: "Recent"; color: Th.tx3; font.pixelSize: 12; font.weight: Font.DemiBold }
            Row {
                spacing: 2
                Repeater {
                    model: r.recent.slice(0, 9)
                    delegate: Rectangle {
                        width: 38
                        height: 38
                        radius: Th.rs
                        color: rm.containsMouse ? Th.hov : "transparent"
                        Image { visible: !!modelData.url; anchors.centerIn: parent; width: 28; height: 28; source: modelData.url || ""; sourceSize: Qt.size(56, 56); asynchronous: true }
                        Text { visible: !modelData.url; anchors.centerIn: parent; text: modelData.url ? "" : modelData.name; font.pixelSize: 24 }
                        MouseArea { id: rm; anchors.fill: parent; hoverEnabled: true; cursorShape: Qt.PointingHandCursor; onClicked: m => r.pick(modelData, !!(m.modifiers & Qt.ShiftModifier)) }
                    }
                }
            }
        }
        GridView {
            width: parent.width
            height: parent.height - y
            clip: true
            cellWidth: 40
            cellHeight: 40
            topMargin: 8
            bottomMargin: 8
            leftMargin: 0
            model: r.items
            ScrollBar.vertical: Scr {}
            delegate: Rectangle {
                width: 38
                height: 38
                radius: Th.rs
                color: em.containsMouse ? Th.hov : "transparent"
                Image { visible: !!modelData.c; anchors.centerIn: parent; width: 28; height: 28; source: modelData.c ? (modelData.c.url || "") : ""; sourceSize: Qt.size(56, 56); asynchronous: true }
                Text { visible: !!modelData.u; anchors.centerIn: parent; text: modelData.u || ""; font.pixelSize: 24 }
                MouseArea {
                    id: em
                    anchors.fill: parent
                    hoverEnabled: true
                    cursorShape: Qt.PointingHandCursor
                    onClicked: m => r.pickItem(modelData, !!(m.modifiers & Qt.ShiftModifier))
                }
            }
            Lbl { visible: r.items.length === 0; anchors.centerIn: parent; text: "No matches"; color: Th.tx3 }
        }
    }
}
