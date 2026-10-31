import QtQuick
import QtQuick.Controls
import Cord

Slider {
    id: r
    property bool lab: false
    property string unit: ""
    implicitHeight: 20
    rightPadding: lab ? 56 : 0
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
    Lbl {
        visible: r.lab
        anchors.right: parent.right
        anchors.verticalCenter: parent.verticalCenter
        width: 48
        horizontalAlignment: Text.AlignRight
        text: Math.round(r.value) + r.unit
        color: Th.tx2
        font.pixelSize: Th.fsS
    }
}
