import QtQuick
import Cord

Rectangle {
    id: r
    property string text: ""
    property string ico: ""
    property bool pri: false
    property bool dng: false
    property bool flat: false
    property bool on: true
    property color tc: pri ? Th.acTx : dng ? "#fff" : Th.tx
    signal clicked
    implicitWidth: row.implicitWidth + 24
    implicitHeight: 34 * Th.dens
    radius: Th.rs
    opacity: on ? 1 : 0.45
    activeFocusOnTab: on
    Accessible.role: Accessible.Button
    Accessible.name: text
    Keys.onReturnPressed: if (on) r.clicked()
    Keys.onEnterPressed: if (on) r.clicked()
    Keys.onSpacePressed: if (on) r.clicked()
    color: pri ? (ma.containsMouse ? Qt.lighter(Th.ac, 1.08) : Th.ac) : dng ? (ma.containsMouse ? Qt.lighter(Th.err, 1.1) : Th.err) : flat ? (ma.containsMouse ? Th.hov : "transparent") : (ma.containsMouse ? Th.bg4 : Th.bg3)
    Row {
        id: row
        anchors.centerIn: parent
        spacing: 6
        Ico {
            visible: r.ico !== ""
            n: r.ico
            sz: 16
            col: lb.color
            anchors.verticalCenter: parent.verticalCenter
        }
        Lbl {
            id: lb
            text: r.text
            visible: text !== ""
            color: r.tc
            font.weight: Font.Medium
            anchors.verticalCenter: parent.verticalCenter
        }
    }
    Rectangle {
        anchors.fill: parent
        radius: r.radius
        color: "transparent"
        border.color: Th.ac
        border.width: 2
        visible: r.activeFocus
    }
    MouseArea {
        id: ma
        anchors.fill: parent
        hoverEnabled: true
        cursorShape: Qt.PointingHandCursor
        enabled: r.on
        onClicked: r.clicked()
    }
}
