import QtQuick
import Cord

Item {
    id: r
    property bool on: false
    property string label: ""
    signal toggled(bool v)
    implicitWidth: 36
    implicitHeight: 20
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
        border.color: r.on ? "transparent" : Th.ln
        border.width: 1
        Behavior on color { ColorAnimation { duration: Th.mF } }
        Rectangle {
            width: 14
            height: 14
            radius: 7
            y: 3
            x: r.on ? parent.width - width - 3 : 3
            color: r.on ? "#ffffff" : Th.tx2
            Behavior on x { NumberAnimation { duration: Th.mF; easing.type: Easing.OutCubic } }
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
