import QtQuick
import Cord

Rectangle {
    id: r
    property var o: ({})
    property var cs: S.call || ({})
    property var ch: cs.chan ? S.chans[S.ridOf(cs.chan)] : null
    property real t0: 0
    property string el: ""
    property bool shown: !!cs.chan
    visible: shown
    implicitWidth: 248
    implicitHeight: shown ? 92 : 0
    color: Th.bg3
    onVisibleChanged: if (visible) t0 = Date.now()
    Timer {
        interval: 1000
        repeat: true
        running: r.visible && S.f("callTimer")
        onTriggered: {
            const s = Math.floor((Date.now() - r.t0) / 1000)
            r.el = (s >= 3600 ? Math.floor(s / 3600) + ":" : "") + String(Math.floor(s / 60) % 60).padStart(2, "0") + ":" + String(s % 60).padStart(2, "0")
        }
    }
    Column {
        x: 12
        y: 10
        width: parent.width - 60
        spacing: 2
        Row {
            spacing: 6
            Ico { n: "voice"; sz: 16; col: r.cs.up ? Th.ok : Th.warn }
            Lbl {
                text: r.cs.err ? "Call failed" : r.cs.up ? "Voice connected" : "Connecting"
                color: r.cs.err ? Th.err : r.cs.up ? Th.ok : Th.warn
                font.weight: Font.DemiBold
                font.pixelSize: Th.fsS
            }
            Ico { visible: !!r.cs.e2ee; n: "lock"; sz: 13; col: Th.ok }
        }
        Lbl {
            width: parent.width
            text: (r.ch ? S.chName(r.ch.rid) : "") + (r.ch && r.ch.grid && S.gmap[r.ch.grid] ? " / " + S.gmap[r.ch.grid].name : "")
            color: Th.tx2
            font.pixelSize: 11
        }
        Lbl {
            text: [r.cs.ping ? r.cs.ping + " ms" : "", S.f("callTimer") ? r.el : ""].filter(x => x).join("  ·  ")
            color: Th.tx3
            font.pixelSize: 11
        }
    }
    IBtn {
        anchors.right: parent.right
        anchors.rightMargin: 8
        y: 10
        n: "hang"
        red: true
        tip: "Disconnect"
        onClicked: S.c.leave()
    }
    Row {
        x: 8
        anchors.bottom: parent.bottom
        anchors.bottomMargin: 6
        spacing: 4
        Btn {
            text: r.cs.live ? "Stop sharing" : "Share screen"
            ico: "screen"
            flat: !r.cs.live
            pri: !!r.cs.live
            height: 28
            onClicked: r.cs.live ? S.c.golive(false, "{}") : S.goLive()
        }
        Btn { text: "Open"; flat: true; height: 28; onClicked: if (r.ch) S.open(r.ch.rid) }
    }
}
