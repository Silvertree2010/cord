import QtQuick
import QtQuick.Controls
import Cord

ScrollBar {
    id: r
    width: 10
    policy: ScrollBar.AsNeeded
    contentItem: Rectangle {
        implicitWidth: r.hovered || r.pressed ? 8 : 4
        radius: width / 2
        color: r.pressed ? Th.tx3 : r.hovered ? Th.tx3 : Th.bg4
        opacity: r.active || r.hovered ? 1 : 0
        Behavior on opacity { NumberAnimation { duration: Th.mM } }
        Behavior on implicitWidth { NumberAnimation { duration: Th.mF } }
    }
    background: Item {}
}
