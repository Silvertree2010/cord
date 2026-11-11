import QtQuick
import QtQuick.Effects
import Cord

Item {
    id: r
    property string src: ""
    property real radius: 6
    property bool anim: false
    readonly property bool on: r.src !== "" && S.ui.plates !== false
    visible: r.on
    Item {
        id: art
        anchors.fill: parent
        clip: true
        Image {
            anchors.fill: parent
            visible: !ap.ready
            source: r.on ? r.src + "static.png" : ""
            fillMode: Image.PreserveAspectCrop
            asynchronous: true
            cache: true
        }
        Apng {
            id: ap
            width: parent.width
            height: ap.ready ? width * ap.sh.h / ap.sh.w : parent.height
            anchors.verticalCenter: parent.verticalCenter
            src: r.on && r.anim && Th.motion ? r.src + "asset.webm" : ""
            max: 672
            playing: r.anim
        }
    }
    Rectangle {
        id: msk
        anchors.fill: parent
        radius: r.radius
        visible: false
        layer.enabled: true
    }
    MultiEffect {
        anchors.fill: parent
        source: ShaderEffectSource { sourceItem: art; live: true; hideSource: true }
        maskEnabled: true
        maskSource: msk
    }
}
