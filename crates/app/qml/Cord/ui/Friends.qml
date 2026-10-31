import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import Cord

Rectangle {
    id: r
    color: Th.bg2
    property string tab: "on"
    property string q: ""
    property var list: {
        void S.ver
        const ps = S.pres
        return S.rels.filter(x => {
            const id = x.usr.id ? S.ridOf(x.usr.id) : ""
            const p = ps[id]
            if (tab === "on") return x.k === "Friend" && p && p.st !== "Off"
            if (tab === "all") return x.k === "Friend"
            if (tab === "pend") return x.k === "In" || x.k === "Out"
            return x.k === "Block"
        }).filter(x => !q || (x.usr.disp || x.usr.name).toLowerCase().indexOf(q.toLowerCase()) >= 0)
          .sort((a, b) => (a.usr.disp || a.usr.name).localeCompare(b.usr.disp || b.usr.name))
    }
    ColumnLayout {
        anchors.fill: parent
        spacing: 0
        Rectangle {
            Layout.fillWidth: true
            Layout.preferredHeight: 48
            color: "transparent"
            Rectangle { anchors.bottom: parent.bottom; width: parent.width; height: 1; color: Th.ln }
            Row {
                anchors.verticalCenter: parent.verticalCenter
                x: 16
                spacing: 6
                Ico { n: "users"; col: Th.tx2; anchors.verticalCenter: parent.verticalCenter }
                Lbl { text: "Friends"; font.weight: Font.DemiBold; anchors.verticalCenter: parent.verticalCenter; rightPadding: 12 }
                Repeater {
                    model: [["on", "Online"], ["all", "All"], ["pend", "Pending"], ["blk", "Blocked"]]
                    Rectangle {
                        width: tl.implicitWidth + 20
                        height: 28
                        radius: Th.rs
                        color: r.tab === modelData[0] ? Th.sel : tm.containsMouse ? Th.hov : "transparent"
                        anchors.verticalCenter: parent.verticalCenter
                        Lbl { id: tl; anchors.centerIn: parent; text: modelData[1]; color: r.tab === modelData[0] ? Th.tx : Th.tx2 }
                        MouseArea { id: tm; anchors.fill: parent; hoverEnabled: true; cursorShape: Qt.PointingHandCursor; onClicked: r.tab = modelData[0] }
                    }
                }
            }
        }
        Fld {
            Layout.fillWidth: true
            Layout.margins: 16
            Layout.bottomMargin: 8
            placeholderText: "Search"
            onTextChanged: r.q = text
        }
        Lbl {
            Layout.leftMargin: 20
            text: ({ on: "ONLINE", all: "ALL FRIENDS", pend: "PENDING", blk: "BLOCKED" })[r.tab] + "  ·  " + r.list.length
            color: Th.tx3
            font.pixelSize: 11
            font.weight: Font.DemiBold
        }
        ListView {
            id: lv
            Layout.fillWidth: true
            Layout.fillHeight: true
            Layout.topMargin: 8
            clip: true
            model: r.list
            ScrollBar.vertical: Scr {}
            delegate: Rectangle {
                property string uid: modelData.usr.id ? S.ridOf(modelData.usr.id) : ""
                property var p: S.pres[uid] || null
                x: 12
                width: lv.width - 24
                height: 60
                radius: Th.rs
                color: ma.containsMouse ? Th.hov : "transparent"
                Rectangle { width: parent.width - 16; x: 8; height: 1; color: Th.ln; visible: index > 0 }
                Av {
                    id: av
                    x: 8
                    anchors.verticalCenter: parent.verticalCenter
                    sz: 36
                    src: modelData.usr.av || ""
                    name: modelData.usr.name
                    st: p ? p.st : "Off"
                    sv: S.svc(uid)
                }
                Column {
                    anchors.left: av.right
                    anchors.leftMargin: 12
                    anchors.verticalCenter: parent.verticalCenter
                    Row {
                        spacing: 6
                        Lbl { text: modelData.usr.disp || modelData.usr.name; font.weight: Font.DemiBold }
                        Lbl { text: modelData.usr.name; color: Th.tx3; visible: ma.containsMouse }
                    }
                    Lbl {
                        text: modelData.k === "In" ? "Incoming request" : modelData.k === "Out" ? "Outgoing request" : p && p.act ? p.act : p ? ({ On: "Online", Idle: "Idle", Dnd: "Do not disturb", Off: "Offline" })[p.st] : "Offline"
                        color: Th.tx3
                        font.pixelSize: Th.fsS
                    }
                }
                Row {
                    anchors.right: parent.right
                    anchors.rightMargin: 8
                    anchors.verticalCenter: parent.verticalCenter
                    spacing: 6
                    IBtn {
                        n: "thread"
                        tip: "Message"
                        visible: modelData.k === "Friend"
                        onClicked: {
                            const d = S.dms().find(c => c.k === "Dm" && c.rcp && c.rcp[0] && c.rcp[0].id && S.ridOf(c.rcp[0].id) === uid)
                            if (d) S.open(d.rid)
                        }
                    }
                    IBtn { n: "check"; tip: "Accept"; visible: modelData.k === "In"; onClicked: S.c.rel(uid, "accept") }
                    IBtn { n: "x"; tip: "Remove"; red: modelData.k !== "Friend"; onClicked: S.c.rel(uid, "remove") }
                }
                MouseArea { id: ma; anchors.fill: parent; hoverEnabled: true; z: -1; onClicked: S.prof(modelData.usr, av) }
            }
            Lbl {
                anchors.centerIn: parent
                visible: r.list.length === 0
                text: r.tab === "on" ? "Nobody is online right now" : "Nothing here"
                color: Th.tx3
            }
        }
    }
}
