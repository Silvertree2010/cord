import QtQuick
import Cord

Item {
    id: r
    property bool on: false
    signal toggled(bool v)
    implicitWidth: 40
    implicitHeight: 24
    Rectangle {
        anchors.fill: parent
        radius: height / 2
        color: r.on ? Th.ac : Th.bg4
        Behavior on color { ColorAnimation { duration: S.f("anim") ? 120 : 0 } }
        Rectangle {
            width: 18
            height: 18
            radius: 9
            y: 3
            x: r.on ? parent.width - width - 3 : 3
            color: r.on ? Th.acTx : Th.tx2
            Behavior on x { NumberAnimation { duration: S.f("anim") ? 120 : 0; easing.type: Easing.OutCubic } }
        }
    }
    MouseArea {
        anchors.fill: parent
        cursorShape: Qt.PointingHandCursor
        onClicked: r.toggled(!r.on)
    }
}
