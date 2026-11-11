import QtQuick
import QtMultimedia

Item {
    id: g
    property string src: ""
    property int fill: VideoOutput.PreserveAspectFit
    property real vw: vo.sourceRect.width
    property real vh: vo.sourceRect.height
    MediaPlayer {
        id: mp
        source: g.src
        loops: MediaPlayer.Infinite
        videoOutput: vo
        audioOutput: null
        onSourceChanged: if (source.toString() !== "") play()
        Component.onCompleted: if (source.toString() !== "") play()
    }
    VideoOutput { id: vo; anchors.fill: parent; fillMode: g.fill }
}
