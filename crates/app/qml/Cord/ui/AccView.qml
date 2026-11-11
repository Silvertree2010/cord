import QtQuick
import QtQuick.Controls
import Cord
import "../Fmt.js" as F

Column {
    id: r
    property var a: ({})
    readonly property string tab: S.accSub
    property var info: null
    property var ses: null
    property string err: ""
    property bool busy: false
    readonly property var live: S.accs[a.id]
    readonly property var caps: info && info.caps ? info.caps : []
    readonly property string accId: S.tag(a.svc) + ":" + a.id
    width: parent ? parent.width : 400
    spacing: 8

    function load() {
        info = null
        ses = null
        err = ""
        if (!S.up.includes(a.id) && a.base !== "demo") { err = "Connect this account to manage it"; return }
        S.api(a.id, "acct", {}, (ok, v, e) => { if (ok) info = v; else err = e || "Could not load the account" })
        S.api(a.id, "sessions", {}, (ok, v) => { if (ok) ses = v })
    }
    onAChanged: { S.accSub = "prof"; load() }
    Component.onCompleted: load()
    Connections { target: S; function onUpChanged() { if (!r.info) r.load() } }

    function save(op, args, msg) {
        busy = true
        S.api(a.id, op, args, (ok, v, e) => {
            busy = false
            if (ok) { U.flash(msg); load() } else U.flash(e || "Could not save")
        })
    }

    Rectangle {
        width: parent.width
        height: 72
        radius: Th.r
        color: Th.bg
        border.color: Th.ln
        Av { id: aa; x: 16; anchors.verticalCenter: parent.verticalCenter; sz: 44; src: r.live ? (r.live.me.av || "") : ""; name: r.live ? r.live.me.name : "?" }
        Column {
            anchors.left: aa.right
            anchors.leftMargin: 14
            anchors.verticalCenter: parent.verticalCenter
            spacing: 3
            Lbl { text: r.info && r.info.name ? r.info.name : r.live ? (r.live.me.disp || r.live.me.name) : "Not connected"; font.pixelSize: Th.fsL; font.weight: Font.DemiBold }
            Row {
                spacing: 8
                Badge { sv: r.a.svc || "dc"; txt: true; anchors.verticalCenter: parent.verticalCenter }
                Lbl { text: r.info && r.info.user ? r.info.user : ""; color: Th.tx2; font.pixelSize: Th.fsS; anchors.verticalCenter: parent.verticalCenter }
            }
        }
        Btn {
            anchors.right: parent.right
            anchors.rightMargin: 16
            anchors.verticalCenter: parent.verticalCenter
            text: r.a.base === "demo" ? S.t("Remove") : S.t("Log out")
            gh: true
            onClicked: U.ask({
                title: r.a.base === "demo" ? S.t("Remove demo account") : S.t("Log out"),
                body: (r.a.base === "demo" ? S.t("Remove this account on ") : S.t("Log out of this account on ")) + Th.svcName(S.tag(r.a.svc)) + "?",
                ok: r.a.base === "demo" ? S.t("Remove") : S.t("Log out"),
                danger: true,
                run: () => S.c.rm_acc(r.a.id)
            })
        }
    }

    Seg {
        opts: [["prof", "Profile"], ["sess", "Sessions"]]
        cur: r.tab
        onPick: v => S.accSub = v
    }

    Lbl { visible: r.err !== ""; width: parent.width; text: r.err; color: Th.tx2; wrapMode: Text.Wrap; elide: Text.ElideNone; topPadding: 8 }

    Item {
        id: host
        width: parent.width
        height: r.tab === "prof" ? pf.implicitHeight : ss.implicitHeight
        clip: false

        Column {
            id: pf
            width: parent.width
            visible: opacity > 0.01
            opacity: r.tab === "prof" ? 1 : 0
            x: r.tab === "prof" ? 0 : -14
            Behavior on opacity { NumberAnimation { duration: Th.mM; easing.type: Easing.OutCubic } }
            Behavior on x { NumberAnimation { duration: Th.mM; easing.type: Easing.OutCubic } }
            spacing: 8
            Grp {
                visible: !!r.info
                Item {
                    width: parent.width
                    height: 64
                    Column { x: 16; anchors.verticalCenter: parent.verticalCenter; spacing: 2; Lbl { text: S.t("Display name"); font.weight: Font.Medium } Lbl { text: S.t("Shown instead of your username"); color: Th.tx2; font.pixelSize: Th.fsS } }
                    Row {
                        anchors.right: parent.right
                        anchors.rightMargin: 12
                        anchors.verticalCenter: parent.verticalCenter
                        spacing: 8
                        Fld { id: nm; width: 220; height: 34; text: r.info ? r.info.name : ""; enabled: r.caps.includes("name"); onAccepted: sv.clicked() }
                        Btn { id: sv; text: S.t("Save"); pri: true; height: 34; on: r.caps.includes("name") && !r.busy && nm.text.trim() !== "" && nm.text !== (r.info ? r.info.name : ""); onClicked: r.save("set_name", { name: nm.text.trim() }, "Name saved") }
                    }
                }
                Item {
                    visible: !!(r.info && r.info.user)
                    width: parent.width
                    height: 56
                    Rectangle { visible: true; anchors.top: parent.top; x: 16; width: parent.width - 32; height: 1; color: Th.ln }
                    Column { x: 16; anchors.verticalCenter: parent.verticalCenter; spacing: 2; Lbl { text: S.t("Username"); font.weight: Font.Medium } Lbl { text: r.info ? r.info.user : ""; color: Th.tx2; font.pixelSize: Th.fsS } }
                }
                Item {
                    visible: !!(r.info && r.info.email)
                    width: parent.width
                    height: 56
                    Rectangle { anchors.top: parent.top; x: 16; width: parent.width - 32; height: 1; color: Th.ln }
                    Column { x: 16; anchors.verticalCenter: parent.verticalCenter; spacing: 2; Lbl { text: S.t("Email"); font.weight: Font.Medium } Lbl { text: r.info ? (S.ui.streamer ? "\u2022\u2022\u2022\u2022\u2022\u2022\u2022\u2022@\u2022\u2022\u2022\u2022" : r.info.email) : ""; color: Th.tx2; font.pixelSize: Th.fsS } }
                }
                Item {
                    visible: r.caps.includes("bio")
                    width: parent.width
                    height: 132
                    Rectangle { anchors.top: parent.top; x: 16; width: parent.width - 32; height: 1; color: Th.ln }
                    Lbl { x: 16; y: 14; text: S.t("About me"); font.weight: Font.Medium }
                    Rectangle {
                        x: 16
                        y: 40
                        width: parent.width - 32 - 76
                        height: 76
                        radius: Th.rs
                        color: Qt.darker(Th.bg3, Th.dark ? 1.12 : 1.0)
                        border.color: bio.activeFocus ? Th.a(Th.ac, 0.7) : Th.ln
                        ScrollView {
                            anchors.fill: parent
                            anchors.margins: 2
                            ScrollBar.vertical: Scr {}
                            TextArea { id: bio; text: r.info ? r.info.bio : ""; wrapMode: TextEdit.Wrap; color: Th.tx; font.family: Th.font; font.pixelSize: Th.fs; background: null; leftPadding: 10; topPadding: 8; selectByMouse: true; selectionColor: Th.ac; selectedTextColor: Th.acTx }
                        }
                    }
                    Btn { x: parent.width - 16 - 60; y: 40; width: 60; text: S.t("Save"); pri: true; height: 34; on: !r.busy && bio.text !== (r.info ? r.info.bio : ""); onClicked: r.save("set_bio", { bio: bio.text }, "About me saved") }
                }
            }
        }

        Column {
            id: ss
            width: parent.width
            visible: opacity > 0.01
            opacity: r.tab === "sess" ? 1 : 0
            x: r.tab === "sess" ? 0 : 14
            Behavior on opacity { NumberAnimation { duration: Th.mM; easing.type: Easing.OutCubic } }
            Behavior on x { NumberAnimation { duration: Th.mM; easing.type: Easing.OutCubic } }
            spacing: 8
            Lbl { visible: !!r.ses && r.ses.list.length === 0; text: S.t("No sessions found"); color: Th.tx2 }
            Grp {
                visible: !!r.ses && r.ses.list.length > 0
                Repeater {
                    model: r.ses ? r.ses.list : []
                    Item {
                        id: sr
                        readonly property bool cur: modelData.id === r.ses.cur
                        property bool ren: false
                        width: parent.width
                        height: 64
                        Rectangle { visible: index > 0; anchors.top: parent.top; x: 16; width: parent.width - 32; height: 1; color: Th.ln }
                        Rectangle {
                            id: ic
                            x: 16
                            anchors.verticalCenter: parent.verticalCenter
                            width: 36
                            height: 36
                            radius: Th.rs
                            color: Th.bg4
                            Ico { anchors.centerIn: parent; n: modelData.kind === "mobile" ? "phone" : modelData.kind === "web" ? "globe" : "screen"; sz: 18; col: Th.tx2 }
                        }
                        Column {
                            anchors.left: ic.right
                            anchors.leftMargin: 12
                            anchors.right: ab.left
                            anchors.rightMargin: 12
                            anchors.verticalCenter: parent.verticalCenter
                            spacing: 2
                            Row {
                                spacing: 8
                                visible: !sr.ren
                                Lbl { text: modelData.name || "Unknown device"; font.weight: Font.Medium; anchors.verticalCenter: parent.verticalCenter; elide: Text.ElideRight; width: Math.min(implicitWidth, 300) }
                                Rectangle { visible: sr.cur; width: ct.implicitWidth + 12; height: 16; radius: Th.rc; color: Th.a(Th.ok, 0.18); anchors.verticalCenter: parent.verticalCenter; Lbl { id: ct; anchors.centerIn: parent; text: S.t("This device"); color: Th.ok; font.pixelSize: Th.fsP; font.weight: Font.DemiBold } }
                            }
                            Lbl { visible: !sr.ren; text: [modelData.loc, modelData.last ? (Date.now() / 1000 - modelData.last < 120 ? "Active now" : F.rel(modelData.last * 1000, Date.now())) : ""].filter(x => x).join("  ·  "); color: Th.tx2; font.pixelSize: Th.fsS }
                            Fld { id: rn; visible: sr.ren; width: parent.width; height: 32; text: modelData.name || ""; onAccepted: { r.save("session_name", { id: modelData.id, name: text.trim() }, "Renamed"); sr.ren = false } Keys.onEscapePressed: sr.ren = false; onVisibleChanged: if (visible) forceActiveFocus() }
                        }
                        Row {
                            id: ab
                            anchors.right: parent.right
                            anchors.rightMargin: 12
                            anchors.verticalCenter: parent.verticalCenter
                            spacing: 6
                            Btn { visible: r.caps.includes("rename") && !sr.ren; text: S.t("Rename"); gh: true; height: 30; onClicked: sr.ren = true }
                            Btn { visible: sr.ren; text: S.t("Save"); pri: true; height: 30; onClicked: { r.save("session_name", { id: modelData.id, name: rn.text.trim() }, "Renamed"); sr.ren = false } }
                            Btn { visible: r.caps.includes("end") && !sr.cur; text: S.t("Log out"); gh: true; height: 30; onClicked: U.ask({ title: S.t("Log out session"), body: S.t("Log out ") + (modelData.name || "this device") + "?", ok: S.t("Log out"), danger: true, run: () => r.save("session_end", { id: modelData.id }, "Session ended") }) }
                        }
                    }
                }
            }
            Lbl { visible: !!r.ses && !r.caps.includes("end"); width: parent.width; text: S.t("This network needs your password to end other sessions. Do that in its own app."); color: Th.tx2; font.pixelSize: Th.fsS; wrapMode: Text.Wrap; elide: Text.ElideNone }
        }
    }
}
