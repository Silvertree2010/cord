import QtQuick
import QtQuick.Controls
import Cord

Rectangle {
    id: r
    property var o: ({})
    implicitWidth: 248
    implicitHeight: 60
    color: Th.bg2
    property var al: Object.values(S.accs)
    property int cur: S.ui.meAcc && S.accs[S.ui.meAcc] ? S.ui.meAcc : (al.length ? al[0].id : 0)
    property var a: S.accs[cur] || null
    property var mu: S.call.mute || false
    readonly property bool cst: !!(S.ui.cst && (S.ui.cst.text || S.ui.cst.emoji))
    function setCst(t, e) {
        const v = t || e ? { text: t || "", emoji: e || "" } : null
        S.setUi("cst", v)
        for (const a of r.al) if (a.svc === "dc" || a.svc === "Dc") S.api(a.id, "custom_status", { text: t || "", emoji: e || "" })
    }
    property var de: S.call.deaf || false
    Row {
        x: 10
        anchors.verticalCenter: parent.verticalCenter
        spacing: 10
        width: parent.width - btns.width - 16
        Av {
            id: av
            sz: 36
            src: r.a ? (r.a.me.av || "") : ""
            name: r.a ? r.a.me.name : "?"
            st: S.ui.st || "On"
            ring: Th.bg2
            sv: r.a ? S.tag(r.a.svc) : ""
            anchors.verticalCenter: parent.verticalCenter
            MouseArea { anchors.fill: parent; cursorShape: Qt.PointingHandCursor; onClicked: m => { sm.show(av, 0, -sm.implicitHeight - 8) } }
        }
        Column {
            anchors.verticalCenter: parent.verticalCenter
            width: parent.width - 44
            Row {
                spacing: 6
                width: parent.width
                Lbl { width: parent.width - (U.demo && r.width >= 300 ? 46 : 0); text: r.a ? (r.a.me.disp || r.a.me.name) : "No account"; font.weight: Font.DemiBold; font.pixelSize: Th.fs + 1; anchors.verticalCenter: parent.verticalCenter }
                Rectangle {
                    visible: U.demo && r.width >= 300
                    width: dc.implicitWidth + 10
                    height: 16
                    radius: 4
                    color: Th.bg4
                    anchors.verticalCenter: parent.verticalCenter
                    Lbl { id: dc; anchors.centerIn: parent; text: "Demo"; color: Th.tx2; font.pixelSize: Th.fsP; font.weight: Font.DemiBold }
                }
            }
            Lbl {
                width: parent.width
                text: r.mu || r.de ? (r.de ? "Deafened" : "Muted") : r.cst ? ((S.ui.cst.emoji ? S.ui.cst.emoji + " " : "") + (S.ui.cst.text || "")) : ({ On: "Online", Idle: "Idle", Dnd: "Do Not Disturb", Off: "Invisible" })[S.ui.st || "On"]
                color: r.mu || r.de ? Th.err : Th.tx2
                font.pixelSize: Th.fsS
            }
        }
    }
    Row {
        id: btns
        anchors.right: parent.right
        anchors.rightMargin: 10
        anchors.verticalCenter: parent.verticalCenter
        spacing: 4
        IBtn { n: r.mu ? "micoff" : "mic"; red: r.mu; tip: r.mu ? "Unmute" : "Mute"; isz: 20; onClicked: S.c.vset(!r.mu, r.de) }
        IBtn { visible: r.width >= 300; n: "chev"; sz: 18; isz: 14; tip: "Voice settings"; anchors.verticalCenter: parent.verticalCenter; onClicked: { S.setPage = "voice"; S.showSet = true } }
        IBtn { n: r.de ? "headoff" : "head"; red: r.de; tip: r.de ? "Undeafen" : "Deafen"; isz: 20; onClicked: S.c.vset(r.mu, !r.de) }
        IBtn { visible: r.width >= 300; n: "chev"; sz: 18; isz: 14; tip: "Output settings"; anchors.verticalCenter: parent.verticalCenter; onClicked: { S.setPage = "voice"; S.showSet = true } }
        IBtn { n: "cog"; tip: "Settings  (Ctrl+,)"; isz: 20; onClicked: S.showSet = true }
    }
    Ctx {
        id: sm
        w: 230
        model: {
            const st = [["On", "Online"], ["Idle", "Idle"], ["Dnd", "Do not disturb"], ["Off", "Invisible"]].map(x => ({
                text: x[1],
                checked: (S.ui.st || "On") === x[0],
                run: () => { S.setUi("st", x[0]); S.c.set_st(x[0].toLowerCase(), "") }
            }))
            const ac = r.al.map(x => ({
                text: (x.me.disp || x.me.name) + "  ·  " + Th.svcName(x.svc),
                checked: x.id === r.cur,
                run: () => S.setUi("meAcc", x.id)
            }))
            const cs = [{ text: r.cst ? "Edit Custom Status" : "Set Custom Status", run: () => csP.show() }, { text: "Clear Custom Status", when: r.cst, run: () => r.setCst("", "") }]
            return st.concat([{ sep: true }], cs, [{ sep: true }], ac, [{ text: "Add account", run: () => { S.setPage = "accounts"; S.showSet = true } }])
        }
    }
    Popup {
        id: csP
        parent: Overlay.overlay
        anchors.centerIn: Overlay.overlay
        modal: true
        width: 420
        padding: 20
        enter: PopIn {}
        exit: PopOut {}
        closePolicy: Popup.CloseOnEscape | Popup.CloseOnPressOutside
        Overlay.modal: Rectangle { color: "#99000000" }
        property string emo: ""
        function show() { emo = S.ui.cst ? (S.ui.cst.emoji || "") : ""; cf.text = S.ui.cst ? (S.ui.cst.text || "") : ""; open(); cf.forceActiveFocus() }
        function save() { r.setCst(cf.text.trim(), emo); close() }
        background: Pane {}
        contentItem: Column {
            spacing: 14
            Lbl { text: "Set a custom status"; font.pixelSize: Th.fsL; font.weight: Font.Bold }
            Rectangle {
                width: parent.width
                height: 44
                radius: Th.rs
                color: Th.bg
                border.color: cf.activeFocus ? Th.a(Th.ac, 0.7) : Th.ln
                Rectangle {
                    id: eb
                    x: 6
                    anchors.verticalCenter: parent.verticalCenter
                    width: 32
                    height: 32
                    radius: 6
                    color: em.containsMouse ? Th.hov : "transparent"
                    Lbl { visible: csP.emo !== ""; anchors.centerIn: parent; text: csP.emo; font.pixelSize: 18 }
                    Ico { visible: csP.emo === ""; anchors.centerIn: parent; n: "smile"; sz: 18; col: Th.tx2 }
                    MouseArea { id: em; anchors.fill: parent; hoverEnabled: true; cursorShape: Qt.PointingHandCursor; onClicked: S.emoji(eb, e => { if (!e.id) csP.emo = e.name }) }
                }
                TextInput {
                    id: cf
                    anchors.left: eb.right
                    anchors.leftMargin: 8
                    anchors.right: parent.right
                    anchors.rightMargin: 12
                    anchors.verticalCenter: parent.verticalCenter
                    color: Th.tx
                    font.family: Th.font
                    font.pixelSize: Th.fs
                    maximumLength: 128
                    clip: true
                    selectByMouse: true
                    onAccepted: csP.save()
                    Lbl { visible: !cf.text; text: "Support has arrived!"; color: Th.tx2; anchors.verticalCenter: parent.verticalCenter }
                }
            }
            Row {
                anchors.right: parent.right
                spacing: 8
                Btn { text: "Cancel"; out: true; onClicked: csP.close() }
                Btn { text: "Save"; pri: true; onClicked: csP.save() }
            }
        }
    }
}
