import QtQuick
import Cord

Item {
    id: r
    property string sv: "dc"
    property real sz: 14
    property color ring: Th.bg2
    property bool txt: false
    property real rw: Math.max(1.5, sz * 0.12)
    readonly property string k: sv === "Mx" || sv === "mx" ? "mx" : sv === "St" || sv === "st" ? "st" : "dc"
    width: txt ? lb2.implicitWidth + 10 : sz
    height: txt ? 16 : sz
    visible: S.ui.badges !== false && (txt || S.multi)
    Rectangle {
        visible: r.txt
        anchors.fill: parent
        radius: 4
        color: Th.svcCol(r.sv)
        Text {
            id: lb2
            anchors.centerIn: parent
            text: Th.svcName(r.sv)
            color: "#fff"
            font.pixelSize: 10
            font.family: Th.font
            font.weight: Font.Bold
        }
    }
    Rectangle {
        visible: !r.txt
        anchors.centerIn: parent
        width: r.k === "st" ? r.sz * 0.78 : r.sz
        height: width
        rotation: r.k === "st" ? 45 : 0
        radius: r.k === "dc" ? width / 2 : r.k === "mx" ? r.sz * 0.24 : 2
        color: Th.svcCol(r.sv)
        border.color: r.ring
        border.width: r.rw
    }
    Text {
        visible: !r.txt
        anchors.centerIn: parent
        text: r.k === "mx" ? "M" : r.k === "st" ? "S" : "D"
        color: "#fff"
        font.pixelSize: r.sz * (r.k === "st" ? 0.46 : 0.56)
        font.family: Th.font
        font.weight: Font.Bold
    }
}
