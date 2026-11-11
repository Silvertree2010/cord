import QtQuick
import Cord

Item {
    id: r
    property string src: ""
    property string still: ""
    property int max: 0
    property bool playing: true
    property bool once: false
    signal ended
    readonly property var sh: r.src !== "" ? S.sheetOf(r.src, r.max) : null
    readonly property bool ready: !!r.sh && !!r.sh.path
    Image {
        anchors.fill: parent
        visible: !r.ready && r.still !== ""
        source: r.still
        fillMode: Image.PreserveAspectFit
        asynchronous: true
        cache: true
    }
    AnimatedSprite {
        id: spr
        anchors.fill: parent
        visible: r.ready
        source: r.ready ? "file://" + r.sh.path : ""
        frameCount: r.ready ? r.sh.n : 1
        frameWidth: r.ready ? r.sh.w : 1
        frameHeight: r.ready ? r.sh.h : 1
        frameDuration: r.ready ? r.sh.ms : 100
        interpolate: false
        loops: r.once ? 1 : AnimatedSprite.Infinite
        running: r.ready && r.playing && r.visible
        onFinished: r.ended()
    }
    function restart() { spr.restart() }
}
