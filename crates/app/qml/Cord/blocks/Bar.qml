import QtQuick
import QtQuick.Layouts
import QtQuick.Window
import QtQuick.Controls
import Cord

Rectangle {
    id: r
    property var o: ({})
    implicitWidth: 600
    implicitHeight: 48
    color: Th.bgc
    readonly property string cid: S.selC
    readonly property var ch: S.chans[cid] || null
    readonly property bool dmish: !!r.ch && (r.ch.k === "Dm" || r.ch.k === "Grp")
    readonly property string title: !cid ? "Friends" : cid === "inbox" ? "Inbox" : r.ch && r.ch.k !== "Dm" && r.ch.k !== "Grp" ? "#" + S.chName(cid) : S.chName(cid)

    MouseArea {
        anchors.fill: parent
        acceptedButtons: Qt.LeftButton
        onPressed: if (r.Window.window) r.Window.window.startSystemMove()
        onDoubleClicked: r.max()
    }

    function max() {
        const w = r.Window.window
        if (!w) return
        if (w.visibility === Window.Maximized) w.showNormal(); else w.showMaximized()
    }

    RowLayout {
        anchors.fill: parent
        anchors.leftMargin: 16
        anchors.rightMargin: 14
        spacing: 6
        Lbl {
            text: r.title
            font.pixelSize: Th.fs
            font.weight: Font.DemiBold
            Layout.maximumWidth: 320
        }
        Lbl {
            Layout.fillWidth: true
            Layout.leftMargin: 8
            visible: !!(r.ch && r.ch.topic)
            text: r.ch && r.ch.topic ? S.plain(r.ch.topic) : ""
            color: Th.tx2
            font.pixelSize: Th.fsS
        }
        Item { Layout.fillWidth: !(r.ch && r.ch.topic) }
        Rectangle {
            Layout.preferredWidth: 244
            Layout.preferredHeight: 32
            Layout.rightMargin: 6
            radius: 6
            color: Th.bg
            border.color: sq.activeFocus ? Th.a(Th.ac, 0.6) : "transparent"
            TextInput {
                id: sq
                x: 10
                width: parent.width - 40
                anchors.verticalCenter: parent.verticalCenter
                color: Th.tx
                font.family: Th.font
                font.pixelSize: Th.fsS + 1
                clip: true
                selectByMouse: true
                onAccepted: if (text.trim()) srch.run(text.trim(), r.cid)
                Keys.onEscapePressed: { text = ""; focus = false }
                Lbl {
                    visible: !sq.text
                    anchors.verticalCenter: parent.verticalCenter
                    width: parent.width
                    text: r.ch && r.ch.gld ? "Search " + (S.gmap[r.ch.grid || S.sel] ? S.gmap[r.ch.grid || S.sel].name : "") : r.ch ? "Search" : "Search cord"
                    color: Th.tx2
                    font.pixelSize: Th.fsS + 1
                }
            }
            Ico { anchors.right: parent.right; anchors.rightMargin: 10; anchors.verticalCenter: parent.verticalCenter; n: sq.text ? "x" : "search"; sz: 16; col: Th.tx2; MouseArea { anchors.fill: parent; anchors.margins: -6; cursorShape: Qt.PointingHandCursor; onClicked: { if (sq.text) sq.text = ""; else sq.forceActiveFocus() } } }
            MouseArea { anchors.fill: parent; anchors.rightMargin: 30; cursorShape: Qt.IBeamCursor; visible: !r.ch; onClicked: S.quick() }
        }
        IBtn { visible: r.dmish; n: "phone"; tip: "Start call"; onClicked: S.c.join(r.cid) }
        IBtn { visible: !!r.ch; n: "pin"; tip: "Pinned messages"; onClicked: S.pinReq++ }
        IBtn { visible: !!r.ch && r.ch.k !== "Dm"; n: "users"; tip: "Member list"; act: S.hasBlk("mems"); onClicked: S.toggleBlk("mems", !S.hasBlk("mems")) }
        IBtn { n: "inbox"; tip: "Inbox"; act: S.selC === "inbox"; onClicked: S.hasBlk("inbox") ? S.toggleBlk("inbox", false) : S.open("inbox") }
        Rectangle { Layout.preferredWidth: 1; Layout.preferredHeight: 20; Layout.leftMargin: 6; Layout.rightMargin: 8; color: Th.ln }
        Dot { col: "#f2c9da"; tip: "Minimize"; onHit: if (r.Window.window) r.Window.window.showMinimized() }
        Dot { col: "#a8cdf4"; tip: "Maximize"; onHit: r.max() }
        Dot { col: "#ec5e73"; tip: "Close"; onHit: if (r.Window.window) r.Window.window.close() }
    }

    component Dot: Item {
        id: d
        property color col
        property string tip
        signal hit
        Layout.preferredWidth: 28
        Layout.preferredHeight: 28
        Rectangle {
            anchors.centerIn: parent
            width: 13
            height: 13
            radius: 6.5
            color: dm.containsMouse ? Qt.lighter(d.col, 1.12) : d.col
        }
        MouseArea { id: dm; anchors.fill: parent; hoverEnabled: true; cursorShape: Qt.PointingHandCursor; onClicked: d.hit() }
        Tip { text: d.tip; hov: dm.containsMouse; side: "b" }
    }
    Search {
        id: srch
        parent: Overlay.overlay
        x: (parent ? parent.width : 1200) - width - 24
        y: 70
    }
}
