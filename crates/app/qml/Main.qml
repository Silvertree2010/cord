import QtQuick
import QtQuick.Controls
import QtQuick.Window
import Cord
import "Cord/Lay.js" as L

ApplicationWindow {
    id: win
    width: 1440
    height: 900
    minimumWidth: 720
    minimumHeight: 480
    visible: true
    title: S.selC ? S.chName(S.selC) + " - cord" : "cord"
    color: Th.bg
    font.family: Th.font
    font.pixelSize: Th.fs

    readonly property bool bar: U.inCall && !S.setup && !S.hasBlk("call")

    Component.onCompleted: {
        S.boot()
        const sh = Qt.application.arguments.indexOf("--shot")
        const sp = Qt.application.arguments.indexOf("--page")
        if (sp >= 0) { S.setPage = Qt.application.arguments[sp + 1]; S.showSet = true }
        const sl = Qt.application.arguments.indexOf("--preset")
        if (sl >= 0) S.lay = L.cp(L.presets[Qt.application.arguments[sl + 1]])
        if (Qt.application.arguments.indexOf("--edit") >= 0) { S.edit = true; U.onEdit() }
        if (Qt.application.arguments.indexOf("--quick") >= 0) S.qk = true
        if (Qt.application.arguments.indexOf("--emo") >= 0) Qt.callLater(() => { emoP.cb = null; emoP.x = win.width - emoP.width - 40; emoP.y = win.height - emoP.height - 90; emoP.open() })
        const so = Qt.application.arguments.indexOf("--open")
        if (so >= 0) { const g = Qt.application.arguments[so + 1]; Qt.callLater(() => S.pick(g)) }
        const sd = Qt.application.arguments.indexOf("--dm")
        if (sd >= 0) { const c = Qt.application.arguments[sd + 1]; Qt.callLater(() => { S.sel = "home"; S.open(c) }) }
        if (sh >= 0) shotT.out = Qt.application.arguments[sh + 1] || "/tmp/cord.png"
        const ss = Qt.application.arguments.indexOf("--steps")
        if (ss >= 0) {
            stepT.q = Qt.application.arguments[ss + 1].split(",").map(x => { const p = x.split("@"); return { a: p[0], t: +(p[1] || 0) } })
            stepT.interval = Math.max(1, stepT.q[0].t)
            stepT.start()
        }
    }

    function firstUsr() {
        for (const c of Object.values(S.chans || {})) if (c && c.rcp && c.rcp.length) return c.rcp[0]
        return null
    }

    readonly property var acts: ({
        emo: () => { emoP.cb = null; emoP.q = ""; emoP.x = win.width - emoP.width - 40; emoP.y = win.height - emoP.height - 90; emoP.open() },
        prof: () => S.prof(win.firstUsr(), root),
        quick: () => S.qk = true,
        unquick: () => S.qk = false,
        set: () => S.showSet = true,
        unset: () => S.showSet = false,
        look: () => S.setPage = "look",
        feat: () => S.setPage = "feat",
        edit: () => S.edit = true,
        unedit: () => S.edit = false,
        ctx: () => addM.show(root, 300, 200),
        home: () => S.pick("home"),
        g0: () => S.pick("dc:1:g0"),
        mx: () => S.pick("mx:2:g0"),
        close: () => U.closePop(),
        quit: () => Qt.quit()
    })

    Timer {
        id: stepT
        property var q: []
        property int i: 0
        onTriggered: {
            const s = q[i]
            if (win.acts[s.a]) win.acts[s.a](); else if (s.a.indexOf(":") > 0) S.pick(s.a); else console.warn("step?", s.a)
            console.log("STEP", s.a, s.t, Date.now())
            i++
            if (i < q.length) { interval = Math.max(1, q[i].t - s.t); start() }
        }
    }

    FontLoader { source: "Cord/fonts/Figtree-Regular.ttf" }
    FontLoader { source: "Cord/fonts/Figtree-Medium.ttf" }
    FontLoader { source: "Cord/fonts/Figtree-SemiBold.ttf" }
    FontLoader { source: "Cord/fonts/Figtree-Bold.ttf" }
    FontLoader { source: "Cord/fonts/Figtree-Italic.ttf" }
    FontLoader { source: "Cord/fonts/Figtree-MediumItalic.ttf" }
    FontLoader { source: "Cord/fonts/Figtree-SemiBoldItalic.ttf" }
    FontLoader { source: "Cord/fonts/Figtree-BoldItalic.ttf" }

    Node {
        id: root
        anchors.fill: parent
        readonly property real pad: S.edit ? 0 : Th.gap / 2 + 2
        anchors.leftMargin: pad
        anchors.rightMargin: pad
        Behavior on anchors.leftMargin { NumberAnimation { duration: Th.mM; easing.type: Easing.OutCubic } }
        Behavior on anchors.rightMargin { NumberAnimation { duration: Th.mM; easing.type: Easing.OutCubic } }
        anchors.bottomMargin: (win.bar ? callBar.height : 0) + pad
        anchors.topMargin: S.edit ? tb.height : pad
        n: S.lay
        p: []
        visible: !S.setup
    }

    Rectangle {
        id: callBar
        visible: win.bar
        anchors.bottom: parent.bottom
        width: parent.width
        height: 40
        color: Th.bg3
        z: 40
        Rectangle { width: parent.width; height: 1; color: Th.ln }
        property var ch: S.call.chan ? S.chans[S.ridOf(S.call.chan)] : null
        Row {
            x: 16
            anchors.verticalCenter: parent.verticalCenter
            spacing: 10
            Ico { n: "voice"; sz: 16; col: S.call.err ? Th.err : S.call.up ? Th.ok : Th.tx2; anchors.verticalCenter: parent.verticalCenter }
            Lbl {
                text: S.call.err ? "Call failed" : S.call.up ? "Connected" : "Connecting"
                color: S.call.err ? Th.err : S.call.up ? Th.ok : Th.tx2
                font.weight: Font.DemiBold
                anchors.verticalCenter: parent.verticalCenter
            }
            Lbl {
                text: (callBar.ch ? S.chName(callBar.ch.rid) : "") + (callBar.ch && callBar.ch.grid && S.gmap[callBar.ch.grid] ? " / " + S.gmap[callBar.ch.grid].name : "")
                color: Th.tx
                anchors.verticalCenter: parent.verticalCenter
                MouseArea { anchors.fill: parent; cursorShape: Qt.PointingHandCursor; onClicked: if (callBar.ch) { S.sel = callBar.ch.grid || "home"; S.open(callBar.ch.rid) } }
            }
            Lbl { visible: S.f("callTimer"); text: U.callEl; color: Th.tx2; anchors.verticalCenter: parent.verticalCenter }
        }
        Row {
            anchors.right: parent.right
            anchors.rightMargin: 8
            anchors.verticalCenter: parent.verticalCenter
            spacing: 2
            IBtn { n: S.call.mute ? "micoff" : "mic"; red: !!S.call.mute; tip: S.call.mute ? "Unmute" : "Mute"; onClicked: S.c.vset(!S.call.mute, !!S.call.deaf) }
            IBtn { n: S.call.deaf ? "headoff" : "head"; red: !!S.call.deaf; tip: S.call.deaf ? "Undeafen" : "Deafen"; onClicked: S.c.vset(!!S.call.mute, !S.call.deaf) }
            IBtn { n: "screen"; act: !!S.call.live; tip: S.call.live ? "Stop sharing" : "Share screen"; onClicked: S.call.live ? S.c.golive(false, "{}") : S.goLive() }
            IBtn { n: "hang"; red: true; tip: "Leave call"; onClicked: S.c.leave() }
        }
    }

    Ctx {
        id: addM
        model: U.addItems()
    }

    Rectangle {
        id: tb
        visible: opacity > 0.01
        opacity: S.edit ? 1 : 0
        Behavior on opacity { NumberAnimation { duration: Th.mM; easing.type: Easing.OutCubic } }
        anchors.top: parent.top
        anchors.left: parent.left
        anchors.right: parent.right
        z: 50
        height: 48
        color: Th.bg2
        Rectangle { anchors.bottom: parent.bottom; width: parent.width; height: 1; color: Th.ln }
        Lbl { x: 16; anchors.verticalCenter: parent.verticalCenter; text: "Editing layout"; color: Th.tx2 }
        Row {
            id: er
            anchors.centerIn: parent
            spacing: 8
            Btn { id: ab; text: "Add block"; ico: "plus"; height: 28; onClicked: addM.show(ab, 0, ab.height + 4) }
            Btn { text: "Undo"; ico: "undo"; height: 28; on: U.undo.length > 0; onClicked: U.undoLay() }
            Btn { text: "Reset layout"; out: true; height: 28; onClicked: U.resetLay() }
            Btn { text: "Cancel"; out: true; height: 28; onClicked: U.cancelEdit() }
            Btn { text: "Done"; pri: true; height: 28; onClicked: S.edit = false }
        }
    }

    Rectangle {
        id: emptyV
        visible: !S.setup && S.cfgAccs.length === 0 && !S.edit && !S.showSet
        anchors.fill: parent
        color: Th.bg
        z: 60
        Column {
            anchors.centerIn: parent
            spacing: 12
            width: Math.min(360, parent.width - 32)
            Lbl { width: parent.width; horizontalAlignment: Text.AlignHCenter; text: "No accounts yet"; font.pixelSize: Th.fsXL; font.weight: Font.Bold }
            Lbl { width: parent.width; horizontalAlignment: Text.AlignHCenter; text: "Add an account to see your servers and messages, or look around with demo data."; color: Th.tx2; wrapMode: Text.Wrap; elide: Text.ElideNone }
            Item { width: 1; height: 4 }
            Row {
                anchors.horizontalCenter: parent.horizontalCenter
                spacing: 8
                Btn { text: "Add account"; pri: true; height: 36; onClicked: { S.setPage = "accounts"; S.showSet = true } }
                Btn { text: "Try demo"; height: 36; onClicked: { S.c.add("dc", '{"demo":true}'); S.c.add("mx", '{"demo":true}'); S.c.add("st", '{"demo":true}') } }
            }
        }
    }

    Loader {
        id: stp
        anchors.fill: parent
        active: S.setup
        source: "Cord/Setup.qml"
        z: 80
    }
    Loader {
        id: setL
        anchors.fill: parent
        active: S.showSet || opacity > 0.01
        opacity: S.showSet ? 1 : 0
        Behavior on opacity { NumberAnimation { duration: Th.mF; easing.type: Easing.OutCubic } }
        source: "Cord/Settings.qml"
        z: 70
    }
    Loader {
        id: qs
        anchors.fill: parent
        active: S.qk || opacity > 0.01
        opacity: S.qk ? 1 : 0
        Behavior on opacity { NumberAnimation { duration: Th.mF; easing.type: Easing.OutCubic } }
        source: "Cord/Quick.qml"
        z: 75
    }

    Column {
        anchors.top: parent.top
        anchors.right: parent.right
        anchors.margins: 16
        anchors.topMargin: 56 + (S.edit ? tb.height : 0)
        spacing: 8
        z: 90
        Repeater {
            model: U.toasts
            Rectangle {
                id: toR
                transform: Translate { id: toT }
                Component.onCompleted: toA.start()
                ParallelAnimation {
                    id: toA
                    NumberAnimation { target: toT; property: "x"; from: 40; to: 0; duration: Th.mS; easing.type: Easing.OutCubic }
                    NumberAnimation { target: toR; property: "opacity"; from: 0; to: 1; duration: Th.mM; easing.type: Easing.OutCubic }
                }
                width: 420
                height: Math.max(40, tx.implicitHeight + 20)
                radius: Th.rs
                color: Th.bg3
                border.color: modelData.ok ? Th.ln : Th.err
                Lbl {
                    id: tx
                    x: 12
                    anchors.verticalCenter: parent.verticalCenter
                    width: parent.width - 52 - (modelData.retry ? 64 : 0)
                    text: modelData.msg
                    wrapMode: Text.Wrap
                    elide: Text.ElideNone
                    color: Th.tx
                }
                Btn {
                    visible: !!modelData.retry
                    anchors.right: parent.right
                    anchors.rightMargin: 40
                    anchors.verticalCenter: parent.verticalCenter
                    height: 28
                    text: "Retry"
                    onClicked: { const f = modelData.retry; U.dismiss(modelData.id); f() }
                }
                IBtn {
                    anchors.right: parent.right
                    anchors.rightMargin: 6
                    anchors.verticalCenter: parent.verticalCenter
                    n: "x"
                    sz: 26
                    isz: 14
                    tip: "Dismiss"
                    onClicked: U.dismiss(modelData.id)
                }
            }
        }
    }

    Feats {}
    Emo { id: emoP }
    Prof { id: profP }
    Dlg { id: dlg }
    View { id: viewP; anchors.fill: parent; z: 95 }

    Rectangle {
        id: tipR
        readonly property bool on: U.tipText !== "" && U.tipOk && !!U.tipItem && U.tipItem.visible
        visible: opacity > 0.01
        opacity: on ? 1 : 0
        scale: on ? 1 : 0.94
        Behavior on opacity { NumberAnimation { duration: Th.mF; easing.type: Easing.OutCubic } }
        Behavior on scale { NumberAnimation { duration: Th.mF; easing.type: Easing.OutCubic } }
        z: 300
        width: tt.implicitWidth + 16
        height: 26
        radius: Th.rs
        color: Th.bg4
        border.color: Th.ln
        readonly property bool rt: U.tipSide === "r"
        x: rt ? (U.tipBox.ex >= 0 ? U.tipBox.ex + 4 : U.tipBox.x + U.tipBox.w + 4) : Math.max(4, Math.min(win.width - width - 4, U.tipBox.x + U.tipBox.w / 2 - width / 2))
        y: rt ? Math.max(4, Math.min(win.height - height - 4, U.tipBox.y + U.tipBox.h / 2 - height / 2)) : U.tipBox.y + U.tipBox.h + 6 + height > win.height - 4 ? Math.max(52, U.tipBox.y - height - 6) : Math.max(U.tipBox.y < 48 ? 52 : 4, U.tipBox.y + U.tipBox.h + 6)
        Lbl { id: tt; anchors.centerIn: parent; text: U.tipText; color: Th.tx; font.pixelSize: Th.fsS; elide: Text.ElideNone }
    }

    Connections {
        target: S
        function onEmoReq(at, cb) {
            emoP.cb = cb
            emoP.q = ""
            const p = at.mapToItem(win.contentItem, 0, 0)
            emoP.x = Math.max(8, Math.min(win.width - emoP.width - 8, p.x - emoP.width + at.width))
            emoP.y = p.y - emoP.height - 8 >= 8 ? p.y - emoP.height - 8 : Math.min(win.height - emoP.height - 8, p.y + at.height + 8)
            emoP.open()
        }
        function onProfReq(u, at) {
            if (!u) return
            profP.u = u
            const p = at.mapToItem(win.contentItem, 0, 0)
            const rx = p.x + at.width + 8
            profP.x = rx + profP.width + 8 <= win.width ? rx : Math.max(8, p.x - profP.width - 8)
            profP.y = Math.max(8, Math.min(win.height - profP.height - 8, p.y))
            profP.open()
        }
        function onViewReq(u, v) { U.closePop(); viewP.vid = v; viewP.url = u }
        function onEditChanged() { U.onEdit() }
        function onQkChanged() { U.hideTip(); if (S.qk) U.closePop() }
        function onSelChanged() { U.hideTip() }
        function onSelCChanged() { U.hideTip() }
        function onShowSetChanged() { U.hideTip() }
        function onEv(d) { if (d.t === "Err") U.err(d.d) }
    }
    Connections {
        target: U
        function onAskReq(o) { dlg.show(o) }
    }

    Instantiator {
        model: S.mods.filter(m => m.on)
        delegate: Loader {
            source: modelData.url
            onStatusChanged: if (status === Loader.Error) console.warn("mod failed:", modelData.id)
        }
    }

    function esc() {
        if (U.pop) { U.pop.close(); return }
        if (viewP.url !== "") { viewP.close(); return }
        if (S.qk) { S.qk = false; return }
        if (S.edit) { U.cancelEdit(); return }
        if (S.showSet) { S.showSet = false; return }
        S.reply = ""
        S.editing = ""
    }

    Item {
        focus: true
        Keys.onPressed: e => { if (e.key === Qt.Key_QuoteLeft && (e.modifiers & (Qt.platform.os === "osx" ? Qt.MetaModifier : Qt.ControlModifier)) && !e.isAutoRepeat) S.c.ptt(true) }
        Keys.onReleased: e => { if (e.key === Qt.Key_QuoteLeft && !e.isAutoRepeat) S.c.ptt(false) }
    }
    Shortcut { sequence: "Ctrl+K"; context: Qt.ApplicationShortcut; onActivated: { if (!S.setup) S.qk = !S.qk } }
    Shortcut { sequence: "Ctrl+,"; context: Qt.ApplicationShortcut; onActivated: { if (!S.setup) { if (S.edit) U.cancelEdit(() => { S.showSet = true }); else S.showSet = !S.showSet } } }
    Shortcut { sequence: "Ctrl+E"; context: Qt.ApplicationShortcut; onActivated: { if (S.setup) return; if (S.edit) S.edit = false; else { S.showSet = false; S.edit = true } } }
    Shortcut { sequence: "Escape"; context: Qt.ApplicationShortcut; onActivated: win.esc() }
    Shortcut { sequence: "Shift+Escape"; context: Qt.ApplicationShortcut; onActivated: S.c.ack(S.selC) }
    Shortcut { sequence: "Alt+Down"; context: Qt.ApplicationShortcut; onActivated: win.step(1) }
    Shortcut { sequence: "Alt+Up"; context: Qt.ApplicationShortcut; onActivated: win.step(-1) }
    function step(d) {
        const l = (S.sel === "home" ? S.dms() : S.chList(S.sel)).filter(c => c.k !== "Cat" && c.k !== "Voice" && c.k !== "Stage")
        const i = l.findIndex(c => c.rid === S.selC)
        const n = l[(i + d + l.length) % l.length]
        if (n) S.open(n.rid)
    }
}
