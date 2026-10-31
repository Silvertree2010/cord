import QtQuick
import Cord
import "Lay.js" as L

Item {
    id: r
    property var n: ({})
    property var p: []
    implicitWidth: ld.item ? ld.item.implicitWidth : 0
    implicitHeight: ld.item ? ld.item.implicitHeight : 0
    property bool shown: S.edit || !ld.item || ld.item.shown !== false
    property string nm: (S.allBlks().find(b => b.id === r.n.b) || { name: r.n.b }).name
    property bool canRm: !(r.n.b === "chat" && U.cnt("chat") <= 1)
    property bool tiny: r.width < 130
    visible: shown
    clip: !S.edit

    Loader {
        id: ld
        anchors.fill: parent
        source: S.blkUrl(r.n.b)
        onLoaded: { if (item && "o" in item) item.o = r.n.o || {} }
        onStatusChanged: if (status === Loader.Error) console.warn("block failed:", r.n.b)
    }

    Rectangle {
        anchors.fill: parent
        visible: S.edit
        color: Th.a(Th.bg, 0.35)
        border.color: Th.ac
        border.width: dz.containsDrag ? 2 : 1
        z: 20

        Rectangle {
            id: hl
            color: Th.a(Th.ac, 0.25)
            visible: dz.containsDrag
            property string zn: "c"
            x: zn === "r" ? parent.width / 2 : 0
            y: zn === "b" ? parent.height / 2 : 0
            width: zn === "l" || zn === "r" ? parent.width / 2 : parent.width
            height: zn === "t" || zn === "b" ? parent.height / 2 : parent.height
        }

        Rectangle {
            visible: !r.tiny
            anchors.top: parent.top
            anchors.left: parent.left
            anchors.right: parent.right
            height: 28
            color: Th.bg3
            Rectangle { anchors.bottom: parent.bottom; width: parent.width; height: 1; color: Th.ac }
            MouseArea {
                id: dm
                anchors.fill: parent
                anchors.rightMargin: 28
                cursorShape: Qt.SizeAllCursor
                drag.target: px
                onPressed: U.hideTip()
                onReleased: px.Drag.drop()
            }
            Row {
                anchors.verticalCenter: parent.verticalCenter
                x: 8
                spacing: 6
                width: parent.width - 40
                Ico { n: "drag"; sz: 14; col: Th.tx2; anchors.verticalCenter: parent.verticalCenter }
                Lbl { width: parent.width - 24; text: r.nm; font.weight: Font.DemiBold; font.pixelSize: Th.fsS; anchors.verticalCenter: parent.verticalCenter }
            }
            IBtn {
                visible: r.canRm
                anchors.right: parent.right
                anchors.rightMargin: 2
                anchors.verticalCenter: parent.verticalCenter
                n: "x"
                sz: 24
                isz: 14
                onClicked: U.setLay(L.rm(S.lay, r.p).root)
            }
        }

        Column {
            visible: r.tiny
            anchors.centerIn: parent
            spacing: 4
            Rectangle {
                width: 28
                height: 28
                radius: Th.rs
                color: dm2.containsMouse ? Th.sel : Th.bg3
                border.color: Th.ac
                Ico { anchors.centerIn: parent; n: "drag"; sz: 14; col: Th.tx }
                MouseArea {
                    id: dm2
                    anchors.fill: parent
                    hoverEnabled: true
                    cursorShape: Qt.SizeAllCursor
                    drag.target: px
                    onReleased: px.Drag.drop()
                }
            }
            IBtn {
                visible: r.canRm
                n: "x"
                sz: 28
                isz: 14
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
        visible: dm.drag.active || dm2.drag.active
        z: 99
        Drag.active: dm.drag.active || dm2.drag.active
        Drag.keys: ["cord-blk"]
        Drag.source: px
        Drag.hotSpot.x: 60
        Drag.hotSpot.y: 16
        Rectangle {
            anchors.fill: parent
            radius: Th.rs
            color: Th.ac
            Lbl { anchors.centerIn: parent; width: parent.width - 8; horizontalAlignment: Text.AlignHCenter; text: r.nm; color: Th.acTx }
        }
        onVisibleChanged: if (!visible) { x = 0; y = 0 }
    }
}
