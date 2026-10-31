import QtQuick
import Cord
import "../Fmt.js" as F

Rectangle {
    id: r
    property var e: ({})
    property real maxW: 520
    readonly property bool media: !e.title && !e.desc && (e.img || e.thumb || e.vid)
    width: media ? im.width : Math.min(maxW, Math.max(cl.implicitWidth + 28, 200))
    height: media ? im.height : cl.implicitHeight + 24
    radius: Th.rs
    color: media ? "transparent" : Th.bg3
    Rectangle {
        visible: !r.media
        width: 4
        height: parent.height
        radius: 2
        color: r.e.col ? "#" + ("000000" + r.e.col.toString(16)).slice(-6) : Th.bg4
    }
    Column {
        id: cl
        visible: !r.media
        x: 16
        y: 12
        width: r.maxW - 32
        spacing: 6
        Lbl { visible: !!r.e.prov; text: r.e.prov || ""; color: Th.tx3; font.pixelSize: 11; width: parent.width }
        Lbl { visible: !!r.e.auth; text: r.e.auth || ""; font.pixelSize: Th.fsS; font.weight: Font.DemiBold; width: parent.width }
        Text {
            visible: !!r.e.title
            width: Math.min(implicitWidth, parent.width)
            text: r.e.url ? '<a href="' + r.e.url + '" style="color:' + Th.ac + ';text-decoration:none">' + F.esc(r.e.title || "") + '</a>' : F.esc(r.e.title || "")
            textFormat: Text.RichText
            color: Th.tx
            font.family: Th.font
            font.pixelSize: Th.fs
            font.weight: Font.DemiBold
            wrapMode: Text.Wrap
            onLinkActivated: l => Qt.openUrlExternally(l)
        }
        Text {
            visible: !!r.e.desc
            width: Math.min(implicitWidth, parent.width)
            text: F.html(r.e.desc || "", { ac: Th.ac, cb: Th.bg4, mono: Th.mono, names: S.names })
            textFormat: Text.RichText
            color: Th.tx2
            font.family: Th.font
            font.pixelSize: Th.fsS
            wrapMode: Text.Wrap
            onLinkActivated: l => Qt.openUrlExternally(l)
        }
        Repeater {
            model: r.e.fields || []
            Column {
                width: parent.width
                Lbl { text: modelData[0]; font.pixelSize: Th.fsS; font.weight: Font.DemiBold; width: parent.width }
                Text { text: F.html(modelData[1], { ac: Th.ac, cb: Th.bg4, names: S.names }); textFormat: Text.RichText; color: Th.tx2; font.pixelSize: Th.fsS; width: parent.width; wrapMode: Text.Wrap }
            }
        }
        Image {
            visible: !!r.e.img
            source: r.e.img || ""
            width: Math.min(parent.width, sourceSize.width > 0 ? sourceSize.width : 400)
            fillMode: Image.PreserveAspectFit
            asynchronous: true
            MouseArea { anchors.fill: parent; cursorShape: Qt.PointingHandCursor; onClicked: S.view(r.e.img) }
        }
        Lbl { visible: !!r.e.foot; text: r.e.foot || ""; color: Th.tx3; font.pixelSize: 11; width: parent.width }
    }
    Image {
        id: im
        visible: r.media
        source: r.media ? (r.e.img || r.e.thumb || "") : ""
        property real k: implicitWidth > 0 ? Math.min(1, 420 / implicitWidth, 320 / implicitHeight) : 1
        width: implicitWidth * k
        height: implicitHeight * k
        asynchronous: true
        fillMode: Image.PreserveAspectFit
        MouseArea { anchors.fill: parent; cursorShape: Qt.PointingHandCursor; onClicked: r.e.vid ? S.view(r.e.vid, true) : S.view(im.source) }
    }
}
