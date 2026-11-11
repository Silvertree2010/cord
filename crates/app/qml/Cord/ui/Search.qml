import QtQuick
import QtQuick.Controls
import Cord

Popup {
    id: r
    enter: PopIn {}
    exit: PopOut {}
    transformOrigin: Popup.TopRight
    width: 460
    height: Math.min(parent ? parent.height - 120 : 600, 640)
    padding: 0
    closePolicy: Popup.CloseOnEscape | Popup.CloseOnPressOutside
    background: Pane {}
    property string q: ""
    property string cid: ""
    property var res: []
    property int total: 0
    property bool busy: false
    property string err: ""
    property int seq: 0
    readonly property var ch: S.chans[r.cid] || null
    function run(q, cid) {
        r.q = q
        r.cid = cid
        r.res = []
        r.total = 0
        r.err = ""
        if (!q.trim() || !r.ch || S.svc(cid) !== "dc") { r.err = S.svc(cid) !== "dc" ? "Search works on Discord only." : ""; open(); return }
        const n = ++r.seq
        r.busy = true
        open()
        const a = r.ch.grid ? { gld: S.raw(r.ch.grid), q: q } : { chan: S.raw(cid), q: q }
        S.api(S.acc(cid), "search", a, (ok, v, e) => {
            if (n !== r.seq) return
            r.busy = false
            if (!ok) { r.err = e; return }
            r.total = v.total || 0
            r.res = v.msgs || []
        })
    }
    onAboutToShow: U.reg(r)
    onClosed: U.unreg(r)
    Column {
        anchors.fill: parent
        Item {
            width: parent.width
            height: 48
            Lbl {
                x: 16
                anchors.verticalCenter: parent.verticalCenter
                width: parent.width - 60
                text: r.busy ? "Searching…" : r.total === 1 ? "1 Result" : r.total + " Results"
                font.weight: Font.DemiBold
            }
            IBtn { anchors.right: parent.right; anchors.rightMargin: 8; anchors.verticalCenter: parent.verticalCenter; n: "x"; sz: 28; isz: 14; onClicked: r.close() }
            Rectangle { anchors.bottom: parent.bottom; width: parent.width; height: 1; color: Th.ln }
        }
        ListView {
            id: lv
            width: parent.width
            height: parent.height - 48
            clip: true
            model: r.res
            spacing: 6
            topMargin: 8
            bottomMargin: 8
            ScrollBar.vertical: Scr {}
            Smooth {}
            delegate: Column {
                width: lv.width
                spacing: 2
                readonly property string mc: modelData.chan ? S.ridOf(modelData.chan) : ""
                Lbl {
                    x: 16
                    visible: !!r.ch && !!r.ch.grid
                    text: "#" + S.chName(parent.mc)
                    color: Th.tx2
                    font.pixelSize: Th.fsXS
                    font.weight: Font.DemiBold
                }
                Rectangle {
                    x: 8
                    width: lv.width - 16
                    height: mm.height + 8
                    radius: Th.rs
                    color: hm.containsMouse ? Th.bg4 : Th.bg3
                    border.color: Th.ln
                    Msg {
                        id: mm
                        y: 4
                        width: parent.width
                        m: modelData
                        cid: parent.parent.mc
                    }
                    MouseArea {
                        id: hm
                        anchors.fill: parent
                        hoverEnabled: true
                        acceptedButtons: Qt.LeftButton
                        cursorShape: Qt.PointingHandCursor
                        onClicked: { const c = parent.parent.mc; if (c) { S.open(c) } r.close() }
                    }
                }
            }
            BusyIndicator { anchors.centerIn: parent; running: r.busy; visible: running; width: 32; height: 32 }
            Lbl { anchors.centerIn: parent; width: parent.width - 40; visible: r.err !== ""; text: r.err; color: Th.err; wrapMode: Text.Wrap; elide: Text.ElideNone; horizontalAlignment: Text.AlignHCenter }
            Lbl { anchors.centerIn: parent; visible: !r.busy && r.err === "" && r.res.length === 0 && r.q !== ""; text: "No results for \"" + r.q + "\""; color: Th.tx2 }
        }
    }
}
