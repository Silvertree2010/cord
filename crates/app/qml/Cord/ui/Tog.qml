import QtQuick
import Cord

Item {
    id: r
    property bool on: false
    property string label: ""
    signal toggled(bool v)
    implicitWidth: 32
    implicitHeight: 18
    activeFocusOnTab: true
    Accessible.role: Accessible.CheckBox
    Accessible.name: label
    Accessible.checked: on
    Keys.onReturnPressed: r.toggled(!r.on)
    Keys.onSpacePressed: r.toggled(!r.on)
    Rectangle {
        anchors.fill: parent
        radius: height / 2
        color: r.on ? Th.ac : Th.bg4
        Behavior on color { ColorAnimation { duration: S.f("anim") ? 120 : 0 } }
        Rectangle {
            width: 12
            height: 12
            radius: 6
            y: 3
            x: r.on ? parent.width - width - 3 : 3
            color: r.on ? Th.acTx : Th.tx2
            Behavior on x { NumberAnimation { duration: S.f("anim") ? 120 : 0; easing.type: Easing.OutCubic } }
        }
    }
    Rectangle {
        anchors.fill: parent
        anchors.margins: -2
        radius: height / 2
        color: "transparent"
        border.color: Th.ac
        border.width: 2
        visible: r.activeFocus
    }
    MouseArea {
        anchors.fill: parent
        cursorShape: Qt.PointingHandCursor
        onClicked: r.toggled(!r.on)
    }
}
