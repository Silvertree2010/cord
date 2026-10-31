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
    readonly property real fz: r.hz ? r.isz + 24 : r.isz + 20

    ListView {
        id: lv
        anchors.left: parent.left
        anchors.top: parent.top
        anchors.right: r.hz ? gear.left : parent.right
        anchors.bottom: r.hz ? parent.bottom : gear.top
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
                    dotN: S.dmUnr().length
                }
                Repeater {
                    model: S.dmUnr()
                    Ic {
                        gid: "dm:" + modelData.rid
                        name: S.chName(modelData.rid)
                        img: modelData.rcp && modelData.rcp.length ? (modelData.rcp[0].av || "") : (modelData.icon || "")
                        sv: S.svc(modelData.rid)
                        ment: (S.unr[modelData.rid] || {}).ment || 0
                        dotN: 1
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
                dotN: u.n
                ment: u.m
            }
        }
        footer: Item {
            width: r.hz ? r.isz + 24 : lv.width
            height: r.hz ? lv.height : r.isz + 24
            Ic {
                id: addIc
                x: r.hz ? 12 : (parent.width - r.isz) / 2
                y: r.hz ? (parent.height - r.isz) / 2 : 12
                gid: "add"
                name: "Add"
                ico: "plus"
                onPick: r.hz ? addM.show(addIc, 0, addIc.height + 4) : addM.show(addIc, addIc.width + 4, 0)
            }
        }
        ScrollBar.vertical: Scr {}
    }

    Item {
        id: gear
        anchors.right: parent.right
        anchors.bottom: parent.bottom
        width: r.hz ? r.fz : parent.width
        height: r.hz ? parent.height : r.fz
        Rectangle {
            visible: !r.hz
            width: parent.width * 0.5
            height: 1
            color: Th.ln
            anchors.horizontalCenter: parent.horizontalCenter
            y: 0
        }
        Ic {
            x: (parent.width - r.isz) / 2
            y: (parent.height - r.isz) / 2
            gid: "cog"
            name: "Settings  (Ctrl+,)"
            ico: "cog"
            onPick: S.showSet = true
        }
    }

    readonly property bool canJoin: typeof S.c.join_invite === "function"
    Ctx {
        id: addM
        model: [
            { text: "Add account", run: () => { S.setPage = "accounts"; S.showSet = true } },
            { text: "Join server", when: r.canJoin, run: () => joinP.show() }
        ]
    }
    Popup {
        id: joinP
        parent: Overlay.overlay
        anchors.centerIn: Overlay.overlay
        modal: true
        width: 400
        padding: 20
        closePolicy: Popup.CloseOnEscape | Popup.CloseOnPressOutside
        onAboutToShow: U.reg(joinP)
        onOpened: jf.forceActiveFocus()
        onClosed: U.unreg(joinP)
        Overlay.modal: Rectangle { color: "#80000000" }
        function show() { jf.text = ""; open() }
        function go() {
            if (!jf.text.trim()) return
            S.c.join_invite(jf.text.trim())
            close()
        }
        background: Rectangle { color: Th.bg3; radius: Th.r; border.color: Th.ln }
        contentItem: Column {
            spacing: 12
            Lbl { width: parent.width; text: "Join a Discord server"; font.pixelSize: Th.fsL; font.weight: Font.DemiBold }
            Fld { id: jf; width: parent.width; placeholderText: "Invite link or code"; onAccepted: joinP.go() }
            Row {
                anchors.right: parent.right
                spacing: 8
                Btn { text: "Cancel"; out: true; onClicked: joinP.close() }
                Btn { text: "Join"; pri: true; on: jf.text.trim() !== ""; onClicked: joinP.go() }
            }
        }
    }

    component Ic: Item {
        id: ic
        property string gid
        property string name
        property string img: ""
        property string ico: ""
        property string sv: ""
        property int dotN: 0
        property int ment: 0
        property bool cur: S.sel === gid
        signal pick
        width: r.isz
        height: r.isz
        activeFocusOnTab: true
        Accessible.role: Accessible.Button
        Accessible.name: ic.name
        Keys.onReturnPressed: ic.pick()
        Keys.onSpacePressed: ic.pick()
        onPick: S.pick(gid)
        Rectangle {
            visible: !r.hz && ic.cur
            width: 3
            height: r.isz
            color: Th.ac
            x: -((r.width - r.isz) / 2)
            anchors.verticalCenter: parent.verticalCenter
        }
        Rectangle {
            visible: r.hz && ic.cur
            height: 3
            width: r.isz
            color: Th.ac
            y: parent.height + 3
            anchors.horizontalCenter: parent.horizontalCenter
        }
        Rectangle {
            anchors.fill: parent
            radius: Th.r * 1.4
            color: ic.cur || ma.containsMouse ? Th.bg4 : Th.bg3
            visible: ic.ico !== "" || ic.img === ""
            Ico {
                visible: ic.ico !== ""
                anchors.centerIn: parent
                n: ic.ico || "home"
                sz: 22
                col: ic.cur ? Th.ac : Th.tx
            }
            Lbl {
                visible: ic.ico === ""
                anchors.centerIn: parent
                width: parent.width - 6
                horizontalAlignment: Text.AlignHCenter
                text: ic.name.split(/\s+/).map(w => w.charAt(0)).join("").slice(0, 3)
                color: Th.tx
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
            sq: true
        }
        Rectangle {
            anchors.fill: parent
            radius: Th.r * 1.4
            color: "transparent"
            border.color: Th.ac
            border.width: 2
            visible: ic.activeFocus
        }
        Badge {
            visible: ic.sv !== "" && S.ui.badges !== false
            sv: ic.sv
            sz: 14
            rw: 2
            ring: Th.bg
            anchors.left: parent.left
            anchors.bottom: parent.bottom
            anchors.leftMargin: -3
            anchors.bottomMargin: -3
        }
        Rectangle {
            visible: ic.ment > 0
            height: 16
            width: Math.max(16, mt.implicitWidth + 8)
            radius: 8
            color: Th.mr
            border.color: Th.bg
            border.width: 2
            anchors.right: parent.right
            anchors.top: parent.top
            anchors.margins: -5
            Lbl {
                id: mt
                anchors.centerIn: parent
                text: ic.ment > 99 ? "99+" : ic.ment
                color: "#fff"
                font.pixelSize: 10
                font.weight: Font.Bold
            }
        }
        Rectangle {
            visible: ic.ment === 0 && ic.dotN > 0 && !ic.cur
            width: 8
            height: 8
            radius: 4
            color: Th.tx
            anchors.right: parent.right
            anchors.top: parent.top
            anchors.margins: -2
        }
        MouseArea {
            id: ma
            anchors.fill: parent
            hoverEnabled: true
            cursorShape: Qt.PointingHandCursor
            acceptedButtons: Qt.LeftButton | Qt.RightButton
            onPressed: tp.cancel()
            onClicked: m => {
                if (m.button === Qt.RightButton && ic.gid.indexOf(":") > 0 && ic.gid.indexOf("dm:") !== 0) cm.show(ma, m.x, m.y)
                else ic.pick()
            }
        }
        Tip { id: tp; text: ic.name + (ic.sv ? "  ·  " + Th.svcName(ic.sv) : ""); hov: ma.containsMouse; side: r.hz ? "b" : "r" }
        Ctx {
            id: cm
            model: [
                { text: "Mark as read", run: () => { for (const c of S.chList(ic.gid)) if (S.unr[c.rid]) S.c.ack(c.rid) } },
                { text: "Move up", run: () => ic.mv(-1) },
                { text: "Move down", run: () => ic.mv(1) },
                { sep: true },
                { text: "Copy server ID", run: () => S.copy(S.raw(ic.gid)) }
            ]
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
