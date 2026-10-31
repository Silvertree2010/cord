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
        color: Th.a(Th.bg, 0.8)
        border.color: dz.containsDrag ? Th.ac : Th.ln
        border.width: dz.containsDrag ? 2 : 1
        z: 20
        Column {
            anchors.centerIn: parent
            spacing: 8
            Lbl {
                anchors.horizontalCenter: parent.horizontalCenter
                width: Math.min(implicitWidth, r.width - 8)
                horizontalAlignment: Text.AlignHCenter
                text: (S.allBlks().find(b => b.id === r.n.b) || { name: r.n.b }).name
                font.weight: Font.DemiBold
                visible: r.width > 60
            }
            Flow {
                anchors.horizontalCenter: parent.horizontalCenter
                width: Math.min(68, r.width - 4)
                spacing: 4
                Rectangle {
                    width: 32
                    height: 32
                    radius: Th.rs
                    color: dm.containsMouse ? Th.sel : Th.bg3
                    Ico { anchors.centerIn: parent; n: "drag"; col: Th.tx }
                    MouseArea {
                        id: dm
                        anchors.fill: parent
                        hoverEnabled: true
                        cursorShape: Qt.SizeAllCursor
                        drag.target: px
                        onReleased: px.Drag.drop()
                    }
                }
                IBtn { n: "x"; tip: "Remove block"; onClicked: S.setLay(L.rm(S.lay, r.p).root) }
            }
        }
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
                S.setLay(L.move(S.lay, from, r.p, hl.zn))
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
            color: Th.ac
            Lbl { anchors.centerIn: parent; text: r.n.b; color: Th.acTx }
        }
        onVisibleChanged: if (!visible) { x = 0; y = 0 }
    }
}
