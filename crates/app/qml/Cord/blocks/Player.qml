import QtQuick
import Cord

Rectangle {
    id: r
    property var o: ({})
    property var p: S.media || null
    property bool shown: S.f("spotify") && !!p && !!p.title
    visible: shown
    implicitWidth: 248
    implicitHeight: shown ? 76 : 0
    color: Th.bg3
    Image {
        id: art
        x: 10
        anchors.verticalCenter: parent.verticalCenter
        width: 52
        height: 52
        source: r.p ? (r.p.art || "") : ""
        fillMode: Image.PreserveAspectCrop
        sourceSize: Qt.size(104, 104)
        Rectangle { anchors.fill: parent; color: Th.bg4; visible: parent.status !== Image.Ready; z: -1; Ico { anchors.centerIn: parent; n: "music" } }
    }
    Column {
        anchors.left: art.right
        anchors.leftMargin: 10
        anchors.right: parent.right
        anchors.rightMargin: 8
        y: 10
        spacing: 1
        Lbl { width: parent.width; text: r.p ? r.p.title : ""; font.weight: Font.DemiBold; font.pixelSize: Th.fsS }
        Lbl { width: parent.width; text: r.p ? r.p.artist : ""; color: Th.tx3; font.pixelSize: Th.fsT }
        Row {
            spacing: 0
            IBtn { sz: 26; isz: 14; n: "prev"; onClicked: S.c.media("prev") }
            IBtn { sz: 26; isz: 14; n: r.p && r.p.playing ? "pause" : "play"; onClicked: S.c.media("toggle") }
            IBtn { sz: 26; isz: 14; n: "next"; onClicked: S.c.media("next") }
            IBtn { sz: 26; isz: 14; n: "link"; tip: S.t("Share in chat"); visible: !!S.selC && r.p && !!r.p.url; onClicked: S.send(S.selC, r.p.url) }
        }
    }
    Rectangle {
        anchors.bottom: parent.bottom
        height: 2
        width: r.p && r.p.len ? parent.width * Math.min(1, r.p.pos / r.p.len) : 0
        color: Th.ac
    }
}
