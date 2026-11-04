import QtQuick
import QtQuick.Effects
import Cord

Item {
    id: r
    property color color: Th.bg3
    property real radius: Th.r
    Rectangle {
        id: sh
        anchors.fill: parent
        radius: r.radius
        color: Th.dark ? "#000000" : "#40000000"
        visible: false
    }
    MultiEffect {
        anchors.fill: sh
        source: sh
        shadowEnabled: true
        shadowColor: "#000000"
        shadowOpacity: Th.dark ? 0.55 : 0.18
        shadowBlur: 0.9
        shadowVerticalOffset: 8
        blurMax: 32
        autoPaddingEnabled: true
        maskEnabled: false
        opacity: 1
    }
    Rectangle {
        anchors.fill: parent
        radius: r.radius
        color: r.color
        border.color: Th.dark ? Qt.lighter(Th.bg4, 1.1) : Th.ln
        border.width: 1
    }
}
