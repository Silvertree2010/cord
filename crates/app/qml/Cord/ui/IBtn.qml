import QtQuick
import Cord

Rectangle {
    id: r
    property string n: "x"
    property string tip: ""
    property bool act: false
    property bool red: false
    property real sz: 32
    property real isz: 18
    property int mods: 0
    signal clicked
    width: sz
    height: sz
    radius: Th.rs
    activeFocusOnTab: true
    Accessible.role: Accessible.Button
    Accessible.name: tip !== "" ? tip : n
    Keys.onReturnPressed: r.clicked()
    Keys.onEnterPressed: r.clicked()
    Keys.onSpacePressed: r.clicked()
    color: red ? (ma.containsMouse ? Qt.lighter(Th.err, 1.1) : Th.a(Th.err, 0.2)) : act ? Th.sel : ma.containsMouse ? Th.hov : "transparent"
    Behavior on color { ColorAnimation { duration: Th.mF } }
    scale: ma.pressed ? 0.9 : 1
    Behavior on scale { NumberAnimation { duration: Th.mF; easing.type: Easing.OutBack } }
    Ico {
        anchors.centerIn: parent
        n: r.n
        sz: r.isz
        col: r.red ? (ma.containsMouse ? "#fff" : Th.err) : r.act ? Th.tx : ma.containsMouse ? Th.tx : Th.tx2
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
        onPressed: tp.cancel()
        onClicked: m => { r.mods = m.modifiers; r.clicked() }
    }
    Tip { id: tp; text: r.tip; hov: ma.containsMouse && r.tip !== "" }
}
