import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import Cord

Rectangle {
    id: r
    color: Th.bgc
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
    readonly property int dcAcc: { const a = Object.values(S.accs).find(x => x.svc === "dc" || x.svc === "Dc"); return a ? a.id : 0 }
    property string addSt: ""
    property string addMsg: ""
    function addFriend() {
        const n = af.text.trim()
        if (!n || !r.dcAcc) return
        addSt = "busy"
        addMsg = ""
        S.api(r.dcAcc, "add_friend", { name: n }, (ok, v, err) => {
            if (ok) af.text = ""
            r.addSt = ok ? "ok" : "err"
            r.addMsg = ok ? "Success! Your friend request to " + n + " was sent." : "Hm, didn't work. " + (/80004|Unknown User|No users/.test(err) ? "Double check that the username is correct." : err)
        })
    }
    ColumnLayout {
        anchors.fill: parent
        spacing: 0
        Rectangle {
            Layout.fillWidth: true
            Layout.preferredHeight: 48
            color: "transparent"
            Rectangle { anchors.bottom: parent.bottom; width: parent.width; height: 1; color: Th.ln }
            Flickable {
              anchors.fill: parent
              clip: true
              contentWidth: tabs.width + 32
              contentHeight: height
              boundsBehavior: Flickable.StopAtBounds
              interactive: contentWidth > width
            Row {
                id: tabs
                y: (48 - height) / 2
                x: 16
                spacing: 6
                Ico { visible: r.width >= 640; n: "users"; col: Th.tx2; anchors.verticalCenter: parent.verticalCenter }
                Lbl { visible: r.width >= 640; text: S.t("Friends"); font.weight: Font.DemiBold; anchors.verticalCenter: parent.verticalCenter; rightPadding: 12 }
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
                Rectangle {
                    visible: r.dcAcc > 0
                    width: afl.implicitWidth + 20
                    height: 28
                    radius: Th.rs
                    color: r.tab === "add" ? "transparent" : afm.containsMouse ? Qt.lighter(Th.ac, 1.08) : Th.ac
                    anchors.verticalCenter: parent.verticalCenter
                    Lbl { id: afl; anchors.centerIn: parent; text: S.t("Add Friend"); color: r.tab === "add" ? Th.ac : Th.acTx; font.weight: Font.DemiBold }
                    MouseArea { id: afm; anchors.fill: parent; hoverEnabled: true; cursorShape: Qt.PointingHandCursor; onClicked: r.tab = "add" }
                }
            }
            }
        }
        Column {
            visible: r.tab === "add"
            Layout.fillWidth: true
            Layout.maximumWidth: 720 - 32
            Layout.alignment: Qt.AlignHCenter
            Layout.topMargin: 20
            spacing: 8
            Lbl { text: S.t("Add Friend"); font.pixelSize: Th.fsL; font.weight: Font.Bold }
            Lbl { width: parent.width; text: S.t("You can add friends with their Discord username."); color: Th.tx2; wrapMode: Text.Wrap; elide: Text.ElideNone }
            Item { width: 1; height: 4 }
            Rectangle {
                width: parent.width
                height: 52
                radius: Th.r
                color: Th.bg
                border.color: r.addSt === "err" ? Th.err : r.addSt === "ok" ? Th.ok : af.activeFocus ? Th.a(Th.ac, 0.7) : Th.ln
                TextInput {
                    id: af
                    x: 14
                    width: parent.width - sb.width - 36
                    anchors.verticalCenter: parent.verticalCenter
                    color: Th.tx
                    font.family: Th.font
                    font.pixelSize: Th.fs
                    clip: true
                    selectByMouse: true
                    onTextChanged: r.addSt = ""
                    onAccepted: r.addFriend()
                    Lbl { visible: !af.text; text: S.t("You can add friends with their Discord username."); color: Th.tx2; anchors.verticalCenter: parent.verticalCenter }
                }
                Btn { id: sb; anchors.right: parent.right; anchors.rightMargin: 10; anchors.verticalCenter: parent.verticalCenter; height: 32; text: S.t("Send Friend Request"); pri: true; on: af.text.trim() !== "" && r.addSt !== "busy"; onClicked: r.addFriend() }
            }
            Lbl { visible: r.addMsg !== ""; width: parent.width; text: r.addMsg; color: r.addSt === "ok" ? Th.ok : Th.err; font.pixelSize: Th.fsS; wrapMode: Text.Wrap; elide: Text.ElideNone }
        }
        Fld {
            visible: r.tab !== "add"
            Layout.fillWidth: true
            Layout.maximumWidth: 720 - 32
            Layout.alignment: Qt.AlignHCenter
            Layout.topMargin: 16
            Layout.bottomMargin: 16
            placeholderText: S.t("Search")
            onTextChanged: r.q = text
        }
        Lbl {
            visible: r.tab !== "add"
            Layout.fillWidth: true
            Layout.maximumWidth: 720 - 32
            Layout.alignment: Qt.AlignHCenter
            leftPadding: 12
            text: (({ on: S.t("Online"), all: S.t("All friends"), pend: S.t("Pending"), blk: S.t("Blocked") })[r.tab] + " — " + r.list.length).toUpperCase()
            color: Th.tx3
            font.pixelSize: Th.fsS - 1
            font.weight: Font.Bold
            font.letterSpacing: 0.4
            bottomPadding: 4
        }
        Item { visible: r.tab === "add"; Layout.fillHeight: true }
        ListView {
            Smooth {}
            id: lv
            visible: r.tab !== "add"
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
                        text: row.f.k === "In" ? S.t("Incoming request") : row.f.k === "Out" ? S.t("Outgoing request") : row.f.k === "Block" ? S.t("Blocked") : row.p && row.p.act ? row.p.act : row.p ? ({ On: "Online", Idle: "Idle", Dnd: "Do not disturb", Off: "Offline" })[row.p.st] : "Offline"
                        color: Th.tx3
                        font.pixelSize: Th.fsXS
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
                        tip: S.t("Message")
                        visible: row.f.k === "Friend"
                        onClicked: {
                            if (modelData.items.length === 1) r.msg(row.uid)
                            else mm.show(mb, 0, mb.height + 4)
                        }
                    }
                    IBtn { n: "check"; tip: S.t("Accept"); visible: row.f.k === "In"; onClicked: S.c.rel(row.uid, "accept") }
                    IBtn { n: "x"; tip: S.t("Ignore request"); red: true; visible: row.f.k === "In" || row.f.k === "Out"; onClicked: S.c.rel(row.uid, "remove") }
                    IBtn { id: dots; n: "dots"; tip: S.t("More"); visible: row.f.k === "Friend" || row.f.k === "Block"; onClicked: dm.show(dots, 0, dots.height + 4) }
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
                            a.push({ text: S.t("Message"), run: () => mb.clicked() })
                            a.push({ text: S.t("View profile"), run: () => S.prof(row.f.usr, av) })
                            a.push({ sep: true })
                            for (const x of modelData.items) {
                                const id = x.usr.id ? S.ridOf(x.usr.id) : ""
                                const nm = x.usr.disp || x.usr.name
                                a.push({ text: S.t("Remove friend") + (modelData.items.length > 1 ? " (" + Th.svcName(S.svc(id)) + ")" : ""), danger: true, run: () => U.ask({ title: S.t("Remove friend"), body: S.t("Remove ") + nm + " from your " + Th.svcName(S.svc(id)) + " friends?", ok: S.t("Remove"), danger: true, run: () => S.c.rel(id, "remove") }) })
                            }
                        } else {
                            a.push({ text: S.t("Unblock"), run: () => S.c.rel(row.uid, "remove") })
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
