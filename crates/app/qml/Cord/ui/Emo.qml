import QtQuick
import QtQuick.Controls
import Cord
import "emo.js" as E

Popup {
    id: r
    enter: PopIn {}
    exit: PopOut {}
    transformOrigin: Popup.Bottom
    property var cb: null
    width: 420
    height: 460
    padding: 12
    closePolicy: Popup.CloseOnEscape | Popup.CloseOnPressOutside
    onAboutToShow: { U.reg(r); ef.text = "" }
    onOpened: ef.forceActiveFocus()
    onClosed: U.unreg(r)
    background: Pane {}
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
    property var secs: {
        if (r.q.trim()) return [{ t: S.t("Results"), ic: "", it: r.items }]
        const o = []
        for (const g of S.glds) if ((g.emos || []).length) o.push({ t: g.name, ic: g.icon || "", it: g.emos.map(e => ({ c: e, g: g.rid })) })
        o.push({ t: "Emoji", ic: "", it: E.all.map(x => ({ u: x[0] })) })
        return o
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
    property alias body: col
    Column {
        id: col
        anchors.fill: parent
        spacing: 8
        Fld {
            id: ef
            width: parent.width
            placeholderText: S.t("Search emoji")
            onTextChanged: r.q = text.toLowerCase()
            Keys.onReturnPressed: e => { if (r.items.length) r.pickItem(r.items[0], !!(e.modifiers & Qt.ShiftModifier)) }
        }
        Column {
            visible: !r.q && r.recent.length > 0
            width: parent.width
            spacing: 4
            Lbl { text: S.t("Recent"); color: Th.tx3; font.pixelSize: Th.fsXS; font.weight: Font.DemiBold }
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
        Row {
            width: parent.width
            height: parent.height - y
            spacing: 6
            ListView {
                id: nav
                visible: !r.q && r.secs.length > 1
                width: visible ? 36 : 0
                height: parent.height
                clip: true
                spacing: 4
                model: r.secs
                delegate: Rectangle {
                    width: 36
                    height: 36
                    radius: Th.rs
                    color: nm_.containsMouse ? Th.hov : "transparent"
                    Av { visible: !!modelData.ic; anchors.centerIn: parent; sz: 28; src: modelData.ic; name: modelData.t; sq: true }
                    Text { visible: !modelData.ic; anchors.centerIn: parent; text: modelData.t === "Emoji" ? "😀" : modelData.t.charAt(0); color: Th.tx; font.pixelSize: modelData.t === "Emoji" ? 20 : 14; font.family: Th.font }
                    MouseArea { id: nm_; anchors.fill: parent; hoverEnabled: true; cursorShape: Qt.PointingHandCursor; onClicked: sl.positionViewAtIndex(index, ListView.Beginning) }
                    Tip { text: modelData.t; hov: nm_.containsMouse; side: "r" }
                }
            }
            ListView {
                id: sl
                width: parent.width - nav.width - (nav.visible ? 6 : 0)
                height: parent.height
                clip: true
                model: r.secs
                cacheBuffer: 400
                ScrollBar.vertical: Scr {}
                readonly property int cols: Math.max(1, Math.floor((width - 8) / 40))
                readonly property real cw: (width - 8) / cols
                delegate: Column {
                    width: sl.width
                    Lbl { text: modelData.t; color: Th.tx3; font.pixelSize: Th.fsXS; font.weight: Font.DemiBold; width: parent.width; topPadding: 6; bottomPadding: 4; elide: Text.ElideRight }
                    Grid {
                        columns: sl.cols
                        Repeater {
                            model: modelData.it
                            delegate: Rectangle {
                                width: sl.cw
                                height: 40
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
                        }
                    }
                }
                Lbl { visible: r.items.length === 0; anchors.centerIn: parent; text: S.t("No matches"); color: Th.tx3 }
            }
        }
    }
}
