import QtQuick
import Cord

Item {
    id: r
    property Item tgt: parent
    property string text: ""
    property bool hov: false
    property string side: "b"
    property Item edge: null
    width: 0
    height: 0
    Timer {
        id: t
        interval: 400
        onTriggered: if (r.hov) U.showTip(r.tgt, r.text, r.side, r.edge)
    }
    onHovChanged: {
        if (hov) t.restart()
        else cancel()
    }
    function cancel() {
        t.stop()
        U.hideTip(r.tgt)
    }
    Component.onDestruction: U.hideTip(r.tgt)
}
