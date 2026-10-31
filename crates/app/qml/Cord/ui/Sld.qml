import QtQuick
import QtQuick.Controls
import Cord

Slider {
    id: r
    implicitHeight: 20
    background: Rectangle {
        x: r.leftPadding
        y: r.topPadding + (r.availableHeight - height) / 2
        width: r.availableWidth
        height: 4
        radius: 2
        color: Th.bg4
        Rectangle {
            width: r.visualPosition * parent.width
            height: parent.height
            radius: 2
            color: Th.ac
        }
    }
    handle: Rectangle {
        x: r.leftPadding + r.visualPosition * (r.availableWidth - width)
        y: r.topPadding + (r.availableHeight - height) / 2
        width: 14
        height: 14
        radius: 7
        color: Th.ac
        border.color: r.activeFocus ? Th.tx : Th.bg
        border.width: 2
    }
}
