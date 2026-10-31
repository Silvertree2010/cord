import QtQuick
import QtQuick.Controls
import QtMultimedia
import Cord

Rectangle {
    id: r
    property string url: ""
    property bool vid: false
    color: "#000000e6"
    visible: url !== ""
    MouseArea { anchors.fill: parent; onClicked: { mp.stop(); r.url = "" } }
    Image {
        visible: !r.vid
        anchors.centerIn: parent
        width: Math.min(parent.width - 80, sourceSize.width > 0 ? sourceSize.width : parent.width)
        height: Math.min(parent.height - 120, sourceSize.height > 0 ? sourceSize.height : parent.height)
        fillMode: Image.PreserveAspectFit
        source: r.vid ? "" : r.url
        asynchronous: true
    }
    MediaPlayer {
        id: mp
        source: r.vid ? r.url : ""
        videoOutput: vo
        audioOutput: AudioOutput {}
        onSourceChanged: if (r.vid && source != "") play()
    }
    VideoOutput {
        id: vo
        visible: r.vid
        anchors.fill: parent
        anchors.margins: 60
    }
    Row {
        anchors.bottom: parent.bottom
        anchors.horizontalCenter: parent.horizontalCenter
        anchors.bottomMargin: 20
        spacing: 8
        Btn { text: "Open in browser"; onClicked: Qt.openUrlExternally(r.url) }
        Btn { text: "Copy link"; onClicked: S.copy(r.url) }
        Btn { visible: r.vid; text: mp.playbackState === MediaPlayer.PlayingState ? "Pause" : "Play"; onClicked: mp.playbackState === MediaPlayer.PlayingState ? mp.pause() : mp.play() }
    }
}
