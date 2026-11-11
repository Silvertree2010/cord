import QtQuick
import QtMultimedia
import Cord

Rectangle {
    id: r
    property string src: ""
    property string name: ""
    width: 360
    height: 56
    radius: Th.rs
    color: Th.bg3
    border.color: Th.ln
    function t(ms) {
        const s = Math.max(0, Math.floor(ms / 1000))
        return Math.floor(s / 60) + ":" + ("0" + (s % 60)).slice(-2)
    }
    MediaPlayer {
        id: mp
        audioOutput: AudioOutput {}
    }
    Rectangle {
        id: pb
        x: 12
        anchors.verticalCenter: parent.verticalCenter
        width: 34
        height: 34
        radius: 17
        color: Th.ac
        Ico { anchors.centerIn: parent; n: mp.playbackState === MediaPlayer.PlayingState ? "pause" : "play"; sz: 16; col: Th.acTx }
        MouseArea {
            anchors.fill: parent
            cursorShape: Qt.PointingHandCursor
            onClicked: {
                if (!mp.source.toString()) mp.source = r.src
                if (mp.playbackState === MediaPlayer.PlayingState) mp.pause(); else mp.play()
            }
        }
    }
    Column {
        anchors.left: pb.right
        anchors.leftMargin: 12
        anchors.right: parent.right
        anchors.rightMargin: 12
        anchors.verticalCenter: parent.verticalCenter
        spacing: 6
        Lbl { width: parent.width; text: r.name; color: Th.tx2; font.pixelSize: Th.fsT }
        Item {
            width: parent.width
            height: 12
            Rectangle { anchors.verticalCenter: parent.verticalCenter; width: parent.width - tl.width - 8; height: 4; radius: 2; color: Th.bg4 }
            Rectangle { anchors.verticalCenter: parent.verticalCenter; width: mp.duration > 0 ? (parent.width - tl.width - 8) * mp.position / mp.duration : 0; height: 4; radius: 2; color: Th.ac }
            MouseArea {
                width: parent.width - tl.width - 8
                height: parent.height
                cursorShape: Qt.PointingHandCursor
                onClicked: m => { if (mp.duration > 0) mp.position = mp.duration * m.x / width }
            }
            Lbl { id: tl; anchors.right: parent.right; anchors.verticalCenter: parent.verticalCenter; text: r.t(mp.position) + (mp.duration > 0 ? " / " + r.t(mp.duration) : ""); color: Th.tx3; font.pixelSize: Th.fsT }
        }
    }
}
