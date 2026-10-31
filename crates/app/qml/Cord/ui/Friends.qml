import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import Cord

Rectangle {
    id: r
    color: Th.bg2
    property string tab: "on"
    property string q: ""
    readonly property var rank: ({ On: 0, Idle: 1, Dnd: 2, Off: 3 })
    property var list: {
        void S.ver
        const ps = S.pres
        const f = S.rels.filter(x => {
            const id = x.usr.id ? S.ridOf(x.usr.id) : ""
            const p = ps[id]
            if (tab === "on") return x.k === "Friend" && p && p.st !== "Off"
            if (tab === "all") return x.k === "Friend"
            if (tab === "pend") return x.k === "In" || x.k === "Out"
            return x.k === "Block"
        }).filter(x => !q || (x.usr.disp || x.usr.name).toLowerCase().indexOf(q.toLowerCase()) >= 0)
        const g = {}
        const out = []
        for (const x of f) {
            const k = (x.usr.disp || x.usr.name).toLowerCase() + "|" + x.k
            if (!g[k]) {
                g[k] = { name: x.usr.disp || x.usr.name, items: [] }
                out.push(g[k])
            }
            g[k].items.push(x)
        }
        for (const e of out) e.items.sort((a, b) => {
            const pa = ps[a.usr.id ? S.ridOf(a.usr.id) : ""], pb = ps[b.usr.id ? S.ridOf(b.usr.id) : ""]
            return (r.rank[pa ? pa.st : "Off"] || 0) - (r.rank[pb ? pb.st : "Off"] || 0)
        })
        return out.sort((a, b) => a.name.localeCompare(b.name))
    }
    function dmOf(uid) {
        return S.dms().find(c => c.k === "Dm" && c.rcp && c.rcp[0] && c.rcp[0].id && S.ridOf(c.rcp[0].id) === uid)
    }
    function msg(uid) { S.openDm(uid) }
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
                        activeFocusOnTab: true
                        Accessible.role: Accessible.Button
                        Accessible.name: modelData[1]
                        Keys.onReturnPressed: r.tab = modelData[0]
                        Keys.onSpacePressed: r.tab = modelData[0]
                        Lbl { id: tl; anchors.centerIn: parent; text: modelData[1]; color: r.tab === modelData[0] ? Th.tx : Th.tx2 }
                        Rectangle { anchors.fill: parent; radius: parent.radius; color: "transparent"; border.color: Th.ac; border.width: 2; visible: parent.activeFocus }
                        MouseArea { id: tm; anchors.fill: parent; hoverEnabled: true; cursorShape: Qt.PointingHandCursor; onClicked: r.tab = modelData[0] }
                    }
                }
            }
        }
        Fld {
            Layout.fillWidth: true
            Layout.maximumWidth: 720 - 32
            Layout.alignment: Qt.AlignHCenter
            Layout.topMargin: 16
            Layout.bottomMargin: 16
            placeholderText: "Search"
            onTextChanged: r.q = text
        }
        Lbl {
            Layout.fillWidth: true
            Layout.maximumWidth: 720 - 32
            Layout.alignment: Qt.AlignHCenter
            leftPadding: 12
            text: ({ on: "Online", all: "All friends", pend: "Pending", blk: "Blocked" })[r.tab] + "  ·  " + r.list.length
            color: Th.tx2
            font.pixelSize: Th.fsS
            font.weight: Font.DemiBold
            bottomPadding: 4
        }
        ListView {
            id: lv
            Layout.fillWidth: true
            Layout.maximumWidth: 720
            Layout.alignment: Qt.AlignHCenter
            Layout.fillHeight: true
            Layout.topMargin: 8
            clip: true
            model: r.list
            ScrollBar.vertical: Scr {}
            delegate: Rectangle {
                id: row
                property var f: modelData.items[0]
                property string uid: f.usr.id ? S.ridOf(f.usr.id) : ""
                property var p: S.pres[uid] || null
                property bool off: !p || p.st === "Off"
                x: 0
                width: lv.width
                height: 56
                radius: Th.rs
                color: ma.containsMouse || dots.containsMouse ? Th.hov : "transparent"
                Rectangle { width: parent.width - 16; x: 8; height: 1; color: Th.ln; visible: index > 0 && !ma.containsMouse }
                MouseArea { id: ma; anchors.fill: parent; hoverEnabled: true; z: -1; onClicked: S.prof(row.f.usr, av) }
                Av {
                    id: av
                    x: 8
                    anchors.verticalCenter: parent.verticalCenter
                    sz: 34
                    opacity: row.off && r.tab !== "pend" ? 0.6 : 1
                    src: row.f.usr.av || ""
                    name: row.f.usr.name
                    st: row.p && row.f.k !== "In" && row.f.k !== "Out" ? row.p.st : ""
                    ring: row.color.a > 0 ? Th.bg3 : Th.bg2
                    sv: S.svc(row.uid)
                }
                Column {
                    anchors.left: av.right
                    anchors.leftMargin: 12
                    anchors.right: acts.left
                    anchors.rightMargin: 8
                    anchors.verticalCenter: parent.verticalCenter
                    Row {
                        spacing: 6
                        Lbl { text: modelData.name; font.weight: Font.DemiBold; color: row.off && r.tab === "all" ? Th.tx2 : Th.tx; anchors.verticalCenter: parent.verticalCenter }
                        Lbl { text: row.f.usr.name; color: Th.tx3; visible: ma.containsMouse && row.f.usr.name !== modelData.name; anchors.verticalCenter: parent.verticalCenter }
                    }
                    Lbl {
                        width: parent.width
                        text: row.f.k === "In" ? "Incoming request" : row.f.k === "Out" ? "Outgoing request" : row.f.k === "Block" ? "Blocked" : row.p && row.p.act ? row.p.act : row.p ? ({ On: "Online", Idle: "Idle", Dnd: "Do not disturb", Off: "Offline" })[row.p.st] : "Offline"
                        color: Th.tx3
                        font.pixelSize: 12
                    }
                }
                Row {
                    id: acts
                    anchors.right: parent.right
                    anchors.rightMargin: 8
                    anchors.verticalCenter: parent.verticalCenter
                    spacing: 4
                    IBtn {
                        id: mb
                        n: "thread"
                        tip: "Message"
                        visible: row.f.k === "Friend"
                        onClicked: {
                            if (modelData.items.length === 1) r.msg(row.uid)
                            else mm.show(mb, 0, mb.height + 4)
                        }
                    }
                    IBtn { n: "check"; tip: "Accept"; visible: row.f.k === "In"; onClicked: S.c.rel(row.uid, "accept") }
                    IBtn { n: "x"; tip: "Ignore request"; red: true; visible: row.f.k === "In" || row.f.k === "Out"; onClicked: S.c.rel(row.uid, "remove") }
                    IBtn { id: dots; n: "dots"; tip: "More"; visible: row.f.k === "Friend" || row.f.k === "Block"; onClicked: dm.show(dots, 0, dots.height + 4) }
                }
                Ctx {
                    id: mm
                    model: modelData.items.map(x => ({ text: Th.svcName(S.svc(x.usr.id ? S.ridOf(x.usr.id) : "")), run: () => r.msg(x.usr.id ? S.ridOf(x.usr.id) : "") }))
                }
                Ctx {
                    id: dm
                    model: {
                        const a = []
                        if (row.f.k === "Friend") {
                            a.push({ text: "Message", run: () => mb.clicked() })
                            a.push({ text: "View profile", run: () => S.prof(row.f.usr, av) })
                            a.push({ sep: true })
                            for (const x of modelData.items) {
                                const id = x.usr.id ? S.ridOf(x.usr.id) : ""
                                const nm = x.usr.disp || x.usr.name
                                a.push({ text: "Remove friend" + (modelData.items.length > 1 ? " (" + Th.svcName(S.svc(id)) + ")" : ""), danger: true, run: () => U.ask({ title: "Remove friend", body: "Remove " + nm + " from your " + Th.svcName(S.svc(id)) + " friends?", ok: "Remove", danger: true, run: () => S.c.rel(id, "remove") }) })
                            }
                        } else {
                            a.push({ text: "Unblock", run: () => S.c.rel(row.uid, "remove") })
                        }
                        return a
                    }
                }
            }
            Column {
                anchors.centerIn: parent
                visible: r.list.length === 0
                spacing: 6
                Ico { anchors.horizontalCenter: parent.horizontalCenter; n: "users"; sz: 28; col: Th.tx3 }
                Lbl {
                    text: r.q ? "No matches" : ({ on: "Nobody is online right now", all: "Your friends list is empty", pend: "No pending requests", blk: "You have not blocked anyone" })[r.tab]
                    color: Th.tx3
                }
            }
        }
    }
}
