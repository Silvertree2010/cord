import QtQuick
import Cord

Rectangle {
    id: r
    property int n: 0
    property bool ring: false
    height: 16
    width: Math.max(16, lb.implicitWidth + 8)
    radius: 8
    color: Th.mr
    border.color: Th.bg
    border.width: ring ? 2 : 0
    Lbl { id: lb; anchors.centerIn: parent; text: r.n > 99 ? "99+" : r.n; color: Th.wh; font.pixelSize: Th.fsP; font.weight: Font.Bold }
}
