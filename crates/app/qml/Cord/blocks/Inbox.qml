import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import Cord

Rectangle {
    id: r
    property var o: ({})
    implicitWidth: 300
    implicitHeight: 400
    color: Th.bgc
    property var items: {
        void S.uver
        return Object.keys(S.unr).filter(k => S.chans[k] && (S.unr[k].ment > 0 || S.chans[k].k === "Dm" || S.chans[k].k === "Grp"))
            .map(k => ({ c: S.chans[k], u: S.unr[k] }))
            .sort((a, b) => b.u.ment - a.u.ment)
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
                x: 16
                anchors.verticalCenter: parent.verticalCenter
                spacing: 8
                Ico { n: "inbox"; col: Th.tx2; anchors.verticalCenter: parent.verticalCenter }
                Lbl { text: "Inbox"; font.weight: Font.DemiBold; anchors.verticalCenter: parent.verticalCenter }
            }
            Btn {
                visible: S.f("readall")
                anchors.right: parent.right
                anchors.rightMargin: 8
                anchors.verticalCenter: parent.verticalCenter
                text: "Read all"
                flat: true
                height: 28
                onClicked: S.readAll()
            }
        }
        ListView {
            Smooth {}
            id: lv
            Layout.fillWidth: true
            Layout.fillHeight: true
            clip: true
            model: r.items
            ScrollBar.vertical: Scr {}
            delegate: Item {
                width: lv.width
                height: 56
                Rectangle {
                width: Math.min(lv.width - 24, 760)
                x: (lv.width - width) / 2
                height: 56
                radius: Th.rs
                color: ma.containsMouse ? Th.hov : "transparent"
                Behavior on color { ColorAnimation { duration: Th.mF } }
                Av {
                    id: av
                    x: 12
                    anchors.verticalCenter: parent.verticalCenter
                    sz: 32
                    sq: !!modelData.c.grid
                    rad: modelData.c.grid ? sz * 0.28 : -1
                    src: modelData.c.grid && S.gmap[modelData.c.grid] ? (S.gmap[modelData.c.grid].icon || "") : modelData.c.rcp && modelData.c.rcp[0] ? (modelData.c.rcp[0].av || "") : ""
                    name: modelData.c.grid && S.gmap[modelData.c.grid] ? S.gmap[modelData.c.grid].name : S.chName(modelData.c.rid)
                    sv: S.svc(modelData.c.rid)
                }
                Column {
                    anchors.left: av.right
                    anchors.leftMargin: 10
                    anchors.right: parent.right
                    anchors.rightMargin: 40
                    anchors.verticalCenter: parent.verticalCenter
                    Lbl { width: parent.width; text: (modelData.c.grid ? "#" : "@") + S.chName(modelData.c.rid); font.weight: Font.DemiBold }
                    Lbl { width: parent.width; text: modelData.c.grid && S.gmap[modelData.c.grid] ? S.gmap[modelData.c.grid].name : "Direct message"; color: Th.tx3; font.pixelSize: Th.fsS - 1 }
                }
                Rectangle {
                    visible: modelData.u.ment > 0
                    anchors.right: parent.right
                    anchors.rightMargin: 12
                    anchors.verticalCenter: parent.verticalCenter
                    height: 18
                    width: Math.max(18, ibn.implicitWidth + 10)
                    radius: 9
                    color: Th.mr
                    Lbl { id: ibn; anchors.centerIn: parent; text: modelData.u.ment > 99 ? "99+" : modelData.u.ment; color: Th.wh; font.pixelSize: Th.fsT; font.weight: Font.Bold }
                }
                Rectangle {
                    visible: modelData.u.ment === 0 && modelData.u.n > 0
                    anchors.right: parent.right
                    anchors.rightMargin: 18
                    anchors.verticalCenter: parent.verticalCenter
                    width: 8
                    height: 8
                    radius: 4
                    color: Th.tx2
                }
                MouseArea {
                    id: ma
                    anchors.fill: parent
                    hoverEnabled: true
                    cursorShape: Qt.PointingHandCursor
                    onClicked: { S.sel = modelData.c.grid || "home"; S.open(modelData.c.rid, S.hasBlk("inbox") && S.lay && JSON.stringify(S.lay).indexOf('"slot":1') >= 0 ? 1 : 0) }
                }
            }
            }
            Column {
                anchors.centerIn: parent
                visible: r.items.length === 0
                spacing: 8
                Ico { anchors.horizontalCenter: parent.horizontalCenter; n: "inbox"; sz: 28; col: Th.tx3 }
                Lbl { text: "No unread mentions or messages"; color: Th.tx3 }
            }
        }
    }
}
