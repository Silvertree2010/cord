import QtQuick
import QtQuick.Controls
import Cord
import "../Fmt.js" as F

Item {
    id: r
    property var u: null
    property string gld: ""
    property bool popup: false
    property bool playing: true
    readonly property string uid: r.u && r.u.id ? S.ridOf(r.u.id) : ""
    readonly property var p: { void S.profs; return r.uid ? S.profOf(r.uid, r.gld) : null }
    readonly property var pr: !r.uid ? null : S.isMe(r.u, S.acc(r.uid)) ? { st: S.ui.st || "On" } : S.presOf(r.uid)
    readonly property var th: r.p && r.p.theme && r.p.theme.length === 2 ? r.p.theme : null
    readonly property real bh: r.popup ? 106 : 120
    readonly property var tag: (r.p && r.p.tag) || (r.u && r.u.tag) || null
    readonly property var nu: r.p && r.p.ncol && r.p.ncol.length ? { ncol: r.p.ncol } : r.u
    readonly property string deco: (r.p && r.p.deco) || (r.u && r.u.deco) || ""
    implicitHeight: body.y + body.implicitHeight + 18
    readonly property var roles: {
        if (!r.gld || !r.uid) return []
        const m = (U.memC[r.gld] || {})[r.uid]
        const g = S.gmap[r.gld]
        if (!m || !g) return []
        const ids = (m.roles || []).map(x => S.ridOf(x))
        return (g.roles || []).filter(x => ids.indexOf(S.ridOf(x.id)) >= 0 && x.name !== "@everyone").sort((a, b) => b.pos - a.pos)
    }
    onUidChanged: if (uid) S.reqProf(uid, gld)
    Component.onCompleted: if (uid) S.reqProf(uid, gld)

    function hex(n) { return "#" + ("000000" + (n >>> 0).toString(16)).slice(-6) }
    function day(ts) { return ts ? new Date(ts).toLocaleDateString(Qt.locale("en_US"), "MMM d, yyyy") : "" }
    readonly property string created: {
        if (!r.u || !r.u.id || S.svc(r.uid) !== "dc") return ""
        return day(Math.floor(Number(r.u.id.raw) / 4194304) + 1420070400000)
    }
    readonly property color base: r.th ? Qt.tint(r.hex(r.th[0]), Th.a("#000000", 0.62)) : "transparent"

    Rectangle {
        anchors.fill: parent
        radius: Th.r
        visible: !!r.th
        gradient: Gradient {
            GradientStop { position: 0; color: r.th ? Qt.tint(r.hex(r.th[0]), Th.shade) : "transparent" }
            GradientStop { position: 1; color: r.th ? Qt.tint(r.hex(r.th[1]), Th.a("#000000", 0.62)) : "transparent" }
        }
    }

    Item {
        id: ban
        x: 1
        y: 1
        width: parent.width - 2
        height: r.bh
        clip: true
        Rectangle {
            anchors.fill: parent
            anchors.bottomMargin: -Th.r
            radius: Th.r - 1
            color: r.p && r.p.accent !== null && r.p.accent !== undefined ? r.hex(r.p.accent) : r.th ? r.hex(r.th[0]) : r.p ? "#5160e7" : Th.bg4
        }
        AnimatedImage {
            anchors.fill: parent
            visible: !!(r.p && r.p.banner) && status === AnimatedImage.Ready
            source: r.p && r.p.banner ? r.p.banner : ""
            fillMode: Image.PreserveAspectCrop
            asynchronous: true
            playing: r.playing && Th.motion
        }
    }

    Rectangle {
        id: ring
        x: 14
        y: r.bh - 46
        width: 92
        height: 92
        radius: 46
        color: r.th ? r.base : r.popup ? Th.bg3 : Th.bg
        Av {
            anchors.centerIn: parent
            sz: 80
            src: r.u ? (r.u.av || "") : ""
            name: r.u ? r.u.name : ""
            st: r.pr ? r.pr.st : "Off"
            ring: ring.color
            deco: r.deco
            anim: r.playing
        }
    }

    Column {
        id: body
        x: 16
        y: ring.y + ring.height + 10
        width: parent.width - 32
        spacing: 10

        Column {
            width: parent.width
            spacing: 3
            Name {
                width: Math.min(implicitWidth, parent.width)
                usr: r.nu
                text: r.u ? (r.u.disp || r.u.name) : ""
                fs: Th.fsXL - 1
                weight: Font.Bold
            }
            Row {
                spacing: 6
                width: parent.width
                Lbl {
                    width: Math.min(implicitWidth, parent.width - (tp.visible ? tp.width + 6 : 0))
                    text: (r.u ? r.u.name : "") + (r.p && r.p.pronouns ? "  •  " + r.p.pronouns : "")
                    color: Th.tx
                    font.pixelSize: Th.fsS + 1
                    anchors.verticalCenter: parent.verticalCenter
                }
                TagPill { id: tp; tag: r.tag; fs: 12; anchors.verticalCenter: parent.verticalCenter }
            }
        }

        Rectangle {
            readonly property var all: r.p && r.p.badges ? r.p.badges : []
            readonly property int cap: r.popup ? 6 : 99
            visible: all.length > 0
            width: bf.implicitWidth + 8
            height: 34
            radius: 8
            color: Th.a(Th.tx, 0.07)
            Row {
                id: bf
                x: 4
                anchors.verticalCenter: parent.verticalCenter
                spacing: 2
            Repeater {
                model: parent.parent.all.slice(0, parent.parent.cap)
                Rectangle {
                    width: 28
                    height: 28
                    radius: 6
                    color: bm.containsMouse ? Th.a(Th.tx, 0.1) : "transparent"
                    Image {
                        anchors.centerIn: parent
                        width: 22
                        height: 22
                        source: modelData.icon
                        sourceSize: Qt.size(44, 44)
                        asynchronous: true
                        cache: true
                    }
                    MouseArea {
                        id: bm
                        anchors.fill: parent
                        hoverEnabled: true
                        cursorShape: modelData.link ? Qt.PointingHandCursor : Qt.ArrowCursor
                        onClicked: if (modelData.link) Qt.openUrlExternally(modelData.link)
                    }
                    Tip { text: modelData.d; hov: bm.containsMouse; side: "b" }
                }
            }
            Lbl {
                visible: parent.parent.all.length > parent.parent.cap
                text: "+" + (parent.parent.all.length - parent.parent.cap)
                font.pixelSize: Th.fsS + 1
                font.weight: Font.DemiBold
                leftPadding: 4
                rightPadding: 6
                anchors.verticalCenter: parent.verticalCenter
            }
            }
        }

        TextEdit {
            visible: !!(r.p && r.p.bio)
            width: parent.width
            readOnly: true
            selectByMouse: true
            wrapMode: TextEdit.Wrap
            textFormat: TextEdit.RichText
            color: Th.tx
            font.family: Th.font
            font.pixelSize: Th.fsS + 1
            selectionColor: Th.ac
            selectedTextColor: Th.acTx
            text: r.p && r.p.bio ? F.html(r.p.bio, { ac: Th.ac, mt: Qt.lighter(Th.ac, 1.12).toString(), mb: Qt.tint(Th.bgc, Th.a(Th.ac, 0.24)).toString(), cb: Th.bg4, mono: Th.mono, sub: Th.tx2, fs: Th.fsS + 1, names: S.names, em: 18 }) : ""
            onLinkActivated: l => { if (!l.startsWith("cord:")) Qt.openUrlExternally(l) }
            HoverHandler { cursorShape: parent.hoveredLink ? Qt.PointingHandCursor : Qt.IBeamCursor }
        }

        Sec {
            visible: !!(r.pr && r.pr.act)
            t: "Activity"
            Lbl { width: parent.width; text: r.pr && r.pr.act ? r.pr.act : ""; wrapMode: Text.Wrap; elide: Text.ElideNone; font.pixelSize: Th.fsS + 1 }
        }

        Sec {
            visible: !r.popup && (r.created !== "" || !!(r.p && r.p.joined))
            t: "Member Since"
            Row {
                spacing: 10
                Lbl { visible: r.created !== ""; text: r.created; font.pixelSize: Th.fsS + 1 }
                Rectangle { visible: r.created !== "" && !!(r.p && r.p.joined); width: 4; height: 4; radius: 2; color: Th.tx2; anchors.verticalCenter: parent.verticalCenter }
                Lbl { visible: !!(r.p && r.p.joined); text: r.p && r.p.joined ? r.day(Date.parse(r.p.joined)) : ""; font.pixelSize: Th.fsS + 1 }
            }
        }

        Sec {
            visible: r.roles.length > 0
            t: "Roles"
            Flow {
                width: parent.width
                spacing: 4
                Repeater {
                    model: r.roles
                    Rectangle {
                        height: 24
                        width: rl.implicitWidth + 26
                        radius: 6
                        color: Th.a(Th.tx, 0.06)
                        border.color: Th.ln
                        Rectangle {
                            x: 8
                            anchors.verticalCenter: parent.verticalCenter
                            width: 10
                            height: 10
                            radius: 5
                            color: modelData.col ? "#" + ("000000" + modelData.col.toString(16)).slice(-6) : Th.tx2
                        }
                        Lbl { id: rl; x: 22; anchors.verticalCenter: parent.verticalCenter; text: modelData.name; font.pixelSize: Th.fsXS; font.weight: Font.Medium }
                    }
                }
            }
        }

        Sec {
            visible: !r.popup && !!(r.p && r.p.conns && r.p.conns.length)
            t: "Connections"
            Repeater {
                model: r.p && r.p.conns ? r.p.conns : []
                Rectangle {
                    width: parent.width
                    height: 40
                    radius: 8
                    color: cm.containsMouse && modelData.url ? Th.a(Th.tx, 0.08) : Th.a(Th.tx, 0.04)
                    border.color: Th.ln
                    Column {
                        x: 12
                        anchors.verticalCenter: parent.verticalCenter
                        width: parent.width - 48
                        Row {
                            spacing: 6
                            Lbl { text: modelData.name; font.pixelSize: Th.fsS + 1; font.weight: Font.DemiBold; anchors.verticalCenter: parent.verticalCenter }
                            Rectangle {
                                visible: modelData.ok
                                width: 14
                                height: 14
                                radius: 7
                                color: Th.ac
                                anchors.verticalCenter: parent.verticalCenter
                                Ico { anchors.centerIn: parent; n: "check"; sz: 10; col: Th.acTx }
                            }
                        }
                        Lbl { text: modelData.t.charAt(0).toUpperCase() + modelData.t.slice(1); color: Th.tx2; font.pixelSize: Th.fsT }
                    }
                    Ico { visible: !!modelData.url; anchors.right: parent.right; anchors.rightMargin: 12; anchors.verticalCenter: parent.verticalCenter; n: "link"; sz: 16; col: Th.tx2 }
                    MouseArea {
                        id: cm
                        anchors.fill: parent
                        hoverEnabled: true
                        cursorShape: modelData.url ? Qt.PointingHandCursor : Qt.ArrowCursor
                        onClicked: if (modelData.url) Qt.openUrlExternally(modelData.url)
                    }
                }
            }
        }

        Row {
            readonly property int ms: r.p && r.p.mutual ? r.p.mutual.length : 0
            readonly property int mf: r.p && r.p.mfriends ? r.p.mfriends : 0
            visible: ms + mf > 0
            spacing: 8
            Row {
                visible: !!(r.p && r.p.mfu && r.p.mfu.length)
                spacing: -6
                anchors.verticalCenter: parent.verticalCenter
                Repeater {
                    model: r.p && r.p.mfu ? r.p.mfu : []
                    Rectangle {
                        width: 22
                        height: 22
                        radius: 11
                        color: r.th ? r.base : r.popup ? Th.bg3 : Th.bg
                        Av { anchors.centerIn: parent; sz: 18; src: modelData.av || ""; name: modelData.name || "" }
                    }
                }
            }
            Lbl {
                text: [parent.mf ? parent.mf + (parent.mf === 1 ? " Mutual Friend" : " Mutual Friends") : "", parent.ms ? parent.ms + (parent.ms === 1 ? " Mutual Server" : " Mutual Servers") : ""].filter(x => x).join("  •  ")
                color: Th.tx2
                font.pixelSize: Th.fsS + 1
                anchors.verticalCenter: parent.verticalCenter
            }
        }

        Rectangle {
            visible: !!r.popup && !(r.u && S.isMe(r.u, S.acc(r.uid)))
            width: parent.width
            height: 42
            radius: 8
            color: Th.a(Th.tx, 0.07)
            border.color: qf.activeFocus ? Th.a(Th.ac, 0.6) : "transparent"
            TextInput {
                id: qf
                x: 14
                width: parent.width - 48
                anchors.verticalCenter: parent.verticalCenter
                color: Th.tx
                font.family: Th.font
                font.pixelSize: Th.fs
                clip: true
                selectByMouse: true
                onAccepted: { if (r.u && text.trim()) { S.dmSend(r.u, text); text = ""; r.closed() } }
                Lbl { visible: !qf.text; text: "Message @" + (r.u ? (r.u.disp || r.u.name) : ""); color: Th.tx2; anchors.verticalCenter: parent.verticalCenter }
            }
            Ico { anchors.right: parent.right; anchors.rightMargin: 12; anchors.verticalCenter: parent.verticalCenter; n: "smile"; sz: 18; col: Th.tx2 }
        }
    }

    signal closed

    Row {
        visible: r.popup
        z: 60
        anchors.right: parent.right
        anchors.top: parent.top
        anchors.margins: 10
        spacing: 6
        Rectangle {
            width: 32
            height: 32
            radius: 16
            color: Th.a("#000000", pdm.containsMouse ? 0.55 : 0.4)
            Ico { anchors.centerIn: parent; n: "dots"; sz: 18; col: Th.wh }
            MouseArea { id: pdm; anchors.fill: parent; hoverEnabled: true; cursorShape: Qt.PointingHandCursor; onClicked: pmenu.show(parent, 0, parent.height + 4) }
            Ctx {
                id: pmenu
                model: [
                    { text: "Send Message", run: () => { if (r.u) S.openDm(r.u); r.closed() } },
                    { text: "Copy Username", run: () => S.copy(r.u ? r.u.name : "") },
                    { text: "Copy User ID", run: () => S.copy(r.u && r.u.id ? r.u.id.raw : "") }
                ]
            }
        }
    }

    Effect {
        anchors.fill: parent
        z: 50
        cfg: r.p && r.p.effect ? r.p.effect : null
        playing: r.playing && r.visible && Th.motion && S.ui.peff !== false
        visible: !!cfg && S.ui.peff !== false
    }

    component Sec: Column {
        property string t
        width: body.width
        spacing: 6
        Lbl { text: parent.t.toUpperCase(); color: Th.tx2; font.pixelSize: Th.fsS - 1; font.weight: Font.Bold; font.letterSpacing: 0.6 }
    }
}
