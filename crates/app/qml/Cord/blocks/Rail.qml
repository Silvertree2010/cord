import QtQuick
import QtQuick.Controls
import Cord

Rectangle {
    id: r
    property var o: ({})
    property bool hz: !!o.horiz
    implicitWidth: hz ? 300 : 72
    implicitHeight: hz ? 56 : 300
    color: Th.bg
    readonly property real isz: Math.min(48, (hz ? height : width) - 22)

    ListView {
        id: lv
        anchors.fill: parent
        anchors.margins: 0
        orientation: r.hz ? ListView.Horizontal : ListView.Vertical
        spacing: 6
        clip: true
        header: Item {
            width: r.hz ? hc.implicitWidth + 12 : lv.width
            height: r.hz ? lv.height : hc.implicitHeight + 12
            Flow {
                id: hc
                flow: r.hz ? Flow.LeftToRight : Flow.TopToBottom
                spacing: 6
                x: r.hz ? 6 : (parent.width - r.isz) / 2
                y: r.hz ? (parent.height - r.isz) / 2 : 10
                Ic {
                    gid: "home"
                    name: "Home"
                    ico: "home"
                    unrN: S.dmUnr().length
                }
                Repeater {
                    model: S.dmUnr()
                    Ic {
                        gid: "dm:" + modelData.rid
                        name: S.chName(modelData.rid)
                        img: modelData.rcp && modelData.rcp.length ? (modelData.rcp[0].av || "") : (modelData.icon || "")
                        sv: S.svc(modelData.rid)
                        ment: (S.unr[modelData.rid] || {}).ment || 1
                        round: true
                        onPick: { S.sel = "home"; S.open(modelData.rid) }
                    }
                }
                Rectangle {
                    width: r.hz ? 2 : r.isz * 0.6
                    height: r.hz ? r.isz * 0.6 : 2
                    radius: 1
                    color: Th.ln
                    x: r.hz ? 0 : r.isz * 0.2
                    y: r.hz ? r.isz * 0.2 : 0
                }
            }
        }
        model: S.glds
        delegate: Item {
            width: r.hz ? r.isz + 12 : lv.width
            height: r.hz ? lv.height : r.isz
            Ic {
                x: r.hz ? 6 : (parent.width - r.isz) / 2
                y: r.hz ? (parent.height - r.isz) / 2 : 0
                gid: modelData.rid
                name: modelData.name
                img: modelData.icon || ""
                sv: S.svc(modelData.rid)
                property var u: S.gUnr(modelData.rid)
                unrN: u.n
                ment: u.m
            }
        }
        footer: Item {
            width: r.hz ? r.isz + 24 : lv.width
            height: r.hz ? lv.height : r.isz + 24
            Ic {
                x: r.hz ? 12 : (parent.width - r.isz) / 2
                y: r.hz ? (parent.height - r.isz) / 2 : 12
                gid: "add"
                name: "Add account"
                ico: "plus"
                onPick: { S.setPage = "accounts"; S.showSet = true }
            }
        }
        ScrollBar.vertical: Scr {}
    }

    component Ic: Item {
        id: ic
        property string gid
        property string name
        property string img: ""
        property string ico: ""
        property string sv: ""
        property int unrN: 0
        property int ment: 0
        property bool round: false
        property bool cur: S.sel === gid
        signal pick
        width: r.isz
        height: r.isz
        onPick: S.pick(gid)
        Rectangle {
            visible: !r.hz && (ic.cur || ic.unrN > 0 || ma.containsMouse)
            width: 4
            height: ic.cur ? r.isz * 0.8 : ma.containsMouse ? r.isz * 0.4 : 8
            radius: 2
            color: Th.tx
            x: -((r.width - r.isz) / 2)
            anchors.verticalCenter: parent.verticalCenter
            Behavior on height { NumberAnimation { duration: S.f("anim") ? 120 : 0 } }
        }
        Rectangle {
            anchors.fill: parent
            radius: ic.cur || ma.containsMouse || ic.round ? Th.r * 1.4 : width / 2
            color: ic.cur ? Th.ac : ma.containsMouse ? Th.bg4 : Th.bg3
            visible: ic.ico !== "" || ic.img === ""
            Behavior on radius { NumberAnimation { duration: S.f("anim") ? 120 : 0 } }
            Ico {
                visible: ic.ico !== ""
                anchors.centerIn: parent
                n: ic.ico || "home"
                sz: 22
                col: ic.cur ? Th.acTx : Th.tx
            }
            Lbl {
                visible: ic.ico === ""
                anchors.centerIn: parent
                width: parent.width - 6
                horizontalAlignment: Text.AlignHCenter
                text: ic.name.split(/\s+/).map(w => w.charAt(0)).join("").slice(0, 3)
                color: ic.cur ? Th.acTx : Th.tx
                font.pixelSize: Th.fs - 1
                font.weight: Font.DemiBold
            }
        }
        Av {
            visible: ic.img !== "" && ic.ico === ""
            anchors.fill: parent
            sz: r.isz
            src: ic.img
            name: ic.name
            sq: !ic.round
        }
        Badge {
            visible: ic.sv !== "" && S.ui.badges !== false
            sv: ic.sv
            sz: 16
            ring: Th.bg
            anchors.left: parent.left
            anchors.bottom: parent.bottom
            anchors.margins: -3
        }
        Rectangle {
            visible: ic.ment > 0
            height: 18
            width: Math.max(18, mt.implicitWidth + 8)
            radius: 9
            color: Th.err
            border.color: Th.bg
            border.width: 3
            anchors.right: parent.right
            anchors.bottom: parent.bottom
            anchors.margins: -4
            Lbl {
                id: mt
                anchors.centerIn: parent
                text: ic.ment > 99 ? "99+" : ic.ment
                color: "#fff"
                font.pixelSize: 11
                font.weight: Font.Bold
            }
        }
        MouseArea {
            id: ma
            anchors.fill: parent
            hoverEnabled: true
            cursorShape: Qt.PointingHandCursor
            acceptedButtons: Qt.LeftButton | Qt.RightButton
            onClicked: m => {
                if (m.button === Qt.RightButton && ic.gid.indexOf(":") > 0) cm.popup()
                else ic.pick()
            }
        }
        ToolTip.visible: ma.containsMouse
        ToolTip.text: ic.name + (ic.sv ? "  ·  " + Th.svcName(ic.sv) : "")
        ToolTip.delay: 300
        Menu {
            id: cm
            MenuItem {
                text: "Mark as read"
                onTriggered: { for (const c of S.chList(ic.gid)) if (S.unr[c.rid]) S.c.ack(c.rid) }
            }
            MenuItem {
                text: "Move up"
                onTriggered: ic.mv(-1)
            }
            MenuItem {
                text: "Move down"
                onTriggered: ic.mv(1)
            }
            MenuItem {
                text: "Copy ID"
                onTriggered: S.copy(S.raw(ic.gid))
            }
        }
        function mv(d) {
            const o = S.glds.map(g => g.rid)
            const i = o.indexOf(gid)
            const j = i + d
            if (j < 0 || j >= o.length) return
            o.splice(i, 1)
            o.splice(j, 0, gid)
            S.setUi("gorder", o)
            S.refresh()
        }
    }
}
