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
        ["feat", "Features", "grid"], ["voice", "Voice and video", "mic"], ["mods", "Mods and themes", "file"],
        ["keys", "Shortcuts", "dots"], ["about", "About", "globe"]
    ]
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
                Lbl { text: "SETTINGS"; color: Th.tx3; font.pixelSize: 11; font.weight: Font.DemiBold; leftPadding: 10; bottomPadding: 6 }
                Repeater {
                    model: r.pages
                    Rectangle {
                        width: parent.width
                        height: 36
                        radius: Th.rs
                        color: S.setPage === modelData[0] ? Th.sel : pm.containsMouse ? Th.hov : "transparent"
                        Row {
                            x: 10
                            spacing: 10
                            anchors.verticalCenter: parent.verticalCenter
                            Ico { n: modelData[2]; sz: 16; col: S.setPage === modelData[0] ? Th.tx : Th.tx2; anchors.verticalCenter: parent.verticalCenter }
                            Lbl { text: modelData[1]; color: S.setPage === modelData[0] ? Th.tx : Th.tx2; anchors.verticalCenter: parent.verticalCenter }
                        }
                        MouseArea { id: pm; anchors.fill: parent; hoverEnabled: true; cursorShape: Qt.PointingHandCursor; onClicked: S.setPage = modelData[0] }
                    }
                }
            }
        }
        ScrollView {
            id: sv
            Layout.fillWidth: true
            Layout.fillHeight: true
            contentWidth: availableWidth
            Loader {
                width: Math.min(sv.availableWidth - 80, 760)
                x: 40
                y: 48
                sourceComponent: ({ accounts: acP, layout: layP, look: lookP, feat: featP, voice: voiceP, mods: modP, keys: keyP, about: aboutP })[S.setPage] || acP
            }
        }
    }
    IBtn { anchors.right: parent.right; anchors.top: parent.top; anchors.margins: 20; n: "x"; sz: 36; tip: "Close  (Esc)"; onClicked: S.showSet = false }

    component H: Lbl { font.pixelSize: Th.fsXL; font.weight: Font.Bold; bottomPadding: 12 }
    component Sub: Lbl { color: Th.tx3; font.pixelSize: 11; font.weight: Font.DemiBold; topPadding: 18; bottomPadding: 6 }
    component Opt: Rectangle {
        id: op
        property string t
        property string d
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
            Lbl { text: op.t; font.weight: Font.Medium }
            Lbl { visible: op.d !== ""; text: op.d; color: Th.tx3; font.pixelSize: Th.fsS; wrapMode: Text.Wrap; width: parent.width }
        }
        Tog { id: tg; anchors.right: parent.right; anchors.verticalCenter: parent.verticalCenter; onToggled: v => op.tog(v) }
    }
    component Seg: Row {
        id: sg
        property var opts: []
        property string cur
        signal pick(string v)
        spacing: 2
        Repeater {
            model: sg.opts
            Rectangle {
                width: sl.implicitWidth + 20
                height: 30
                radius: Th.rs
                color: sg.cur === modelData[0] ? Th.ac : sm.containsMouse ? Th.bg4 : Th.bg3
                Lbl { id: sl; anchors.centerIn: parent; text: modelData[1]; color: sg.cur === modelData[0] ? Th.acTx : Th.tx; font.pixelSize: Th.fsS }
                MouseArea { id: sm; anchors.fill: parent; hoverEnabled: true; cursorShape: Qt.PointingHandCursor; onClicked: sg.pick(modelData[0]) }
            }
        }
    }

    Component {
        id: acP
        Column {
            spacing: 8
            H { text: "Accounts" }
            Repeater {
                model: S.cfgAccs
                Rectangle {
                    width: parent.width
                    height: 64
                    radius: Th.r
                    color: Th.bg2
                    property var a: S.accs[modelData.id]
                    Av { id: aa; x: 14; anchors.verticalCenter: parent.verticalCenter; sz: 38; src: parent.a ? (parent.a.me.av || "") : ""; name: parent.a ? parent.a.me.name : "?"; sv: S.tag(modelData.svc) }
                    Column {
                        anchors.left: aa.right
                        anchors.leftMargin: 12
                        anchors.verticalCenter: parent.verticalCenter
                        Lbl { text: parent.parent.a ? (parent.parent.a.me.disp || parent.parent.a.me.name) : modelData.base === "demo" ? "Demo" : "Not connected"; font.weight: Font.DemiBold }
                        Row {
                            spacing: 6
                            Badge { sv: modelData.svc; txt: true }
                            Lbl { text: S.up.indexOf(modelData.id) >= 0 ? "Online" : "Offline"; color: S.up.indexOf(modelData.id) >= 0 ? Th.ok : Th.tx3; font.pixelSize: 11 }
                        }
                    }
                    Btn { anchors.right: parent.right; anchors.rightMargin: 14; anchors.verticalCenter: parent.verticalCenter; text: "Log out"; dng: true; onClicked: S.c.rm_acc(modelData.id) }
                }
            }
            Sub { text: "ADD ACCOUNT" }
            AccAdd {}
        }
    }

    Component {
        id: layP
        Column {
            spacing: 8
            H { text: "Layout" }
            Lbl { text: "Pick a preset, flip blocks left or right, or drag them around in edit mode."; color: Th.tx2; wrapMode: Text.Wrap; width: parent.width }
            Sub { text: "PRESETS" }
            Flow {
                width: parent.width
                spacing: 10
                Repeater {
                    model: Object.keys(L.presets)
                    Rectangle {
                        width: 176
                        height: 132
                        radius: Th.r
                        color: Th.bg2
                        border.color: pm2.containsMouse ? Th.ac : Th.ln
                        LayPrev { x: 10; y: 10; width: parent.width - 20; height: 88; n: L.presets[modelData] }
                        Lbl { x: 12; y: 106; text: L.presets[modelData].name; font.pixelSize: Th.fsS; font.weight: Font.DemiBold }
                        MouseArea { id: pm2; anchors.fill: parent; hoverEnabled: true; cursorShape: Qt.PointingHandCursor; onClicked: S.preset(modelData) }
                    }
                }
            }
            Sub { text: "BLOCKS" }
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
            Sub { text: "ADVANCED" }
            Row {
                spacing: 8
                Btn { text: "Edit layout"; ico: "layout"; pri: true; onClicked: { S.showSet = false; S.edit = true } }
                Btn { text: "Copy layout"; ico: "copy"; onClicked: S.copy(JSON.stringify(S.lay)) }
                Btn { text: "Paste layout"; onClicked: pst.visible = !pst.visible }
            }
            Row {
                id: pst
                visible: false
                spacing: 8
                Fld { id: pf; width: 480; placeholderText: "Paste a shared layout" }
                Btn { text: "Apply"; pri: true; onClicked: { try { S.setLay(JSON.parse(pf.text)); pst.visible = false } catch (e) { } } }
            }
        }
    }

    Component {
        id: lookP
        Column {
            spacing: 8
            H { text: "Appearance" }
            Sub { text: "THEME" }
            Flow {
                width: parent.width
                spacing: 10
                Repeater {
                    model: S.themes()
                    Rectangle {
                        width: 150
                        height: 64
                        radius: Th.r
                        color: Th.bg2
                        border.color: (S.ui.theme || "graphite") === modelData.id ? Th.ac : tm.containsMouse ? Th.tx3 : Th.ln
                        border.width: (S.ui.theme || "graphite") === modelData.id ? 2 : 1
                        Lbl { anchors.centerIn: parent; text: modelData.id.charAt(0).toUpperCase() + modelData.id.slice(1) }
                        MouseArea { id: tm; anchors.fill: parent; hoverEnabled: true; cursorShape: Qt.PointingHandCursor; onClicked: { S.setUi("theme", modelData.id); S.loadTheme() } }
                    }
                }
            }
            Lbl { text: "Drop .json themes into " + S.c.home() + "/themes. Changes load live."; color: Th.tx3; font.pixelSize: Th.fsS; wrapMode: Text.Wrap; width: parent.width }
            Sub { text: "ACCENT" }
            Row {
                spacing: 8
                Repeater {
                    model: ["", "#8fb3ff", "#7dd3c0", "#b69cff", "#ff8fa3", "#e6e6e6", "#6ea8fe"]
                    Rectangle {
                        width: 30
                        height: 30
                        radius: 15
                        color: modelData || Th.bg3
                        border.color: (S.ui.ac || "") === modelData ? Th.tx : Th.ln
                        border.width: 2
                        Lbl { visible: !modelData; anchors.centerIn: parent; text: "–"; color: Th.tx3 }
                        MouseArea { anchors.fill: parent; cursorShape: Qt.PointingHandCursor; onClicked: { S.setUi("ac", modelData); S.loadTheme() } }
                    }
                }
            }
            Sub { text: "TEXT SIZE" }
            Seg { opts: [["12", "12"], ["13", "13"], ["14", "14"], ["15", "15"], ["16", "16"], ["18", "18"]]; cur: String(S.ui.fs || 14); onPick: v => { S.setUi("fs", parseInt(v)); S.loadTheme() } }
            Sub { text: "DENSITY" }
            Seg { opts: [["0.85", "Compact"], ["1", "Default"], ["1.15", "Spacious"]]; cur: String(S.ui.dens || 1); onPick: v => { S.setUi("dens", parseFloat(v)); S.loadTheme() } }
            Sub { text: "NOTIFICATIONS" }
            Opt { t: "Desktop notifications"; d: "Direct messages and mentions while the window is not focused"; on: S.ui.notif !== false; onTog: v => S.setUi("notif", v) }
            Sub { text: "NETWORK BADGES" }
            Opt { t: "Show network badges"; d: "Small D, M and S marks on servers, avatars and DMs"; on: S.ui.badges !== false; onTog: v => S.setUi("badges", v) }
        }
    }

    Component {
        id: featP
        Column {
            spacing: 0
            H { text: "Features" }
            Lbl { text: "Built-in client mods. Each one can be turned off."; color: Th.tx2; bottomPadding: 10 }
            Repeater {
                model: S.featDefs
                Opt { t: modelData.n; d: modelData.d; on: S.f(modelData.k); onTog: v => S.setF(modelData.k, v) }
            }
        }
    }

    Component {
        id: voiceP
        Column {
            spacing: 8
            H { text: "Voice and video" }
            Opt { t: "Noise suppression"; d: "Filter background noise from your microphone"; on: S.ui.ns !== false; onTog: v => { S.setUi("ns", v); S.c.cset("voice.json", JSON.stringify({ ns: v, vad: S.ui.vad || -50, ptt: !!S.ui.ptt })) } }
            Opt { t: "Push to talk"; d: "Only send audio while the key is held (Ctrl+`)"; on: !!S.ui.ptt; onTog: v => { S.setUi("ptt", v); S.c.cset("voice.json", JSON.stringify({ ns: S.ui.ns !== false, vad: S.ui.vad || -50, ptt: v })) } }
            Sub { text: "INPUT SENSITIVITY" }
            Slider {
                width: parent.width
                from: -80
                to: -10
                value: S.ui.vad || -50
                onMoved: { S.setUi("vad", value); S.c.cset("voice.json", JSON.stringify({ ns: S.ui.ns !== false, vad: value, ptt: !!S.ui.ptt })) }
            }
            Sub { text: "SCREEN SHARE" }
            Row {
                spacing: 16
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
            Opt { t: "Share audio"; d: "Send application and system audio with your stream"; on: S.ui.ssnd !== false; onTog: v => S.setUi("ssnd", v) }
            Lbl { visible: !S.f("streamq"); text: "Turn on Stream quality unlock in Features to go above 720p30."; color: Th.warn; font.pixelSize: Th.fsS }
        }
    }

    Component {
        id: modP
        Column {
            spacing: 8
            H { text: "Mods and themes" }
            Lbl {
                width: parent.width
                wrapMode: Text.Wrap
                color: Th.tx2
                text: "A mod is a folder in " + S.c.home() + "/mods with a main.qml. It can import Cord, register blocks with S.regBlk, and hook render, send and ev. A block named like a built-in one in " + S.c.home() + "/blocks replaces it."
            }
            Row {
                spacing: 8
                Btn { text: "Open config folder"; onClicked: Qt.openUrlExternally("file://" + S.c.home()) }
                Btn { text: "Reload"; onClicked: { S.scanMods(); S.loadTheme() } }
            }
            Sub { text: "INSTALLED" }
            Repeater {
                model: S.mods
                Opt {
                    t: modelData.id
                    d: modelData.url.replace("file://", "")
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
            H { text: "Shortcuts" }
            Repeater {
                model: [["Ctrl+K", "Quick switcher"], ["Ctrl+,", "Settings"], ["Ctrl+E", "Edit layout"], ["Alt+Up / Alt+Down", "Previous or next channel"], ["Shift+Esc", "Mark channel as read"], ["Up", "Edit your last message"], ["Esc", "Cancel reply, edit or close"], ["Middle click", "Open a channel in the second chat block"]]
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
            H { text: "About cord" }
            Lbl { text: "Version 0.1.0"; color: Th.tx2 }
            Lbl { text: "A native client for Discord, Matrix and Stoat. GPL-3.0-or-later."; color: Th.tx2 }
            Btn { text: "Source code"; ico: "link"; onClicked: Qt.openUrlExternally("https://github.com/Silvertree2010/cord") }
            Lbl {
                width: parent.width
                wrapMode: Text.Wrap
                topPadding: 12
                color: Th.tx3
                font.pixelSize: Th.fsS
                text: "Using third party clients on Discord is against Discord's Terms of Service. Use at your own risk."
            }
        }
    }
}
