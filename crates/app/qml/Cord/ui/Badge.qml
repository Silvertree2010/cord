import QtQuick
import Cord

Rectangle {
    id: r
    property string sv: "dc"
    property real sz: 14
    property color ring: Th.bg2
    property bool txt: false
    width: txt ? lb.implicitWidth + 8 : sz
    height: sz
    radius: height / 2
    color: Th.svcCol(sv)
    border.color: ring
    border.width: txt ? 0 : Math.max(1.5, sz * 0.14)
    visible: S.ui.badges !== false
    Text {
        id: lb
        anchors.centerIn: parent
        text: r.txt ? Th.svcName(r.sv) : (r.sv === "Mx" || r.sv === "mx" ? "M" : r.sv === "St" || r.sv === "st" ? "S" : "D")
        color: "#fff"
        font.pixelSize: r.txt ? 10 : r.sz * 0.55
        font.family: Th.font
        font.weight: Font.Bold
    }
}
