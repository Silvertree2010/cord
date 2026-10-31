import QtQuick
import QtQuick.Controls
import Cord

ScrollBar {
    id: r
    width: 8
    policy: ScrollBar.AsNeeded
    contentItem: Rectangle {
        implicitWidth: 6
        radius: 3
        color: r.pressed ? Th.tx3 : Th.bg4
        opacity: r.active ? 1 : 0.6
    }
    background: Item {}
}
