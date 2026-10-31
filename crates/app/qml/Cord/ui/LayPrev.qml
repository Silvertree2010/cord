import QtQuick
import Cord

Item {
    id: r
    property var n: ({})
    Loader {
        anchors.fill: parent
        sourceComponent: r.n.b ? leaf : box
    }
    Component {
        id: leaf
        Rectangle {
            radius: 3
            color: r.n.b === "chat" || r.n.b === "stage" ? Th.bg3 : r.n.b === "rail" ? Th.bg4 : Th.bg2
            border.color: Th.ln
            Rectangle {
                visible: r.n.b === "chat"
                anchors.bottom: parent.bottom
                anchors.margins: 4
                x: 4
                width: parent.width - 8
                height: 5
                radius: 2
                color: Th.bg4
            }
            Column {
                visible: r.n.b === "side" || r.n.b === "mems" || r.n.b === "inbox"
                x: 4
                y: 6
                spacing: 4
                Repeater {
                    model: 4
                    Rectangle { width: Math.max(4, r.width - 10 - (index % 2) * 6); height: 3; radius: 1.5; color: Th.tx3; opacity: 0.4 }
                }
            }
            Column {
                visible: r.n.b === "rail"
                anchors.horizontalCenter: parent.horizontalCenter
                y: 4
                spacing: 3
                Repeater {
                    model: 4
                    Rectangle { width: Math.min(r.width, r.height) - 6; height: width; radius: width / 2; color: Th.tx3; opacity: 0.4 }
                }
            }
        }
    }
    Component {
        id: box
        Item {
            Row {
                visible: r.n.t === "row"
                anchors.fill: parent
                spacing: 2
                Repeater {
                    model: r.n.t === "row" ? r.n.ch : []
                    LayPrev {
                        n: modelData
                        height: parent.height
                        width: {
                            const ch = r.n.ch
                            const fixed = ch.filter(c => !c.grow).reduce((a, c) => a + (c.w || 200), 0)
                            const g = ch.filter(c => c.grow).length || 1
                            const tot = fixed * 1.6 + g * 600
                            const k = (r.width - (ch.length - 1) * 2) / tot
                            return modelData.grow ? 600 * k : (modelData.w || 200) * 1.6 * k
                        }
                    }
                }
            }
            Column {
                visible: r.n.t !== "row"
                anchors.fill: parent
                spacing: 2
                Repeater {
                    model: r.n.t !== "row" ? r.n.ch : []
                    LayPrev {
                        n: modelData
                        width: parent.width
                        height: modelData.grow ? Math.max(10, r.height - (r.n.ch.length - 1) * 14) : 12
                    }
                }
            }
        }
    }
}
