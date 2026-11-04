import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import Cord
import "Lay.js" as L

Rectangle {
    id: r
    color: Th.bg
    MouseArea { anchors.fill: parent }
    readonly property var pages: [
        ["accounts", "Accounts", "users"], ["layout", "Layout", "layout"], ["look", "Appearance", "eye"],
        ["feat", "Features", "grid"], ["voice", "Voice and video", "mic"], ["mods", "Mods and themes", "package"],
        ["keys", "Shortcuts", "keyboard"], ["about", "About", "info"]
    ]
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

    RowLayout {
        anchors.fill: parent
        spacing: 0
        Rectangle {
            Layout.fillHeight: true
            Layout.preferredWidth: 240
            color: Th.bg2
            Column {
                anchors.fill: parent
                anchors.topMargin: 48
                anchors.leftMargin: 12
                anchors.rightMargin: 12
                spacing: 2
                Lbl { text: "Settings"; color: Th.tx2; font.pixelSize: Th.fsS; font.weight: Font.DemiBold; leftPadding: 10; bottomPadding: 6 }
                Repeater {
                    model: r.pages
                    Rectangle {
                        width: parent.width
                        height: 36
                        radius: Th.rs
                        color: S.setPage === modelData[0] ? Th.sel : pm.containsMouse ? Th.hov : "transparent"
                        activeFocusOnTab: true
                        Accessible.role: Accessible.Button
                        Accessible.name: modelData[1]
                        Keys.onReturnPressed: S.setPage = modelData[0]
                        Keys.onSpacePressed: S.setPage = modelData[0]
                        Row {
                            x: 10
                            spacing: 10
                            anchors.verticalCenter: parent.verticalCenter
                            Ico { n: modelData[2]; sz: 16; col: S.setPage === modelData[0] ? Th.tx : Th.tx2; anchors.verticalCenter: parent.verticalCenter }
                            Lbl { text: modelData[1]; color: S.setPage === modelData[0] ? Th.tx : Th.tx2; anchors.verticalCenter: parent.verticalCenter }
                        }
                        Rectangle { anchors.fill: parent; radius: parent.radius; color: "transparent"; border.color: Th.ac; border.width: 2; visible: parent.activeFocus }
                        MouseArea { id: pm; anchors.fill: parent; hoverEnabled: true; cursorShape: Qt.PointingHandCursor; onClicked: S.setPage = modelData[0] }
                    }
                }
            }
        }
        Item {
            id: pane
            Layout.fillWidth: true
            Layout.fillHeight: true
            readonly property real cw: Math.min(width - 80, 720)
            readonly property real cx: (width - cw) / 2
            Item {
                id: hd
                x: pane.cx
                y: 0
                width: pane.cw
                height: 72
                Lbl { anchors.left: parent.left; anchors.verticalCenter: parent.verticalCenter; anchors.verticalCenterOffset: 6; text: r.cur[1]; font.pixelSize: Th.fsXL; font.weight: Font.Bold }
                IBtn { anchors.right: parent.right; anchors.verticalCenter: parent.verticalCenter; anchors.verticalCenterOffset: 6; n: "x"; sz: 36; tip: "Close  (Esc)"; onClicked: S.showSet = false }
            }
            ScrollView {
                id: sv
                anchors.top: hd.bottom
                anchors.bottom: parent.bottom
                anchors.left: parent.left
                anchors.right: parent.right
                contentWidth: availableWidth
                contentHeight: pg.height + 96
                clip: true
                Loader {
                    id: pg
                    x: pane.cx
                    width: pane.cw
                    sourceComponent: ({ accounts: acP, layout: layP, look: lookP, feat: featP, voice: voiceP, mods: modP, keys: keyP, about: aboutP })[S.setPage] || acP
                }
            }
        }
    }

    component Sub: Lbl { font.pixelSize: Th.fs; font.weight: Font.DemiBold; topPadding: 20; bottomPadding: 8 }
    component Opt: Rectangle {
        id: op
        property string t
        property string d
        property string tag: ""
        property alias on: tg.on
        signal tog(bool v)
        width: parent ? parent.width : 400
        height: Math.max(56, oc.implicitHeight + 20)
        color: "transparent"
        Rectangle { anchors.bottom: parent.bottom; width: parent.width; height: 1; color: Th.ln }
        Column {
            id: oc
            anchors.verticalCenter: parent.verticalCenter
            width: parent.width - 70
            spacing: 2
            Row {
                spacing: 8
                Lbl { text: op.t; font.weight: Font.Medium; anchors.verticalCenter: parent.verticalCenter }
                Rectangle {
                    visible: op.tag !== ""
                    width: tgl.implicitWidth + 10
                    height: 16
                    radius: 3
                    color: Th.a(Th.err, 0.16)
                    anchors.verticalCenter: parent.verticalCenter
                    Lbl { id: tgl; anchors.centerIn: parent; text: op.tag; color: Th.err; font.pixelSize: 10; font.weight: Font.DemiBold }
                }
            }
            Lbl { visible: op.d !== ""; text: op.d; color: Th.tx2; font.pixelSize: Th.fsS; wrapMode: Text.WrapAtWordBoundaryOrAnywhere; elide: Text.ElideNone; width: parent.width }
        }
        Tog { id: tg; label: op.t; anchors.right: parent.right; anchors.verticalCenter: parent.verticalCenter; onToggled: v => op.tog(v) }
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
            Repeater {
                model: S.cfgAccs
                Rectangle {
                    id: ar
                    width: parent.width
                    height: 64
                    radius: Th.r
                    color: Th.bg2
                    property var a: S.accs[modelData.id]
                    property string who: a ? (a.me.disp || a.me.name) : modelData.base === "demo" ? "Demo account" : "Not connected"
                    Av { id: aa; x: 14; anchors.verticalCenter: parent.verticalCenter; sz: 38; src: parent.a ? (parent.a.me.av || "") : ""; name: parent.a ? parent.a.me.name : "?" }
                    Column {
                        anchors.left: aa.right
                        anchors.leftMargin: 12
                        anchors.verticalCenter: parent.verticalCenter
                        Row {
                            spacing: 8
                            Lbl { text: ar.who; font.weight: Font.DemiBold; anchors.verticalCenter: parent.verticalCenter }
                            Rectangle {
                                visible: modelData.base === "demo"
                                width: dl.implicitWidth + 12
                                height: 16
                                radius: 4
                                color: Th.bg4
                                anchors.verticalCenter: parent.verticalCenter
                                Lbl { id: dl; anchors.centerIn: parent; text: "Demo"; color: Th.tx2; font.pixelSize: 10; font.weight: Font.DemiBold }
                            }
                        }
                        Row {
                            spacing: 8
                            Badge { sv: modelData.svc; txt: true; anchors.verticalCenter: parent.verticalCenter }
                            Rectangle { width: 8; height: 8; radius: 4; color: S.up.indexOf(modelData.id) >= 0 ? Th.ok : Th.tx3; anchors.verticalCenter: parent.verticalCenter }
                            Lbl { text: S.up.indexOf(modelData.id) >= 0 ? "Online" : "Offline"; color: Th.tx3; font.pixelSize: 11; anchors.verticalCenter: parent.verticalCenter }
                        }
                    }
                    Btn {
                        anchors.right: parent.right
                        anchors.rightMargin: 14
                        anchors.verticalCenter: parent.verticalCenter
                        text: modelData.base === "demo" ? "Remove" : "Log out"
                        gh: true
                        onClicked: U.ask({
                            title: modelData.base === "demo" ? "Remove demo account" : "Log out",
                            body: (modelData.base === "demo" ? "Remove " : "Log out of ") + ar.who + " on " + Th.svcName(S.tag(modelData.svc)) + "?",
                            ok: modelData.base === "demo" ? "Remove" : "Log out",
                            danger: true,
                            run: () => S.c.rm_acc(modelData.id)
                        })
                    }
                }
            }
            Item { visible: S.cfgAccs.some(a => a.base === "demo"); width: 1; height: 8 }
            Btn {
                visible: S.cfgAccs.some(a => a.base === "demo")
                text: "Exit demo"
                onClicked: U.exitDemo()
            }
            Sub { text: "Add account" }
            AccAdd { onDone: U.flash("Account added") }
        }
    }

    Component {
        id: layP
        Column {
            spacing: 8
            Lbl { text: "Pick a preset, flip blocks left or right, or drag them around in edit mode."; color: Th.tx2; wrapMode: Text.Wrap; elide: Text.ElideNone; width: parent.width }
            Sub { text: "Presets" }
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
                        color: Th.bg2
                        border.color: pm2.containsMouse || activeFocus ? Th.ac : Th.ln
                        border.width: activeFocus ? 2 : 1
                        activeFocusOnTab: true
                        Accessible.role: Accessible.Button
                        Accessible.name: L.presets[modelData].name
                        Keys.onReturnPressed: S.preset(modelData)
                        Keys.onSpacePressed: S.preset(modelData)
                        LayPrev { x: 10; y: 10; width: parent.width - 20; height: 88; n: L.presets[modelData] }
                        Lbl { x: 12; y: 106; text: L.presets[modelData].name; font.pixelSize: 12; font.weight: Font.DemiBold }
                        MouseArea { id: pm2; anchors.fill: parent; hoverEnabled: true; cursorShape: Qt.PointingHandCursor; onClicked: S.preset(modelData) }
                    }
                }
            }
            Sub { text: "Blocks" }
            Repeater {
                model: S.allBlks().filter(b => b.id !== "chat")
                Rectangle {
                    width: parent.width
                    height: 52
                    color: "transparent"
                    Rectangle { anchors.bottom: parent.bottom; width: parent.width; height: 1; color: Th.ln }
                    Lbl { anchors.verticalCenter: parent.verticalCenter; text: modelData.name; font.weight: Font.Medium }
                    Seg {
                        anchors.right: parent.right
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
            Sub { text: "Theme" }
            Flow {
                width: parent.width
                spacing: 10
                Repeater {
                    model: S.themes()
                    Rectangle {
                        width: 150
                        height: 72
                        radius: Th.r
                        color: Th.bg2
                        property var cs: r.tcols(modelData)
                        border.color: (S.ui.theme || "graphite") === modelData.id ? Th.ac : tm.containsMouse ? Th.tx3 : Th.ln
                        border.width: 1
                        activeFocusOnTab: true
                        Accessible.role: Accessible.Button
                        Accessible.name: modelData.id
                        Keys.onReturnPressed: { S.setUi("theme", modelData.id); S.loadTheme() }
                        Keys.onSpacePressed: { S.setUi("theme", modelData.id); S.loadTheme() }
                        Lbl { x: 12; y: 12; width: parent.width - 24; text: modelData.id.charAt(0).toUpperCase() + modelData.id.slice(1) }
                        Row {
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
            Lbl { text: "Drop .json themes into " + S.c.home() + "/themes. Changes load live."; color: Th.tx2; font.pixelSize: Th.fsS; wrapMode: Text.Wrap; elide: Text.ElideNone; width: parent.width }
            Sub { text: "Accent" }
            Row {
                spacing: 6
                Repeater {
                    model: ["#8fb3ff", "#6ea8fe", "#5fc4d6", "#7dd3c0", "#9aa7ff", "#b69cff"]
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
            Sub { text: "Text size" }
            Seg { opts: [["12", "12"], ["13", "13"], ["14", "14"], ["15", "15"], ["16", "16"], ["18", "18"]]; cur: String(S.ui.fs || 14); onPick: v => { S.setUi("fs", parseInt(v)); S.loadTheme() } }
            Sub { text: "Density" }
            Seg { opts: [["0.85", "Compact"], ["1", "Default"], ["1.15", "Spacious"]]; cur: String(S.ui.dens || 1); onPick: v => { S.setUi("dens", parseFloat(v)); S.loadTheme() } }
            Sub { text: "Notifications" }
            Opt { t: "Desktop notifications"; d: "Direct messages and mentions while the window is not focused"; on: S.ui.notif !== false; onTog: v => S.setUi("notif", v) }
            Sub { text: "Network badges" }
            Opt { t: "Show network badges"; d: "Marks for Discord, Matrix and Stoat on servers, avatars and DMs"; on: S.ui.badges !== false; onTog: v => S.setUi("badges", v) }
        }
    }

    Component {
        id: featP
        Column {
            spacing: 0
            Lbl { width: parent.width; text: "Built-in client mods. Each one can be turned off."; color: Th.tx2; bottomPadding: 4 }
            Lbl { width: parent.width; text: "Client mods on Discord can get an account banned, see the Ban risk tags."; color: Th.tx3; font.pixelSize: Th.fsS; bottomPadding: 10 }
            Repeater {
                model: S.featDefs
                Opt { t: modelData.n; d: modelData.d; tag: modelData.risk === "ban" ? "Ban risk" : ""; on: S.f(modelData.k); onTog: v => S.setF(modelData.k, v) }
            }
        }
    }

    Component {
        id: voiceP
        Column {
            spacing: 8
            Opt { t: "Noise suppression"; d: "Filter background noise from your microphone"; on: S.ui.ns !== false; onTog: v => { S.setUi("ns", v); S.c.cset("voice.json", JSON.stringify({ ns: v, vad: S.ui.vad || -50, ptt: !!S.ui.ptt })) } }
            Opt { t: "Push to talk"; d: "Only send audio while the key is held (" + (Qt.platform.os === "osx" ? "⌃`" : "Ctrl+`") + ", while cord is focused)"; on: !!S.ui.ptt; onTog: v => { S.setUi("ptt", v); S.c.cset("voice.json", JSON.stringify({ ns: S.ui.ns !== false, vad: S.ui.vad || -50, ptt: v })) } }
            Sub { text: "Input sensitivity" }
            Sld {
                width: parent.width
                lab: true
                unit: " dB"
                from: -80
                to: -10
                value: S.ui.vad || -50
                onMoved: { S.setUi("vad", value); S.c.cset("voice.json", JSON.stringify({ ns: S.ui.ns !== false, vad: value, ptt: !!S.ui.ptt })) }
            }
            Sub { text: "Screen share" }
            Row {
                spacing: 24
                Column {
                    spacing: 6
                    Lbl { text: "Resolution"; color: Th.tx2; font.pixelSize: Th.fsS }
                    Seg { opts: [["720", "720p"], ["1080", "1080p"], ["1440", "1440p"], ["2160", "4K"]]; cur: String(S.ui.sh || 1080); onPick: v => S.setUi("sh", parseInt(v)) }
                }
                Column {
                    spacing: 6
                    Lbl { text: "Frame rate"; color: Th.tx2; font.pixelSize: Th.fsS }
                    Seg { opts: [["30", "30"], ["60", "60"], ["120", "120"], ["144", "144"]]; cur: String(S.ui.sfps || 60); onPick: v => S.setUi("sfps", parseInt(v)) }
                }
            }
            Opt { t: "Share audio"; d: Qt.platform.os === "osx" ? "System audio capture is not available on macOS yet" : "Send application and system audio with your stream"; enabled: Qt.platform.os !== "osx"; opacity: enabled ? 1 : 0.5; on: Qt.platform.os !== "osx" && S.ui.ssnd !== false; onTog: v => S.setUi("ssnd", v) }
            Lbl { visible: !S.f("streamq"); text: "Turn on Stream quality unlock in Features to go above 720p30."; color: Th.tx2; font.pixelSize: Th.fsS }
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
            Row {
                spacing: 8
                Btn { text: "Open config folder"; onClicked: Qt.openUrlExternally("file://" + S.c.home()) }
                Btn { text: "Reload"; onClicked: { S.scanMods(); S.loadTheme() } }
            }
            Sub { text: "Installed" }
            Repeater {
                model: S.mods
                Opt {
                    t: modelData.id
                    d: modelData.url.replace("file://", "").replace(/\/main\.qml$/, "")
                    on: modelData.on
                    onTog: v => { const m = Object.assign({}, S.ui.mods || {}); m[modelData.id] = v; S.setUi("mods", m); S.scanMods() }
                }
            }
            Lbl { visible: S.mods.length === 0; text: "No mods installed"; color: Th.tx3 }
        }
    }

    Component {
        id: keyP
        Column {
            spacing: 0
            Repeater {
                model: [[Th.mod + "K", "Quick switcher"], [Th.mod + ",", "Settings"], [Th.mod + "E", "Edit layout"], ["Alt+Up / Alt+Down", "Previous or next channel"], ["Shift+Esc", "Mark channel as read"], ["Up", "Edit your last message"], ["Esc", "Close popup, cancel reply, edit or layout editing"], ["Shift+click Delete", "Delete a message without confirmation"], ["Middle click", "Open a channel in the second chat block"]]
                Rectangle {
                    width: parent.width
                    height: 44
                    color: "transparent"
                    Rectangle { anchors.bottom: parent.bottom; width: parent.width; height: 1; color: Th.ln }
                    Lbl { anchors.verticalCenter: parent.verticalCenter; text: modelData[1] }
                    Rectangle {
                        anchors.right: parent.right
                        anchors.verticalCenter: parent.verticalCenter
                        width: kl.implicitWidth + 16
                        height: 26
                        radius: 5
                        color: Th.bg3
                        Lbl { id: kl; anchors.centerIn: parent; text: modelData[0]; font.family: Th.mono; font.pixelSize: Th.fsS }
                    }
                }
            }
        }
    }

    Component {
        id: aboutP
        Column {
            spacing: 8
            Lbl { text: "Version 0.1.0"; color: Th.tx2 }
            Lbl { text: "A native client for Discord, Matrix and Stoat. GPL-3.0-or-later."; color: Th.tx2 }
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
