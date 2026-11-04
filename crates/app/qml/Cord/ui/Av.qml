import QtQuick
import QtQuick.Effects
import Cord

Item {
    id: r
    property string src: ""
    property string name: ""
    property real sz: 36
    property bool sq: false
    property string st: ""
    property color ring: Th.bg2
    property string sv: ""
    property bool speak: false
    property bool gray: false
    width: sz
    height: sz
    Rectangle {
        id: bgc
        anchors.fill: parent
        radius: r.sq ? Th.r * (r.sz / 48) * 1.4 : width / 2
        color: {
            let h = 0
            for (let i = 0; i < r.name.length; i++) h = (h * 31 + r.name.charCodeAt(i)) % 360
            return Qt.hsla(h / 360, r.gray ? 0.04 : Th.dark ? 0.32 : 0.4, Th.dark ? 0.32 : 0.62, 1)
        }
        visible: img.status !== Image.Ready
        Text {
            anchors.centerIn: parent
            text: (r.name || "?").replace(/[^\p{L}\p{N}]/gu, "").slice(0, r.sq ? 2 : 1).toUpperCase() || "?"
            color: "#fff"
            font.pixelSize: r.sz * 0.4
            font.family: Th.font
            font.weight: Font.DemiBold
        }
    }
    Image {
        id: img
        anchors.fill: parent
        source: S.u(r.src)
        sourceSize: Qt.size(r.sz * 2, r.sz * 2)
        fillMode: Image.PreserveAspectCrop
        asynchronous: true
        cache: true
        visible: false
    }
    Item {
        id: msk
        anchors.fill: parent
        visible: false
        layer.enabled: true
        layer.smooth: true
        Rectangle {
            anchors.fill: parent
            radius: bgc.radius
        }
    }
    MultiEffect {
        anchors.fill: parent
        source: img
        saturation: r.gray ? -1 : 0
        maskEnabled: true
        maskSource: msk
        maskThresholdMin: 0.5
        maskSpreadAtMin: 1.0
        visible: img.status === Image.Ready
    }
    Rectangle {
        anchors.fill: parent
        anchors.margins: -3
        radius: bgc.radius + 3
        color: "transparent"
        border.color: Th.ok
        border.width: 2
        visible: r.speak
    }
    Rectangle {
        visible: r.st !== ""
        width: r.sz * 0.36
        height: width
        radius: width / 2
        anchors.right: parent.right
        anchors.bottom: parent.bottom
        anchors.margins: -1
        color: Th.stCol(r.st)
        border.color: r.ring
        border.width: Math.max(2, r.sz * 0.07)
    }
    Badge {
        visible: r.sv !== "" && S.multi && S.ui.badges !== false
        sv: r.sv
        anchors.left: parent.left
        anchors.top: parent.top
        anchors.margins: -2
        ring: r.ring
        sz: Math.max(10, r.sz * 0.32)
    }
}
