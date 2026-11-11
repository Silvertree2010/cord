import QtQuick
import Cord

Rectangle {
    id: r
    property var tag: null
    property real fs: 11
    visible: !!r.tag && !!r.tag.t
    width: visible ? rw.implicitWidth + 8 : 0
    height: r.fs + 7
    radius: 4
    color: Th.a(Th.tx, 0.08)
    Row {
        id: rw
        anchors.centerIn: parent
        spacing: 3
        Image {
            visible: !!(r.tag && r.tag.badge)
            source: r.tag && r.tag.badge ? r.tag.badge : ""
            width: r.fs + 1
            height: r.fs + 1
            sourceSize: Qt.size(32, 32)
            asynchronous: true
            anchors.verticalCenter: parent.verticalCenter
        }
        Lbl {
            text: r.tag ? r.tag.t : ""
            font.pixelSize: r.fs
            font.weight: Font.Bold
            color: Th.tx
            anchors.verticalCenter: parent.verticalCenter
        }
    }
}
