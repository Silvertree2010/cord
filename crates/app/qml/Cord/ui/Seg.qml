import QtQuick
import Cord

Rectangle {
    id: sg
    property var opts: []
    property string cur
    signal pick(string v)
    implicitHeight: 28
    implicitWidth: rw.implicitWidth + 2
    width: implicitWidth
    radius: Th.rs
    color: Th.bg3
    border.color: Th.ln
    border.width: 1
    Row {
        id: rw
        x: 1
        y: 1
        spacing: 0
        Repeater {
            model: sg.opts
            Rectangle {
                id: it
                property bool on: sg.cur === modelData[0]
                width: sl.implicitWidth + 16
                height: 26
                radius: Th.rs - 1
                color: on ? Th.bg4 : sm.containsMouse ? Th.hov : "transparent"
                activeFocusOnTab: true
                Accessible.role: Accessible.Button
                Accessible.name: modelData[1]
                Keys.onReturnPressed: sg.pick(modelData[0])
                Keys.onSpacePressed: sg.pick(modelData[0])
                Lbl { id: sl; anchors.centerIn: parent; text: modelData[1]; color: it.on ? Th.tx : Th.tx2; font.pixelSize: Th.fsS }
                Rectangle { anchors.fill: parent; radius: parent.radius; color: "transparent"; border.color: Th.ac; border.width: 2; visible: it.activeFocus }
                MouseArea { id: sm; anchors.fill: parent; hoverEnabled: true; cursorShape: Qt.PointingHandCursor; onClicked: sg.pick(modelData[0]) }
            }
        }
    }
}
