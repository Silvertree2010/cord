import QtQuick
import QtMultimedia

Item {
    id: g
    property string src: ""
    property real vw: vo.sourceRect.width
    property real vh: vo.sourceRect.height
    MediaPlayer {
        id: mp
        source: g.src
        loops: MediaPlayer.Infinite
        videoOutput: vo
        audioOutput: null
        Component.onCompleted: play()
    }
    VideoOutput { id: vo; anchors.fill: parent; fillMode: VideoOutput.PreserveAspectFit }
}
