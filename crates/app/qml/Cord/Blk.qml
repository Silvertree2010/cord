import QtQuick
import Cord
import "Lay.js" as L

Item {
    id: r
    property var n: ({})
    property var p: []
    readonly property real strip: 24
    implicitWidth: Math.max(ld.item ? ld.item.implicitWidth : 0, S.edit ? 72 : 0)
    implicitHeight: S.edit ? Math.max(ld.item ? ld.item.implicitHeight : 0, 40) + strip : (ld.item ? ld.item.implicitHeight : 0)
    property bool shown: S.edit || !ld.item || ld.item.shown !== false
    property string nm: (S.allBlks().find(b => b.id === r.n.b) || { name: r.n.b }).name
    property bool canRm: !(r.n.b === "chat" && U.cnt("chat") <= 1)
    visible: shown
    clip: !S.edit
    readonly property bool card: !S.edit
    readonly property real gap: card ? Th.gap / 2 : 0
    readonly property bool deep: r.n.b === "mems"
    Rectangle {
        id: cardBg
        visible: r.card
        anchors.fill: ld
        radius: Th.r
        color: r.deep ? Th.bg : Th.bgc
    }
    Rectangle {
        visible: r.card
        anchors.fill: ld
        z: 10
        radius: Th.r
        color: "transparent"
        border.color: Th.ln
        border.width: 1
    }

    Loader {
        id: ld
        anchors.fill: parent
        anchors.topMargin: S.edit ? r.strip : r.gap
        anchors.bottomMargin: r.gap
        anchors.rightMargin: r.gap
        anchors.leftMargin: r.gap
        source: S.blkUrl(r.n.b)
        onLoaded: {
            if (item && "o" in item) item.o = r.n.o || {}
            if (item && "color" in item) item.color = Qt.binding(() => r.card ? "transparent" : (r.deep ? Th.bg : Th.bg2))
        }
        onStatusChanged: if (status === Loader.Error) console.warn("block failed:", r.n.b)
    }

    Item {
        anchors.fill: parent
        visible: S.edit
        z: 20

        Lbl {
            visible: !ld.item || ld.item.shown === false || ld.item.implicitHeight < 8
            anchors.horizontalCenter: parent.horizontalCenter
            y: r.strip + (parent.height - r.strip - height) / 2
            text: r.nm
            color: Th.tx3
        }

        Rectangle {
            id: hl
            property string zn: "c"
            visible: dz.containsDrag
            color: Th.a(Th.ac, 0.14)
            border.color: Th.ac
            border.width: 1
            x: zn === "r" ? parent.width / 2 : 0
            y: zn === "b" ? parent.height / 2 : zn === "t" ? 0 : 0
            width: zn === "l" || zn === "r" ? parent.width / 2 : parent.width
            height: zn === "t" || zn === "b" ? parent.height / 2 : parent.height
        }

        Rectangle {
            anchors.fill: parent
            color: "transparent"
            border.color: Th.a(Th.ac, 0.7)
            border.width: 1
        }

        Rectangle {
            id: sb
            anchors.top: parent.top
            anchors.left: parent.left
            anchors.right: parent.right
            height: r.strip
            color: Th.bg3
            Rectangle { anchors.bottom: parent.bottom; width: parent.width; height: 1; color: Th.a(Th.ac, 0.7) }
            MouseArea {
                id: dm
                anchors.fill: parent
                anchors.rightMargin: r.canRm ? 24 : 0
                cursorShape: Qt.SizeAllCursor
                drag.target: px
                onPressed: U.hideTip()
                onReleased: px.Drag.drop()
            }
            Ico { id: gi; n: "drag"; sz: 14; col: Th.tx3; x: 6; anchors.verticalCenter: parent.verticalCenter }
            Lbl {
                visible: r.width >= 120
                anchors.left: gi.right
                anchors.leftMargin: 6
                anchors.right: parent.right
                anchors.rightMargin: 28
                anchors.verticalCenter: parent.verticalCenter
                text: r.nm
                font.weight: Font.DemiBold
                font.pixelSize: Th.fsS
            }
            IBtn {
                visible: r.canRm
                anchors.right: parent.right
                anchors.verticalCenter: parent.verticalCenter
                n: "x"
                sz: 24
                isz: 12
                onClicked: U.setLay(L.rm(S.lay, r.p).root)
            }
        }

        DropArea {
            id: dz
            anchors.fill: parent
            keys: ["cord-blk"]
            onPositionChanged: d => {
                const fx = d.x / width, fy = d.y / height
                const m = Math.min(fx, 1 - fx, fy, 1 - fy)
                hl.zn = m > 0.3 ? "c" : m === fx ? "l" : m === 1 - fx ? "r" : m === fy ? "t" : "b"
            }
            onDropped: d => {
                const from = d.source.path
                if (!from) return
                U.setLay(L.move(S.lay, from, r.p, hl.zn))
            }
        }
    }

    Item {
        id: px
        property var path: r.p
        width: 120
        height: 32
        visible: dm.drag.active
        z: 99
        Drag.active: dm.drag.active
        Drag.keys: ["cord-blk"]
        Drag.source: px
        Drag.hotSpot.x: 60
        Drag.hotSpot.y: 16
        Rectangle {
            anchors.fill: parent
            radius: Th.rs
            color: Th.bg4
            border.color: Th.ac
            Lbl { anchors.centerIn: parent; width: parent.width - 8; horizontalAlignment: Text.AlignHCenter; text: r.nm }
        }
        onVisibleChanged: if (!visible) { x = 0; y = 0 }
    }
}
