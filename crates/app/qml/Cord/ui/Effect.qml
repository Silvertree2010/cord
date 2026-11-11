import QtQuick
import Cord

Item {
    id: r
    property var cfg: null
    property bool playing: true
    readonly property var lays: r.cfg && r.cfg.effects ? r.cfg.effects : []
    readonly property real refW: {
        let m = 1
        for (const l of r.lays) m = Math.max(m, (l.position ? l.position.x : 0) + (l.width || 0))
        return m
    }
    readonly property real k: r.width / r.refW
    clip: true
    Repeater {
        model: r.lays
        Item {
            id: ly
            required property var modelData
            readonly property var p: modelData.position || ({ x: 0, y: 0 })
            x: ly.p.x * r.k
            y: ly.p.y * r.k
            z: modelData.zIndex || 0
            width: (modelData.width || 0) * r.k
            height: (modelData.height || 0) * r.k
            property bool live: false
            Timer {
                id: st
                interval: Math.max(1, ly.modelData.start || 0)
                running: r.playing
                onTriggered: { ly.live = true; ap.restart() }
            }
            Timer {
                interval: Math.max(16, (ly.modelData.duration || 0) + (ly.modelData.loopDelay || 0))
                running: ly.live && !!ly.modelData.loop && (ly.modelData.loopDelay || 0) > 0
                repeat: true
                onTriggered: ap.restart()
            }
            Apng {
                id: ap
                anchors.fill: parent
                visible: ly.live
                src: ly.modelData.src || ""
                max: Math.round(Math.max(ly.width, ly.height) * 1.5)
                once: !ly.modelData.loop || (ly.modelData.loopDelay || 0) > 0
                playing: ly.live && r.playing
                onEnded: if (!ly.modelData.loop) ly.live = false
            }
        }
    }
    onPlayingChanged: if (!playing) for (let i = 0; i < children.length; i++) if (children[i].live !== undefined) children[i].live = false
}
