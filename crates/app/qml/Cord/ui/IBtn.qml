import QtQuick
import QtQuick.Controls
import Cord

Rectangle {
    id: r
    property string n: "x"
    property string tip: ""
    property bool act: false
    property bool red: false
    property real sz: 32
    property real isz: 18
    signal clicked
    width: sz
    height: sz
    radius: Th.rs
    color: red ? (ma.containsMouse ? Qt.lighter(Th.err, 1.1) : Th.err + "33") : act ? Th.sel : ma.containsMouse ? Th.hov : "transparent"
    Ico {
        anchors.centerIn: parent
        n: r.n
        sz: r.isz
        col: r.red ? (ma.containsMouse ? "#fff" : Th.err) : r.act ? Th.tx : ma.containsMouse ? Th.tx : Th.tx2
    }
    MouseArea {
        id: ma
        anchors.fill: parent
        hoverEnabled: true
        cursorShape: Qt.PointingHandCursor
        onClicked: r.clicked()
    }
    ToolTip.visible: tip !== "" && ma.containsMouse
    ToolTip.text: tip
    ToolTip.delay: 500
}
