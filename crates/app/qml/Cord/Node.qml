import QtQuick
import QtQuick.Layouts
import Cord

Item {
    id: r
    property var n: ({})
    property var p: []
    property bool row: n.t === "row"
    implicitWidth: ld.item ? ld.item.implicitWidth : 0
    implicitHeight: ld.item ? ld.item.implicitHeight : 0
    property bool shown: ld.item ? ld.item.shown !== false : true
    visible: shown

    Loader {
        id: ld
        anchors.fill: parent
        sourceComponent: r.n.b ? blk : r.n.t === "tabs" ? tabs : r.row ? rowC : colC
    }

    Component {
        id: blk
        Blk { n: r.n; p: r.p }
    }

    Component {
        id: rowC
        RowLayout {
            id: rl
            spacing: 0
            property int lastV: -1
            function recalc() {
                let k = -1, g = 0
                for (let i = 0; i < rp.count; i++) {
                    const it = rp.itemAt(i)
                    if (!it || !it.visible) continue
                    k = i
                    if (it.gr) g++
                }
                lastV = g > 0 ? -1 : k
            }
            Repeater {
                id: rp
                model: r.n.ch || []
                delegate: Loader {
                    required property var modelData
                    required property int index
                    property var p: r.p.concat([index])
                    property bool rail: modelData.b === "rail" && !(modelData.o && modelData.o.horiz)
                    property bool gr: !!modelData.grow
                    Component.onCompleted: { setSource(Qt.resolvedUrl("Node.qml"), { n: modelData, p: p }); Qt.callLater(rl.recalc) }
                    visible: item ? item.shown !== false : true
                    onVisibleChanged: Qt.callLater(rl.recalc)
                    Layout.fillHeight: true
                    Layout.fillWidth: !!modelData.grow || index === rl.lastV
                    Layout.preferredWidth: rail ? (S.edit ? 68 : 64 + Th.gap) : modelData.grow ? 200 : (modelData.w || (item ? item.implicitWidth : 0))
                    Layout.minimumWidth: modelData.grow ? 240 : 0
                    Rectangle {
                        visible: S.edit && index > 0 && parent.visible && parent.width > 0
                        width: 1
                        height: parent.height
                        color: Th.ln
                        z: 5
                    }
                    MouseArea {
                        visible: !modelData.grow && !!modelData.w && !parent.rail
                        width: 6
                        height: parent.height
                        x: (r.n.ch.findIndex(c => c.grow) > index) ? parent.width - 3 : -3
                        z: 6
                        cursorShape: Qt.SplitHCursor
                        property real sx: 0
                        property real sw: 0
                        onPressed: m => { if (S.edit) U.push(); sx = mapToGlobal(m.x, 0).x; sw = parent.width }
                        onPositionChanged: m => {
                            if (!pressed) return
                            const dx = mapToGlobal(m.x, 0).x - sx
                            const right = x > 0
                            parent.Layout.preferredWidth = Math.max(48, Math.min(720, sw + (right ? dx : -dx)))
                        }
                        onReleased: S.setW(parent.p, Math.round(parent.Layout.preferredWidth), "w")
                    }
                }
            }
        }
    }

    Component {
        id: colC
        ColumnLayout {
            spacing: 0
            Repeater {
                model: r.n.ch || []
                delegate: Loader {
                    required property var modelData
                    required property int index
                    property var p: r.p.concat([index])
                    Component.onCompleted: setSource(Qt.resolvedUrl("Node.qml"), { n: modelData, p: p })
                    visible: item ? item.shown !== false : true
                    Layout.fillWidth: true
                    Layout.fillHeight: !!modelData.grow
                    Layout.preferredHeight: modelData.grow ? 100 : (modelData.h || (item ? item.implicitHeight : 0))
                    Rectangle {
                        visible: S.edit && index > 0 && parent.visible && parent.height > 0
                        height: 1
                        width: parent.width
                        color: Th.ln
                        z: 5
                    }
                }
            }
        }
    }

    Component {
        id: tabs
        ColumnLayout {
            spacing: 0
            property int cur: 0
            Row {
                Layout.fillWidth: true
                Layout.preferredHeight: 34
                Repeater {
                    model: r.n.ch || []
                    Rectangle {
                        width: tl.implicitWidth + 24
                        height: 34
                        color: parent.parent.cur === index ? Th.bg2 : Th.bg
                        Lbl {
                            id: tl
                            anchors.centerIn: parent
                            text: (S.allBlks().find(b => b.id === modelData.b) || { name: modelData.b || "Group" }).name
                            color: parent.parent.parent.cur === index ? Th.tx : Th.tx3
                        }
                        MouseArea { anchors.fill: parent; onClicked: parent.parent.parent.cur = index }
                    }
                }
            }
            StackLayout {
                Layout.fillWidth: true
                Layout.fillHeight: true
                currentIndex: parent.cur
                Repeater {
                    model: r.n.ch || []
                    Loader {
                        required property var modelData
                        required property int index
                        Component.onCompleted: setSource(Qt.resolvedUrl("Node.qml"), { n: modelData, p: r.p.concat([index]) })
                    }
                }
            }
        }
    }
}
