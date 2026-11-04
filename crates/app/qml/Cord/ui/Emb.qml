import QtQuick
import Cord
import "../Fmt.js" as F

Rectangle {
    id: r
    property var e: ({})
    property real maxW: 520
    readonly property string kind: e.kind || ""
    readonly property bool gifv: kind === "gifv" && !!e.vid
    readonly property bool media: gifv || kind === "image" || (!e.title && !e.desc && !e.auth && !(e.fields || []).length && !!(e.img || e.thumb || e.vid))
    readonly property bool vidCard: kind === "video" && !!e.thumb
    readonly property bool bigThumb: !media && (vidCard || kind === "article") && !!e.thumb && !e.img
    readonly property bool sideThumb: !media && !bigThumb && !!e.thumb && !e.img
    readonly property string mediaSrc: e.img || e.thumb || ""
    width: media ? mw.width : Math.min(maxW, 520)
    height: media ? mw.height : cl.implicitHeight + 24
    radius: Th.rs
    color: media ? "transparent" : Th.bg3
    function open(u) { if (u) Qt.openUrlExternally(u) }
    Rectangle {
        visible: !r.media
        width: 4
        height: parent.height
        radius: 2
        color: r.e.col ? "#" + ("000000" + r.e.col.toString(16)).slice(-6) : Th.bg4
    }
    Image {
        id: side
        visible: r.sideThumb
        source: r.sideThumb ? r.e.thumb : ""
        anchors.right: parent.right
        anchors.top: parent.top
        anchors.margins: 12
        width: 80
        height: 80
        fillMode: Image.PreserveAspectFit
        horizontalAlignment: Image.AlignRight
        verticalAlignment: Image.AlignTop
        asynchronous: true
        MouseArea { anchors.fill: parent; cursorShape: Qt.PointingHandCursor; onClicked: S.view(r.e.thumb) }
    }
    Column {
        id: cl
        visible: !r.media
        x: 16
        y: 12
        width: r.width - 32 - (r.sideThumb ? 92 : 0)
        spacing: 6
        Lbl { visible: !!r.e.prov; text: r.e.prov || ""; color: Th.tx3; font.pixelSize: 11; width: parent.width }
        Row {
            visible: !!r.e.auth
            spacing: 8
            width: parent.width
            Image { visible: !!r.e.auth_ic; source: r.e.auth_ic || ""; width: 22; height: 22; sourceSize: Qt.size(44, 44); asynchronous: true; anchors.verticalCenter: parent.verticalCenter }
            Lbl { text: r.e.auth || ""; font.pixelSize: Th.fsS; font.weight: Font.DemiBold; anchors.verticalCenter: parent.verticalCenter; width: parent.width - (r.e.auth_ic ? 30 : 0) }
        }
        Text {
            visible: !!r.e.title
            width: parent.width
            text: r.e.url ? '<a href="' + r.e.url + '" style="color:' + Th.ac + ';text-decoration:none">' + F.inl(F.esc(r.e.title || ""), { ac: Th.ac, cb: Th.bg4, mono: Th.mono, names: S.names }) + '</a>' : F.inl(F.esc(r.e.title || ""), { ac: Th.ac, cb: Th.bg4, mono: Th.mono, names: S.names })
            textFormat: Text.RichText
            color: Th.tx
            font.family: Th.font
            font.pixelSize: Th.fs
            font.weight: Font.DemiBold
            wrapMode: Text.Wrap
            onLinkActivated: l => r.open(l)
        }
        Text {
            visible: !!r.e.desc
            width: parent.width
            text: F.html(r.e.desc || "", { ac: Th.ac, cb: Th.bg4, mono: Th.mono, names: S.names })
            textFormat: Text.RichText
            color: Th.tx2
            font.family: Th.font
            font.pixelSize: Th.fsS
            wrapMode: Text.Wrap
            onLinkActivated: l => r.open(l)
        }
        Flow {
            visible: (r.e.fields || []).length > 0
            width: parent.width
            spacing: 8
            Repeater {
                model: r.e.fields || []
                Column {
                    width: modelData[2] ? Math.floor((cl.width - 16) / 3) : cl.width
                    Lbl { text: modelData[0]; font.pixelSize: Th.fsS; font.weight: Font.DemiBold; width: parent.width }
                    Text { text: F.html(modelData[1], { ac: Th.ac, cb: Th.bg4, mono: Th.mono, names: S.names }); textFormat: Text.RichText; color: Th.tx2; font.family: Th.font; font.pixelSize: Th.fsS; width: parent.width; wrapMode: Text.Wrap; onLinkActivated: l => r.open(l) }
                }
            }
        }
        Item {
            visible: !!r.e.img || r.bigThumb
            width: Math.min(parent.width, bi.implicitWidth > 0 ? bi.implicitWidth : parent.width)
            height: bi.implicitWidth > 0 ? width * bi.implicitHeight / bi.implicitWidth : 0
            Image {
                id: bi
                anchors.fill: parent
                source: r.e.img || (r.bigThumb ? r.e.thumb : "")
                fillMode: Image.PreserveAspectFit
                asynchronous: true
            }
            Rectangle {
                visible: r.vidCard
                anchors.centerIn: parent
                width: 52
                height: 52
                radius: 26
                color: Th.a("#000000", 0.6)
                Ico { anchors.centerIn: parent; n: "play"; sz: 24; col: "#ffffff" }
            }
            MouseArea { anchors.fill: parent; cursorShape: Qt.PointingHandCursor; onClicked: r.vidCard ? r.open(r.e.url) : S.view(bi.source) }
        }
        Lbl { visible: !!r.e.foot; text: r.e.foot || ""; color: Th.tx3; font.pixelSize: 11; width: parent.width }
    }
    Item {
        id: mw
        visible: r.media
        property real iw: r.gifv && gv.item && gv.item.vw > 0 ? gv.item.vw : im.implicitWidth
        property real ih: r.gifv && gv.item && gv.item.vh > 0 ? gv.item.vh : im.implicitHeight
        property real k: iw > 0 ? Math.min(1, Math.min(r.maxW, 420) / iw, 320 / ih) : 1
        width: iw > 0 ? iw * k : (r.media && (r.mediaSrc || r.e.vid) ? 240 : 0)
        height: ih > 0 ? ih * k : (r.media && (r.mediaSrc || r.e.vid) ? 135 : 0)
        Rectangle { anchors.fill: parent; radius: Th.rs; color: Th.bg3; visible: im.status !== Image.Ready && !(gv.item && gv.item.vw > 0) }
        Image {
            id: im
            anchors.fill: parent
            visible: !(gv.item && gv.item.vw > 0)
            source: r.media ? r.mediaSrc : ""
            asynchronous: true
            fillMode: Image.PreserveAspectFit
        }
        Loader {
            id: gv
            anchors.fill: parent
            active: r.gifv && r.visible
            source: "GifV.qml"
            onLoaded: item.src = r.e.vid
        }
        Rectangle {
            visible: r.media && !r.gifv && !!r.e.vid && !r.mediaSrc
            anchors.centerIn: parent
            width: 52
            height: 52
            radius: 26
            color: Th.a("#000000", 0.6)
            Ico { anchors.centerIn: parent; n: "play"; sz: 24; col: "#ffffff" }
        }
        MouseArea { anchors.fill: parent; cursorShape: Qt.PointingHandCursor; onClicked: r.gifv ? S.view(r.e.vid, true) : r.e.vid ? S.view(r.e.vid, true) : S.view(im.source) }
    }
}
