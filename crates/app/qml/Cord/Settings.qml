import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import Cord
import "Lay.js" as L

Rectangle {
    id: r
    color: Th.bg
    NumberAnimation on opacity { from: 0; to: 1; duration: Th.mM; easing.type: Easing.OutCubic }
    NumberAnimation on scale { from: 1.02; to: 1; duration: Th.mS; easing.type: Easing.OutCubic }
    MouseArea { anchors.fill: parent }
    readonly property var pages: [
        ["accounts", "My accounts", "users"], ["privacy", "Privacy", "lock"],
        ["look", "Appearance", "eye"], ["layout", "Layout", "layout"], ["notif", "Notifications", "bell"], ["voice", "Voice and video", "mic"],
        ["feat", "Client mods", "grid"],
        ["mods", "Mods and themes", "package"], ["keys", "Shortcuts", "keyboard"], ["about", "About", "info"]
    ]
    readonly property var fcat: ({
        privacy: ["silenttyping", "noread", "notrack", "noreplyping"],
        notif: ["relnotif"],
        voice: ["streamq", "callTimer"],
        look: ["anim", "platform", "typingtweaks"],
        chat: ["msglog", "hidden", "spoilers", "translate", "copyraw", "readall", "pinDms"],
        media: ["fakenitro", "spotify"]
    })
    function fdef(k) { return S.featDefs.find(x => x.k === k) || null }
    function fother() {
        const used = {}
        for (const c in r.fcat) for (const k of r.fcat[c]) used[k] = true
        return S.featDefs.filter(x => !used[x.k]).map(x => x.k)
    }
    readonly property var cur: pages.find(p => p[0] === S.setPage) || pages[0]
    property string pasteErr: ""
    function tcols(t) {
        if (t.id === "caelestia") {
            const x = S.pj(S.c.scheme(), {})
            const c = x.colours
            if (c) return ["#" + (c.surfaceContainerLowest || c.background), "#" + c.surfaceContainer, "#" + c.primary]
            return [Th.bg, Th.bg3, Th.ac]
        }
        const j = S.pj(t.path ? S.c.rdf(t.path) : "", {})
        return [j.bg || Th.bg, j.bg3 || Th.bg3, j.ac || Th.ac]
    }

    property string q: ""
    readonly property var groups: [
        ["User settings", ["accounts", "privacy"]],
        ["App settings", ["look", "layout", "notif", "voice"]],
        ["Client mods", ["feat"]],
        ["Advanced", ["mods", "keys", "about"]]
    ]
    readonly property var desc: ({
        accounts: "Your Discord, Matrix and Stoat accounts and your status",
        privacy: "What other people and Discord can see from you",
        layout: "Presets, blocks and the layout editor",
        look: "Theme, accent, text size, density and motion",
        notif: "Desktop alerts and relationship changes",
        feat: "Built-in tweaks for chat and media",
        voice: "Microphone, push to talk, calls and screen share",
        mods: "Your own QML mods, blocks and themes",
        keys: "Keyboard and mouse shortcuts",
        about: "Version, license and source"
    })
    function pgOf(id) { return r.pages.find(p => p[0] === id) }
    function hit(id) {
        const p = pgOf(id)
        const t = r.q.trim().toLowerCase()
        return !t || p[1].toLowerCase().indexOf(t) >= 0 || (r.desc[id] || "").toLowerCase().indexOf(t) >= 0
    }
    Keys.onEscapePressed: S.showSet = false
    property var al: Object.values(S.accs)
    property var me: al.length ? al[0] : null

    RowLayout {
        anchors.fill: parent
        anchors.margins: Th.gap + 2
        spacing: Th.gap
        Rectangle {
            Layout.fillHeight: true
            Layout.preferredWidth: 272
            radius: Th.r
            color: Th.bgc
            border.color: Th.ln
            Column {
                anchors.fill: parent
                anchors.margins: 10
                spacing: 2
                Rectangle {
                    width: parent.width
                    height: 60
                    radius: Th.rs
                    color: "transparent"
                    Av { id: ma_; x: 6; anchors.verticalCenter: parent.verticalCenter; sz: 40; src: r.me ? (r.me.me.av || "") : ""; name: r.me ? r.me.me.name : "?"; st: S.ui.st || "On"; ring: Th.bgc }
                    Column {
                        anchors.left: ma_.right
                        anchors.leftMargin: 12
                        anchors.right: parent.right
                        anchors.verticalCenter: parent.verticalCenter
                        Lbl { width: parent.width; text: r.me ? (r.me.me.disp || r.me.me.name) : "cord"; font.pixelSize: Th.fs + 1; font.weight: Font.DemiBold }
                        Lbl { width: parent.width; text: r.al.length > 1 ? r.al.length + " accounts" : r.me ? r.me.me.name : "No account"; color: Th.tx2; font.pixelSize: Th.fsS }
                    }
                }
                Item { width: 1; height: 6 }
                Rectangle {
                    width: parent.width
                    height: 34
                    radius: 6
                    color: Th.bg
                    Ico { id: si; x: 10; anchors.verticalCenter: parent.verticalCenter; n: "search"; sz: 16; col: Th.tx2 }
                    TextInput {
                        id: sq
                        anchors.left: si.right
                        anchors.leftMargin: 8
                        anchors.right: parent.right
                        anchors.rightMargin: 10
                        anchors.verticalCenter: parent.verticalCenter
                        color: Th.tx
                        font.family: Th.font
                        font.pixelSize: Th.fsS + 1
                        clip: true
                        selectByMouse: true
                        onTextChanged: r.q = text
                        Keys.onReturnPressed: { const f = r.pages.find(p => r.hit(p[0])); if (f) S.setPage = f[0] }
                        Lbl { visible: !sq.text; text: "Search settings"; color: Th.tx2; font.pixelSize: Th.fsS + 1; anchors.verticalCenter: parent.verticalCenter }
                    }
                }
                Item { width: 1; height: 6 }
                Repeater {
                    model: r.groups
                    Column {
                        width: parent.width
                        spacing: 2
                        readonly property var ids: modelData[1].filter(x => r.hit(x))
                        visible: ids.length > 0
                        Lbl { text: modelData[0].toUpperCase(); color: Th.tx2; font.pixelSize: Th.fsS - 1; font.weight: Font.Bold; font.letterSpacing: 0.6; leftPadding: 10; topPadding: 10; bottomPadding: 4 }
                        Repeater {
                            model: parent.ids
                            Rectangle {
                                readonly property var pp: r.pgOf(modelData)
                                readonly property bool on: S.setPage === modelData
                                width: parent.width
                                height: 38
                                radius: 6
                                color: on ? Th.sel : pm.containsMouse ? Th.hov : "transparent"
                                Behavior on color { ColorAnimation { duration: Th.mF } }
                                activeFocusOnTab: true
                                Accessible.role: Accessible.Button
                                Accessible.name: pp[1]
                                Keys.onReturnPressed: S.setPage = modelData
                                Keys.onSpacePressed: S.setPage = modelData
                                Rectangle {
                                    visible: parent.on
                                    width: 3
                                    height: 18
                                    radius: 2
                                    x: 0
                                    anchors.verticalCenter: parent.verticalCenter
                                    color: Th.ac
                                }
                                Row {
                                    x: 12
                                    spacing: 12
                                    anchors.verticalCenter: parent.verticalCenter
                                    Ico { n: pp[2]; sz: 18; col: parent.parent.on || pm.containsMouse ? Th.tx : Th.tx2; anchors.verticalCenter: parent.verticalCenter }
                                    Lbl { text: pp[1]; color: parent.parent.on || pm.containsMouse ? Th.tx : Th.tx2; font.pixelSize: Th.fs; font.weight: Font.Medium; anchors.verticalCenter: parent.verticalCenter }
                                }
                                Rectangle { anchors.fill: parent; radius: parent.radius; color: "transparent"; border.color: Th.ac; border.width: 2; visible: parent.activeFocus }
                                MouseArea { id: pm; anchors.fill: parent; hoverEnabled: true; cursorShape: Qt.PointingHandCursor; onClicked: S.setPage = modelData }
                            }
                        }
                    }
                }
            }
            Lbl {
                anchors.bottom: parent.bottom
                anchors.bottomMargin: 14
                x: 22
                text: "cord 0.1.0"
                color: Th.tx2
                font.pixelSize: Th.fsS
            }
        }
        Rectangle {
            id: pane
            Layout.fillWidth: true
            Layout.fillHeight: true
            radius: Th.r
            color: Th.bgc
            border.color: Th.ln
            readonly property real cw: Math.min(width - 72, 760)
            readonly property real cx: (width - cw) / 2
            Item {
                id: hd
                x: pane.cx
                y: 0
                width: pane.cw
                height: 96
                Column {
                    anchors.left: parent.left
                    anchors.right: cl.left
                    anchors.verticalCenter: parent.verticalCenter
                    anchors.verticalCenterOffset: 6
                    spacing: 4
                    Lbl { text: r.cur[1]; font.pixelSize: Th.fsXL + 2; font.weight: Font.Bold }
                    Lbl { width: parent.width; text: r.desc[r.cur[0]] || ""; color: Th.tx2 }
                }
                Column {
                    id: cl
                    anchors.right: parent.right
                    anchors.verticalCenter: parent.verticalCenter
                    anchors.verticalCenterOffset: 6
                    spacing: 4
                    Rectangle {
                        anchors.horizontalCenter: parent.horizontalCenter
                        width: 36
                        height: 36
                        radius: 18
                        color: xm.containsMouse ? Th.hov : "transparent"
                        border.color: xm.containsMouse ? Th.tx2 : Th.ln
                        border.width: 2
                        Behavior on border.color { ColorAnimation { duration: Th.mF } }
                        Ico { anchors.centerIn: parent; n: "x"; sz: 16; col: xm.containsMouse ? Th.tx : Th.tx2 }
                        MouseArea { id: xm; anchors.fill: parent; hoverEnabled: true; cursorShape: Qt.PointingHandCursor; onClicked: S.showSet = false }
                    }
                    Lbl { anchors.horizontalCenter: parent.horizontalCenter; text: "ESC"; color: Th.tx2; font.pixelSize: Th.fsT; font.weight: Font.DemiBold }
                }
            }
            Rectangle { x: pane.cx; y: hd.height; width: pane.cw; height: 1; color: Th.ln }
            ScrollView {
                id: sv
                anchors.top: hd.bottom
                anchors.topMargin: 1
                anchors.bottom: parent.bottom
                anchors.bottomMargin: 2
                anchors.left: parent.left
                anchors.right: parent.right
                contentWidth: availableWidth
                contentHeight: pg.height + 72
                clip: true
                Loader {
                    id: pg
                    x: pane.cx
                    y: 8
                    width: pane.cw
                    sourceComponent: ({ accounts: acP, privacy: privP, layout: layP, look: lookP, notif: notifP, feat: featP, voice: voiceP, mods: modP, keys: keyP, about: aboutP })[S.setPage] || acP
                    onLoaded: { pgA.restart(); sv.contentItem.contentY = 0 }
                    transform: Translate { id: pgT }
                    ParallelAnimation {
                        id: pgA
                        NumberAnimation { target: pg; property: "opacity"; from: 0; to: 1; duration: Th.mM; easing.type: Easing.OutCubic }
                        NumberAnimation { target: pgT; property: "y"; from: 10; to: 0; duration: Th.mS; easing.type: Easing.OutCubic }
                    }
                }
            }
        }
    }

    function vsave() {
        S.c.cset("voice.json", JSON.stringify({ ns: S.ui.ns !== false, vad: S.ui.vad || -50, ptt: !!S.ui.ptt, "in": S.ui.vin || "", out: S.ui.vout || "" }))
    }
    property var adev: ({ "in": [], out: [] })
    function adevs() {
        try { adev = JSON.parse(S.c.audio_devs()) } catch (e) { adev = { "in": [], out: [] } }
    }
    function dopts(l) { return [["", "System default"]].concat(l.map(d => [d.id, d.name])) }

    component Sub: Lbl { font.pixelSize: Th.fsS - 1; font.weight: Font.Bold; font.letterSpacing: 0.6; font.capitalization: Font.AllUppercase; color: Th.tx2; topPadding: 24; bottomPadding: 6 }
    component Row_: Item {
        id: rw
        property string t
        property string d
        width: parent ? parent.width : 400
        height: Math.max(60, rc.implicitHeight + 24)
        Rectangle { visible: rw.y > 0; anchors.top: parent.top; x: 16; width: parent.width - 32; height: 1; color: Th.ln }
        Column {
            id: rc
            x: 16
            anchors.verticalCenter: parent.verticalCenter
            spacing: 2
            Lbl { text: rw.t; font.weight: Font.Medium }
            Lbl { visible: rw.d !== ""; text: rw.d; color: Th.tx2; font.pixelSize: Th.fsS }
        }
    }
    component FOpt: Opt {
        property string k
        readonly property var fd: r.fdef(k)
        visible: !!fd
        t: fd ? fd.n : ""
        d: fd ? fd.d : ""
        tag: fd && fd.risk === "ban" ? "Ban risk" : ""
        on: S.f(k)
        onTog: v => S.setF(k, v)
    }
    component Group: Rectangle {
        id: gp
        default property alias content: grpC.data
        width: parent ? parent.width : 400
        height: grpC.implicitHeight
        radius: Th.r
        color: Th.bg
        border.color: Th.ln
        Column { id: grpC; width: parent.width }
    }
    component Opt: Rectangle {
        id: op
        property string t
        property string d
        property string tag: ""
        property alias on: tg.on
        signal tog(bool v)
        width: parent ? parent.width : 400
        height: Math.max(60, oc.implicitHeight + 24)
        color: om.containsMouse ? Th.a(Th.tx, 0.025) : "transparent"
        radius: Th.r
        HoverHandler { id: omh }
        readonly property alias om: omh
        Rectangle { visible: op.y > 0; anchors.top: parent.top; x: 16; width: parent.width - 32; height: 1; color: Th.ln }
        Column {
            id: oc
            x: 16
            anchors.verticalCenter: parent.verticalCenter
            width: parent.width - 100
            spacing: 2
            Row {
                spacing: 8
                Lbl { text: op.t; font.weight: Font.Medium; anchors.verticalCenter: parent.verticalCenter }
                Rectangle {
                    visible: op.tag !== ""
                    width: tgl.implicitWidth + 10
                    height: 16
                    radius: Th.rc
                    color: Th.a(Th.err, 0.16)
                    anchors.verticalCenter: parent.verticalCenter
                    Lbl { id: tgl; anchors.centerIn: parent; text: op.tag; color: Th.err; font.pixelSize: Th.fsP; font.weight: Font.DemiBold }
                }
            }
            Lbl { visible: op.d !== ""; text: op.d; color: Th.tx2; font.pixelSize: Th.fsS; wrapMode: Text.WrapAtWordBoundaryOrAnywhere; elide: Text.ElideNone; width: parent.width }
        }
        Tog { id: tg; label: op.t; anchors.right: parent.right; anchors.rightMargin: 16; anchors.verticalCenter: parent.verticalCenter; onToggled: v => op.tog(v) }
    }

    Component {
        id: acP
        Column {
            spacing: 8
            Rectangle {
                visible: S.secretBackend === "file"
                width: parent.width
                height: sw.implicitHeight + 20
                radius: Th.r
                color: Th.bg3
                border.color: Th.err
                Lbl { id: sw; x: 12; anchors.verticalCenter: parent.verticalCenter; width: parent.width - 24; wrapMode: Text.Wrap; elide: Text.ElideNone; text: "No system keyring was found. Account tokens are stored in a file in " + S.c.home() + " without encryption." }
            }
            AccTabs { id: atb; cur: Math.min(S.accIdx, S.cfgAccs.length) }
            Connections { target: atb; function onCurChanged() { S.accIdx = atb.cur } }
            Loader {
                id: avl
                width: parent.width
                active: atb.cur < S.cfgAccs.length
                visible: active
                transform: Translate { id: avt }
                sourceComponent: AccView { a: S.cfgAccs[Math.min(atb.cur, S.cfgAccs.length - 1)] }
                ParallelAnimation {
                    id: avA
                    NumberAnimation { target: avl; property: "opacity"; from: 0; to: 1; duration: Th.mM; easing.type: Easing.OutCubic }
                    NumberAnimation { target: avt; property: "x"; from: 14; to: 0; duration: Th.mM; easing.type: Easing.OutCubic }
                }
                Connections { target: S; function onAccIdxChanged() { avA.restart() } }
            }
            AccAdd { visible: atb.cur >= S.cfgAccs.length; onDone: { U.flash("Account added"); S.accIdx = Math.max(0, S.cfgAccs.length - 1) } }
            Item { visible: S.cfgAccs.some(a => a.base === "demo"); width: 1; height: 4 }
            Btn {
                visible: S.cfgAccs.some(a => a.base === "demo")
                text: "Exit demo"
                onClicked: U.exitDemo()
            }
            Sub { text: "Status" }
            Group {
                Row_ {
                    t: "Online status"
                    d: "Shown to friends on every account"
                    Seg {
                        anchors.right: parent.right
                        anchors.rightMargin: 12
                        anchors.verticalCenter: parent.verticalCenter
                        opts: [["On", "Online"], ["Idle", "Idle"], ["Dnd", "Do not disturb"], ["Off", "Invisible"]]
                        cur: S.ui.st || "On"
                        onPick: v => { S.setUi("st", v); S.c.set_st(v.toLowerCase(), "") }
                    }
                }
            }
        }
    }

    Component {
        id: layP
        Column {
            spacing: 8
            Lbl { text: "Pick a preset, flip blocks left or right, or drag them around in edit mode."; color: Th.tx2; wrapMode: Text.Wrap; elide: Text.ElideNone; width: parent.width }
            Sub { text: "Presets"; topPadding: 4 }
            Grid {
                id: pgr
                width: parent.width
                columns: 3
                spacing: 12
                Repeater {
                    model: Object.keys(L.presets)
                    Rectangle {
                        width: (pgr.width - 24) / 3
                        height: 132
                        radius: Th.r
                        color: Th.bg
                        border.color: pm2.containsMouse || activeFocus ? Th.ac : Th.ln
                        border.width: activeFocus ? 2 : 1
                        activeFocusOnTab: true
                        Accessible.role: Accessible.Button
                        Accessible.name: L.presets[modelData].name
                        Keys.onReturnPressed: S.preset(modelData)
                        Keys.onSpacePressed: S.preset(modelData)
                        LayPrev { x: 10; y: 10; width: parent.width - 20; height: 88; n: L.presets[modelData] }
                        Lbl { x: 12; y: 106; text: L.presets[modelData].name; font.pixelSize: Th.fsXS; font.weight: Font.DemiBold }
                        MouseArea { id: pm2; anchors.fill: parent; hoverEnabled: true; cursorShape: Qt.PointingHandCursor; onClicked: S.preset(modelData) }
                    }
                }
            }
            Sub { text: "Blocks" }
            Group {
            Repeater {
                model: S.allBlks().filter(b => b.id !== "chat")
                Rectangle {
                    id: br_
                    width: parent.width
                    height: 60
                    color: "transparent"
                    Rectangle { visible: br_.y > 0; anchors.top: parent.top; x: 16; width: parent.width - 32; height: 1; color: Th.ln }
                    Lbl { x: 16; anchors.verticalCenter: parent.verticalCenter; text: modelData.name; font.weight: Font.Medium }
                    Seg {
                        anchors.right: parent.right
                        anchors.rightMargin: 12
                        anchors.verticalCenter: parent.verticalCenter
                        opts: [["l", "Left"], ["r", "Right"], ["off", "Off"]]
                        cur: { void S.lay; return S.sideOf(modelData.id) }
                        onPick: v => {
                            if (v === "off") S.toggleBlk(modelData.id, false)
                            else if (!S.hasBlk(modelData.id)) S.setLay(L.ins(S.lay, L.find(S.lay, "chat") || [], v, { b: modelData.id, w: 248 }))
                            else S.sideBlk(modelData.id, v)
                        }
                    }
                }
            }
            }
            Sub { text: "Advanced" }
            Row {
                spacing: 8
                Btn { text: "Edit layout"; ico: "layout"; pri: true; onClicked: { S.showSet = false; S.edit = true } }
                Btn { text: "Copy layout"; ico: "copy"; onClicked: S.copy(JSON.stringify(S.lay)) }
                Btn { text: "Paste layout"; onClicked: { pst.visible = !pst.visible; r.pasteErr = "" } }
            }
            Column {
                id: pst
                visible: false
                spacing: 6
                function apply() {
                    let ok = false
                    try { ok = U.setLay(JSON.parse(pf.text)) } catch (e) { ok = false }
                    r.pasteErr = ok ? "" : "Invalid layout"
                    if (ok) { pst.visible = false; pf.text = "" }
                }
                Row {
                    spacing: 8
                    Fld { id: pf; width: 480; placeholderText: "Paste a shared layout"; onTextChanged: r.pasteErr = ""; onAccepted: pst.apply() }
                    Btn { text: "Apply"; pri: true; onClicked: pst.apply() }
                }
                Lbl { visible: r.pasteErr !== ""; text: r.pasteErr; color: Th.err; font.pixelSize: Th.fsS }
            }
        }
    }

    Component {
        id: lookP
        Column {
            spacing: 8
            Sub { text: "Theme"; topPadding: 4 }
            Group { Opt { t: "Follow system appearance"; d: "Switch between light and dark with macOS or your desktop"; on: !!S.ui.sys; onTog: v => { S.setUi("sys", v); S.loadTheme() } } }
            Flow {
                width: parent.width
                spacing: 10
                Repeater {
                    model: S.themes()
                    Rectangle {
                        width: 176
                        height: 128
                        radius: Th.r
                        color: Th.bg
                        property var cs: r.tcols(modelData)
                        Rectangle {
                            x: 10
                            y: 10
                            width: parent.width - 20
                            height: 72
                            radius: 8
                            color: parent.cs[0]
                            border.color: Th.a(Th.tx, 0.08)
                            Rectangle { x: 6; y: 6; width: 14; height: parent.height - 12; radius: 4; color: parent.parent.cs[1] }
                            Rectangle { x: 24; y: 6; width: 40; height: parent.height - 12; radius: 4; color: parent.parent.cs[1] }
                            Rectangle { x: 68; y: 6; width: parent.width - 74; height: parent.height - 12; radius: 4; color: parent.parent.cs[1]; opacity: 0.7 }
                            Rectangle { x: 28; y: 12; width: 28; height: 6; radius: 3; color: parent.parent.cs[2] }
                            Rectangle { x: 74; y: parent.height - 20; width: parent.width - 86; height: 8; radius: 4; color: Th.a(parent.parent.cs[2], 0.5) }
                        }
                        Rectangle {
                            visible: (S.ui.theme || "midnight") === modelData.id
                            anchors.right: parent.right
                            anchors.bottom: parent.bottom
                            anchors.margins: 12
                            width: 18
                            height: 18
                            radius: 9
                            color: Th.ac
                            Ico { anchors.centerIn: parent; n: "check"; sz: 12; col: Th.acTx }
                        }
                        border.color: (S.ui.theme || "midnight") === modelData.id ? Th.ac : tm.containsMouse ? Th.tx3 : Th.ln
                        border.width: (S.ui.theme || "midnight") === modelData.id ? 2 : 1
                        Behavior on border.color { ColorAnimation { duration: Th.mF } }
                        activeFocusOnTab: true
                        Accessible.role: Accessible.Button
                        Accessible.name: modelData.id
                        Keys.onReturnPressed: { S.setUi("theme", modelData.id); S.loadTheme() }
                        Keys.onSpacePressed: { S.setUi("theme", modelData.id); S.loadTheme() }
                        Lbl { x: 12; y: 96; width: parent.width - 48; text: modelData.id.charAt(0).toUpperCase() + modelData.id.slice(1); font.weight: Font.DemiBold }
                        Row {
                            visible: false
                            x: 12
                            y: 44
                            width: parent.width - 24
                            height: 14
                            Repeater {
                                model: parent.parent.cs
                                Rectangle {
                                    width: (parent.width) / 3
                                    height: 14
                                    color: modelData
                                    topLeftRadius: index === 0 ? 4 : 0
                                    bottomLeftRadius: index === 0 ? 4 : 0
                                    topRightRadius: index === 2 ? 4 : 0
                                    bottomRightRadius: index === 2 ? 4 : 0
                                    border.color: Th.ln
                                    border.width: 0
                                }
                            }
                        }
                        Rectangle { anchors.fill: parent; radius: parent.radius; color: "transparent"; border.color: Th.ac; border.width: 2; visible: parent.activeFocus }
                        MouseArea { id: tm; anchors.fill: parent; hoverEnabled: true; cursorShape: Qt.PointingHandCursor; onClicked: { S.setUi("theme", modelData.id); S.loadTheme() } }
                    }
                }
            }
            Lbl { text: "More themes: Advanced, Mods and themes."; color: Th.tx2; font.pixelSize: Th.fsS; wrapMode: Text.Wrap; elide: Text.ElideNone; width: parent.width }
            Sub { text: "Accent" }
            Row {
                spacing: 6
                Repeater {
                    model: ["#a2b0e0", "#8fb3ff", "#6ea8fe", "#5fc4d6", "#7dd3c0", "#9aa7ff", "#b69cff", "#f2c9da"]
                    Rectangle {
                        id: sw
                        readonly property bool on: String(Th.ac).toLowerCase() === modelData
                        width: 28
                        height: 28
                        radius: 14
                        color: "transparent"
                        border.color: on || activeFocus ? Th.tx : "transparent"
                        border.width: 2
                        activeFocusOnTab: true
                        Accessible.role: Accessible.Button
                        Accessible.name: "Accent " + modelData
                        Keys.onReturnPressed: { S.setUi("ac", modelData); S.loadTheme() }
                        Keys.onSpacePressed: { S.setUi("ac", modelData); S.loadTheme() }
                        Rectangle { anchors.centerIn: parent; width: 20; height: 20; radius: 10; color: modelData }
                        MouseArea { anchors.fill: parent; cursorShape: Qt.PointingHandCursor; onClicked: { S.setUi("ac", modelData); S.loadTheme() } }
                    }
                }
                Btn { visible: !!S.ui.ac; text: "Reset"; height: 28; flat: true; onClicked: { S.setUi("ac", ""); S.loadTheme() } }
            }
            Sub { text: "Display" }
            Group {
                Row_ { t: "Text size"; d: "Size of messages and the interface"; Seg { anchors.right: parent.right; anchors.rightMargin: 12; anchors.verticalCenter: parent.verticalCenter; opts: [["12", "12"], ["13", "13"], ["14", "14"], ["15", "15"], ["16", "16"], ["18", "18"]]; cur: String(S.ui.fs || Th.fs); onPick: v => { S.setUi("fs", parseInt(v)); S.loadTheme() } } }
                Row_ { t: "Density"; d: "Space between rows and messages"; Seg { anchors.right: parent.right; anchors.rightMargin: 12; anchors.verticalCenter: parent.verticalCenter; opts: [["0.85", "Compact"], ["1", "Default"], ["1.15", "Spacious"]]; cur: String(S.ui.dens || 1); onPick: v => { S.setUi("dens", parseFloat(v)); S.loadTheme() } } }
            }
            Sub { text: "Details" }
            Group {
                Opt { t: "Show network badges"; d: "Marks for Discord, Matrix and Stoat on servers, avatars and DMs"; on: S.ui.badges !== false; onTog: v => S.setUi("badges", v) }
                Opt { t: "Avatar decorations"; d: "Show Discord avatar decorations around profile pictures"; on: S.ui.decos !== false; onTog: v => S.setUi("decos", v) }
                Opt { t: "Nameplates"; d: "Show Discord nameplates behind names in the member and DM lists"; on: S.ui.plates !== false; onTog: v => S.setUi("plates", v) }
                Opt { t: "Display name styles"; d: "Show colors and gradients people picked for their display name"; on: S.ui.nstyle !== false; onTog: v => S.setUi("nstyle", v) }
                Opt { t: "Profile effects"; d: "Play Discord profile effects when you open a profile"; on: S.ui.peff !== false; onTog: v => S.setUi("peff", v) }
                Repeater { model: r.fcat.look; FOpt { k: modelData } }
            }
        }
    }

    Component {
        id: featP
        Column {
            spacing: 0
            Lbl { width: parent.width; text: "Client mods on Discord can get an account banned, see the Ban risk tags."; color: Th.tx2; font.pixelSize: Th.fsS; bottomPadding: 4 }
            Sub { text: "Chat"; topPadding: 4 }
            Group { Repeater { model: r.fcat.chat; FOpt { k: modelData } } }
            Sub { text: "Media" }
            Group { Repeater { model: r.fcat.media; FOpt { k: modelData } } }
            Sub { text: "Other"; visible: r.fother().length > 0 }
            Group { visible: r.fother().length > 0; Repeater { model: r.fother(); FOpt { k: modelData } } }
            Lbl { width: parent.width; topPadding: 16; text: "Privacy, notification, call and appearance tweaks live on their own pages."; color: Th.tx2; font.pixelSize: Th.fsS; wrapMode: Text.Wrap; elide: Text.ElideNone }
        }
    }

    Component {
        id: privP
        Column {
            spacing: 0
            Sub { text: "Streaming"; topPadding: 4 }
            Group { Opt { t: "Streamer mode"; d: "Hide emails and silence desktop notifications while you stream or share your screen"; on: !!S.ui.streamer; onTog: v => S.setUi("streamer", v) } }
            Sub { text: "Activity" }
            Group { Repeater { model: r.fcat.privacy; FOpt { k: modelData } } }
            Sub { text: "Account storage" }
            Group {
                Row_ {
                    t: "Token storage"
                    d: S.secretBackend === "file" ? "No system keyring found, tokens are stored unencrypted in a file" : "Tokens are kept in the system keyring"
                    Rectangle {
                        anchors.right: parent.right
                        anchors.rightMargin: 16
                        anchors.verticalCenter: parent.verticalCenter
                        width: kt.implicitWidth + 16
                        height: 24
                        radius: 12
                        color: S.secretBackend === "file" ? Th.a(Th.err, 0.18) : Th.a(Th.ok, 0.18)
                        Lbl { id: kt; anchors.centerIn: parent; text: S.secretBackend === "file" ? "Unencrypted" : "Keyring"; color: S.secretBackend === "file" ? Th.err : Th.ok; font.pixelSize: Th.fsS; font.weight: Font.DemiBold }
                    }
                }
            }
        }
    }

    Component {
        id: notifP
        Column {
            spacing: 0
            Sub { text: "Desktop"; topPadding: 4 }
            Group {
                Opt { t: "Desktop notifications"; d: "Alerts while the window is not focused"; on: S.ui.notif !== false; onTog: v => S.setUi("notif", v) }
                Row_ { t: "Notify me for"; d: "Which messages raise an alert"; Seg { anchors.right: parent.right; anchors.rightMargin: 12; anchors.verticalCenter: parent.verticalCenter; opts: [["both", "DMs and mentions"], ["dm", "DMs only"], ["men", "Mentions only"]]; cur: S.ui.nmode || "both"; onPick: v => S.setUi("nmode", v) } }
                Opt { t: "Message preview"; d: "Show the message text in the alert, otherwise only the sender"; on: S.ui.npv !== false; onTog: v => S.setUi("npv", v) }
            }
            Sub { text: "Sounds" }
            Group {
                Opt { t: "Message sound"; d: "Play a short chime for new alerts"; on: S.ui.nsnd !== false; onTog: v => S.setUi("nsnd", v) }
                Row_ { t: "Try it"; d: "Plays the message sound once"; Btn { anchors.right: parent.right; anchors.rightMargin: 12; anchors.verticalCenter: parent.verticalCenter; text: "Play"; height: 30; onClicked: { S.ping.stop(); S.ping.play() } } }
            }
            Sub { text: "Relationships" }
            Group { Repeater { model: r.fcat.notif; FOpt { k: modelData } } }
        }
    }

    Component {
        id: voiceP
        Column {
            spacing: 8
            Component.onCompleted: r.adevs()
            Sub { text: "Devices"; topPadding: 4 }
            Group {
                Row_ { t: "Microphone"; d: "Input device for calls"; Sel { anchors.right: parent.right; anchors.rightMargin: 12; anchors.verticalCenter: parent.verticalCenter; opts: r.dopts(r.adev["in"]); cur: S.ui.vin || ""; onPick: v => { S.setUi("vin", v); r.vsave() } } }
                Row_ { t: "Speakers"; d: "Output device for calls"; Sel { anchors.right: parent.right; anchors.rightMargin: 12; anchors.verticalCenter: parent.verticalCenter; opts: r.dopts(r.adev.out); cur: S.ui.vout || ""; onPick: v => { S.setUi("vout", v); r.vsave() } } }
            }
            Lbl { width: parent.width; text: "Device changes apply the next time you join a call."; color: Th.tx2; font.pixelSize: Th.fsS }
            Sub { text: "Input" }
            Group {
            Opt { t: "Noise suppression"; d: "Filter background noise from your microphone"; on: S.ui.ns !== false; onTog: v => { S.setUi("ns", v); r.vsave() } }
            Opt { t: "Push to talk"; d: "Only send audio while the key is held (" + (Qt.platform.os === "osx" ? "⌃`" : "Ctrl+`") + ", while cord is focused)"; on: !!S.ui.ptt; onTog: v => { S.setUi("ptt", v); r.vsave() } }
            }
            Group {
                Row_ {
                    t: "Input sensitivity"
                    d: "Microphone level that counts as speaking"
                    Sld {
                        anchors.right: parent.right
                        anchors.rightMargin: 16
                        anchors.verticalCenter: parent.verticalCenter
                        width: 280
                        lab: true
                        unit: " dB"
                        from: -80
                        to: -10
                        value: S.ui.vad || -50
                        onMoved: { S.setUi("vad", value); r.vsave() }
                    }
                }
            }
            Sub { text: "Screen share" }
            Group {
                Row_ { t: "Resolution"; d: "Capture size of your stream"; Seg { anchors.right: parent.right; anchors.rightMargin: 12; anchors.verticalCenter: parent.verticalCenter; opts: [["720", "720p"], ["1080", "1080p"], ["1440", "1440p"], ["2160", "4K"]]; cur: String(S.ui.sh || 1080); onPick: v => S.setUi("sh", parseInt(v)) } }
                Row_ { t: "Frame rate"; d: "Frames per second of your stream"; Seg { anchors.right: parent.right; anchors.rightMargin: 12; anchors.verticalCenter: parent.verticalCenter; opts: [["30", "30"], ["60", "60"], ["120", "120"], ["144", "144"]]; cur: String(S.ui.sfps || 60); onPick: v => S.setUi("sfps", parseInt(v)) } }
            }
            Group {
            FOpt { k: "streamq" }
            Opt { t: "Share audio"; d: Qt.platform.os === "osx" ? "System audio capture is not available on macOS yet" : "Send application and system audio with your stream"; enabled: Qt.platform.os !== "osx"; opacity: enabled ? 1 : 0.5; on: Qt.platform.os !== "osx" && S.ui.ssnd !== false; onTog: v => S.setUi("ssnd", v) }
            }
            Lbl { visible: !S.f("streamq"); text: "Turn on Stream quality unlock to go above 720p30."; color: Th.tx2; font.pixelSize: Th.fsS }
            Sub { text: "Calls" }
            Group { FOpt { k: "callTimer" } }
        }
    }

    Component {
        id: modP
        Column {
            spacing: 8
            Lbl {
                width: parent.width
                wrapMode: Text.Wrap
                elide: Text.ElideNone
                color: Th.tx2
                text: "A mod is a folder in " + S.c.home() + "/mods with a main.qml. It can import Cord, register blocks with S.regBlk, and hook render, send and ev. A block named like a built-in one in " + S.c.home() + "/blocks replaces it."
            }
            Lbl { width: parent.width; wrapMode: Text.Wrap; elide: Text.ElideNone; color: Th.tx2; text: "A theme is a .json file in " + S.c.home() + "/themes. It shows up in Appearance and reloads live." }
            Row {
                spacing: 8
                Btn { text: "Open config folder"; onClicked: Qt.openUrlExternally("file://" + S.c.home()) }
                Btn { text: "Reload"; onClicked: { S.scanMods(); S.loadTheme() } }
            }
            Sub { text: "Installed" }
            Group {
            Row_ { visible: S.mods.length === 0; t: "No mods installed"; d: "Add a folder with a main.qml to the mods folder, then press Reload" }
            Repeater {
                model: S.mods
                Opt {
                    t: modelData.id
                    d: modelData.url.replace("file://", "").replace(/\/main\.qml$/, "")
                    on: modelData.on
                    onTog: v => { const m = Object.assign({}, S.ui.mods || {}); m[modelData.id] = v; S.setUi("mods", m); S.scanMods() }
                }
            }
            }
        }
    }

    function seqOf(e) {
        const k = e.key
        if ([Qt.Key_Shift, Qt.Key_Control, Qt.Key_Alt, Qt.Key_Meta, Qt.Key_AltGr].includes(k)) return ""
        let n = ""
        if (k >= Qt.Key_A && k <= Qt.Key_Z) n = String.fromCharCode(k)
        else if (k >= Qt.Key_0 && k <= Qt.Key_9) n = String.fromCharCode(k)
        else if (k >= Qt.Key_F1 && k <= Qt.Key_F12) n = "F" + (k - Qt.Key_F1 + 1)
        else n = ({ [Qt.Key_Comma]: ",", [Qt.Key_Period]: ".", [Qt.Key_Slash]: "/", [Qt.Key_Semicolon]: ";", [Qt.Key_Minus]: "-", [Qt.Key_Equal]: "=", [Qt.Key_Up]: "Up", [Qt.Key_Down]: "Down", [Qt.Key_Left]: "Left", [Qt.Key_Right]: "Right", [Qt.Key_Space]: "Space", [Qt.Key_Return]: "Return", [Qt.Key_Tab]: "Tab", [Qt.Key_PageUp]: "PgUp", [Qt.Key_PageDown]: "PgDown", [Qt.Key_Home]: "Home", [Qt.Key_End]: "End" })[k] || ""
        if (!n) return ""
        const m = e.modifiers
        const meta = Qt.platform.os === "osx" ? Qt.MetaModifier : Qt.ControlModifier
        return (m & Qt.ControlModifier ? "Ctrl+" : "") + (m & Qt.AltModifier ? "Alt+" : "") + (m & Qt.ShiftModifier ? "Shift+" : "") + (Qt.platform.os === "osx" && (m & Qt.MetaModifier) ? "Meta+" : "") + n
    }
    function prettyKey(q) {
        const mac = Qt.platform.os === "osx"
        return q.replace("Ctrl+", mac ? "\u2318" : "Ctrl+").replace("Alt+", mac ? "\u2325" : "Alt+").replace("Shift+", mac ? "\u21e7" : "Shift+").replace("Meta+", mac ? "\u2303" : "Meta+").replace("Escape", "Esc")
    }

    Component {
        id: keyP
        Column {
            spacing: 8
            Lbl { width: parent.width; text: "Click a shortcut, then press the new keys. Esc cancels, Backspace resets."; color: Th.tx2; font.pixelSize: Th.fsS; wrapMode: Text.Wrap; elide: Text.ElideNone }
            Grp {
                Repeater {
                    model: S.keyDefs
                    Item {
                        id: kr_
                        property bool listen: false
                        property string clash: ""
                        width: parent.width
                        height: 60
                        Rectangle { visible: index > 0; anchors.top: parent.top; x: 16; width: parent.width - 32; height: 1; color: Th.ln }
                        Lbl { x: 16; anchors.verticalCenter: parent.verticalCenter; text: modelData.t; font.weight: Font.Medium }
                        Lbl { visible: kr_.clash !== ""; anchors.right: kb.left; anchors.rightMargin: 12; anchors.verticalCenter: parent.verticalCenter; text: kr_.clash; color: Th.err; font.pixelSize: Th.fsS }
                        Rectangle {
                            id: kb
                            anchors.right: parent.right
                            anchors.rightMargin: 16
                            anchors.verticalCenter: parent.verticalCenter
                            width: Math.max(72, kl.implicitWidth + 24)
                            height: 30
                            radius: Th.rc + 2
                            color: kr_.listen ? Th.a(Th.ac, 0.16) : km.containsMouse ? Th.hov : Th.bg4
                            border.color: kr_.listen ? Th.ac : Th.a(Th.tx, 0.08)
                            Behavior on color { ColorAnimation { duration: Th.mF } }
                            Lbl { id: kl; anchors.centerIn: parent; text: kr_.listen ? "Press keys" : r.prettyKey(S.key(modelData.id)); color: kr_.listen ? Th.ac : Th.tx; font.family: Th.mono; font.pixelSize: Th.fsS }
                            MouseArea { id: km; anchors.fill: parent; hoverEnabled: true; cursorShape: Qt.PointingHandCursor; onClicked: { kr_.clash = ""; kr_.listen = true; kb.forceActiveFocus() } }
                            activeFocusOnTab: true
                            onActiveFocusChanged: if (!activeFocus) kr_.listen = false
                            Keys.onPressed: e => {
                                if (!kr_.listen) return
                                e.accepted = true
                                if (e.key === Qt.Key_Escape) { kr_.listen = false; return }
                                if (e.key === Qt.Key_Backspace || e.key === Qt.Key_Delete) { S.setKey(modelData.id, ""); kr_.listen = false; return }
                                const q = r.seqOf(e)
                                if (!q) return
                                const o = S.keyDefs.find(x => x.id !== modelData.id && S.key(x.id) === q)
                                if (o) { kr_.clash = "Used by " + o.t; kr_.listen = false; return }
                                S.setKey(modelData.id, q)
                                kr_.listen = false
                            }
                        }
                    }
                }
            }
            Sub { text: "Fixed" }
            Grp {
                Repeater {
                    model: [["Up", "Edit your last message"], ["Esc", "Close popups and go back"], [(Qt.platform.os === "osx" ? "\u2303`" : "Ctrl+`"), "Push to talk"]]
                    Item {
                        width: parent.width
                        height: 60
                        Rectangle { visible: index > 0; anchors.top: parent.top; x: 16; width: parent.width - 32; height: 1; color: Th.ln }
                        Lbl { x: 16; anchors.verticalCenter: parent.verticalCenter; text: modelData[1] }
                        Rectangle {
                            anchors.right: parent.right
                            anchors.rightMargin: 16
                            anchors.verticalCenter: parent.verticalCenter
                            width: fk.implicitWidth + 16
                            height: 26
                            radius: Th.rc
                            color: Th.bg4
                            border.color: Th.a(Th.tx, 0.08)
                            Lbl { id: fk; anchors.centerIn: parent; text: modelData[0]; font.family: Th.mono; font.pixelSize: Th.fsS }
                        }
                    }
                }
            }
            Btn { visible: !!S.ui.keys && Object.keys(S.ui.keys).length > 0; text: "Reset all shortcuts"; out: true; onClicked: S.setUi("keys", {}) }
        }
    }

    Component {
        id: aboutP
        Column {
            spacing: 8
            Group {
                Item {
                    width: parent.width
                    height: 96
                    Rectangle { id: lg; x: 16; anchors.verticalCenter: parent.verticalCenter; width: 56; height: 56; radius: 16; color: Th.ac; Lbl { anchors.centerIn: parent; text: "c"; color: Th.acTx; font.pixelSize: 30; font.weight: Font.Bold } }
                    Column {
                        anchors.left: lg.right
                        anchors.leftMargin: 16
                        anchors.verticalCenter: parent.verticalCenter
                        spacing: 2
                        Lbl { text: "cord"; font.pixelSize: Th.fsL + 2; font.weight: Font.Bold }
                        Lbl { text: "Version 0.1.0 · GPL-3.0-or-later"; color: Th.tx2 }
                        Lbl { text: "A native client for Discord, Matrix and Stoat."; color: Th.tx2 }
                    }
                }
            }
            Item { width: 1; height: 4 }
            Btn { text: "Source code"; ico: "link"; onClicked: Qt.openUrlExternally("https://github.com/Silvertree2010/cord") }
            Lbl {
                width: parent.width
                wrapMode: Text.Wrap
                elide: Text.ElideNone
                topPadding: 12
                color: Th.tx3
                font.pixelSize: Th.fsS
                text: "Using third party clients on Discord is against Discord's Terms of Service. Use at your own risk."
            }
        }
    }
}
