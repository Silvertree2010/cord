import QtQuick
import QtQuick.Controls
import Cord
import "../Fmt.js" as F

Item {
    id: r
    property var m: ({})
    property bool grp: false
    property bool day: false
    property var rep: null
    property string cid: ""
    property bool mine: S.isMe(m.auth, S.acc(cid))
    property bool hov: ma.containsMouse || bar.hov
    property bool showH: false
    signal jump(string id)
    width: ListView.view ? ListView.view.width : 400
    height: col.implicitHeight + (grp ? 2 : 10)
    readonly property real gut: 72

    Rectangle {
        anchors.fill: parent
        color: r.m.del ? Th.err + "14" : r.ment ? Th.ment : r.hov ? Th.hov : "transparent"
        Rectangle { visible: r.ment && !r.m.del; width: 2; height: parent.height; color: Th.ac }
    }
    readonly property bool ment: {
        const me = S.me(S.acc(cid))
        return !!(me && me.id && (m.body || "").indexOf(me.id.raw) >= 0 && !mine)
    }

    MouseArea {
        id: ma
        anchors.fill: parent
        hoverEnabled: true
        acceptedButtons: Qt.RightButton
        onClicked: menu.popup()
    }

    Column {
        id: col
        x: 0
        y: r.grp ? 1 : 8
        width: parent.width
        spacing: 2

        Row {
            visible: r.day
            width: parent.width
            height: 28
            spacing: 8
            Rectangle { width: (parent.width - dl.implicitWidth - 16) / 2; height: 1; color: Th.ln; anchors.verticalCenter: parent.verticalCenter }
            Lbl { id: dl; text: F.day(r.m.ts || 0); color: Th.tx3; font.pixelSize: 11; font.weight: Font.DemiBold; anchors.verticalCenter: parent.verticalCenter }
            Rectangle { width: (parent.width - dl.implicitWidth - 16) / 2; height: 1; color: Th.ln; anchors.verticalCenter: parent.verticalCenter }
        }

        Item {
            visible: !!r.m.reply
            width: parent.width
            height: 20
            Row {
                x: r.gut
                spacing: 6
                width: parent.width - r.gut - 16
                Ico { n: "reply"; sz: 14; col: Th.tx3; anchors.verticalCenter: parent.verticalCenter }
                Av { sz: 16; visible: !!r.rep; src: r.rep ? (r.rep.auth.av || "") : ""; name: r.rep ? r.rep.auth.name : ""; anchors.verticalCenter: parent.verticalCenter }
                Lbl {
                    text: r.rep ? (r.rep.auth.disp || r.rep.auth.name) : "Original message"
                    color: Th.tx2
                    font.pixelSize: Th.fsS
                    font.weight: Font.DemiBold
                    anchors.verticalCenter: parent.verticalCenter
                }
                Lbl {
                    width: parent.width - 200
                    text: r.rep ? r.rep.body.replace(/\n/g, " ") : ""
                    color: Th.tx3
                    font.pixelSize: Th.fsS
                    anchors.verticalCenter: parent.verticalCenter
                }
            }
            MouseArea { anchors.fill: parent; cursorShape: Qt.PointingHandCursor; onClicked: if (r.m.reply) r.jump(S.ridOf(r.m.reply)) }
        }

        Item {
            width: parent.width
            height: Math.max(r.grp ? 0 : 40, body.y + body.height)

            Av {
                visible: !r.grp
                x: 16
                y: 2
                sz: 40
                src: r.m.auth ? (r.m.auth.av || "") : ""
                name: r.m.auth ? r.m.auth.name : ""
                MouseArea { anchors.fill: parent; cursorShape: Qt.PointingHandCursor; onClicked: S.prof(r.m.auth, parent) }
            }
            Lbl {
                visible: r.grp && r.hov
                x: 8
                width: r.gut - 16
                y: 3
                horizontalAlignment: Text.AlignRight
                text: new Date(r.m.ts || 0).toLocaleTimeString(Qt.locale(), "HH:mm")
                color: Th.tx3
                font.pixelSize: 10
            }
            Row {
                id: hd
                visible: !r.grp
                x: r.gut
                spacing: 8
                height: visible ? 22 : 0
                Lbl {
                    text: r.m.auth ? (r.m.auth.disp || r.m.auth.name) : ""
                    color: r.m.auth && r.m.auth.col ? "#" + ("000000" + r.m.auth.col.toString(16)).slice(-6) : Th.tx
                    font.weight: Font.DemiBold
                    anchors.baseline: tm.baseline
                    MouseArea { anchors.fill: parent; cursorShape: Qt.PointingHandCursor; onClicked: S.prof(r.m.auth, parent) }
                }
                Rectangle {
                    visible: r.m.auth && r.m.auth.bot
                    width: bt.implicitWidth + 8
                    height: 15
                    radius: 3
                    color: Th.ac
                    anchors.verticalCenter: parent.verticalCenter
                    Lbl { id: bt; anchors.centerIn: parent; text: "BOT"; color: Th.acTx; font.pixelSize: 9; font.weight: Font.Bold }
                }
                Lbl {
                    id: tm
                    text: F.time(r.m.ts || 0)
                    color: Th.tx3
                    font.pixelSize: 11
                    y: 4
                }
                Lbl {
                    visible: !!r.m.del
                    text: "deleted"
                    color: Th.err
                    font.pixelSize: 11
                    y: 4
                }
            }
            Column {
                id: body
                x: r.gut
                y: r.grp ? 0 : 22
                width: parent.width - r.gut - 24
                spacing: 6

                TextEdit {
                    id: tx
                    width: parent.width
                    visible: (r.m.body || "") !== ""
                    readOnly: true
                    selectByMouse: true
                    wrapMode: TextEdit.Wrap
                    textFormat: TextEdit.RichText
                    color: r.m.del ? Th.err : r.m.sys ? Th.tx3 : Th.tx
                    font.family: Th.font
                    font.pixelSize: F.onlyEmoji(r.m.body) ? Th.fs * 2.4 : Th.fs
                    font.italic: !!r.m.sys
                    selectionColor: Th.ac
                    selectedTextColor: Th.acTx
                    property bool spoil: S.f("spoilers")
                    text: F.html(S.run("render", r.m).body || "", { ac: Th.ac, cb: Th.bg4, qb: Th.tx3, mono: Th.mono, sub: Th.tx3, fs: Th.fs, names: S.names, spoil: spoil, sp: Th.bg4, big: F.onlyEmoji(r.m.body), em: 22 }) + (r.m.ed ? ' <span style="color:' + Th.tx3 + ';font-size:10px">(edited)</span>' : "")
                    onLinkActivated: l => {
                        if (l === "cord:spoil") { spoil = true; return }
                        if (l.startsWith("cord:chan/")) { const id = l.slice(10); const c = Object.keys(S.chans).find(k => S.raw(k) === id); if (c) { S.sel = S.chans[c].grid || "home"; S.open(c) } return }
                        if (l.startsWith("cord:usr/")) return
                        Qt.openUrlExternally(l)
                    }
                    HoverHandler { cursorShape: parent.hoveredLink ? Qt.PointingHandCursor : Qt.IBeamCursor }
                }

                Column {
                    visible: (r.m.hist || []).length > 0
                    spacing: 2
                    Lbl {
                        text: r.showH ? "Hide edit history" : "Edit history (" + (r.m.hist || []).length + ")"
                        color: Th.tx3
                        font.pixelSize: 11
                        MouseArea { anchors.fill: parent; cursorShape: Qt.PointingHandCursor; onClicked: r.showH = !r.showH }
                    }
                    Repeater {
                        model: r.showH ? r.m.hist : []
                        Lbl { text: modelData; color: Th.tx3; font.pixelSize: Th.fsS; font.strikeout: true; width: body.width; wrapMode: Text.Wrap }
                    }
                }

                Repeater {
                    model: r.m.atts || []
                    Loader {
                        property var a: modelData
                        sourceComponent: (a.mime || "").startsWith("image/") || /\.(png|jpe?g|gif|webp)$/i.test(a.name) ? imgA : (a.mime || "").startsWith("video/") || /\.(mp4|webm|mov)$/i.test(a.name) ? vidA : fileA
                    }
                }
                Repeater {
                    model: r.m.stks || []
                    Image { source: modelData.url; width: 160; height: 160; fillMode: Image.PreserveAspectFit; asynchronous: true }
                }
                Repeater {
                    model: r.m.embs || []
                    Emb { e: modelData; maxW: Math.min(body.width, 520) }
                }

                Flow {
                    visible: (r.m.reacts || []).length > 0
                    width: parent.width
                    spacing: 4
                    Repeater {
                        model: r.m.reacts || []
                        Rectangle {
                            height: 26
                            width: rr.implicitWidth + 14
                            radius: 8
                            color: modelData.me ? Th.ac + "33" : Th.bg3
                            border.color: modelData.me ? Th.ac : rm.containsMouse ? Th.tx3 : "transparent"
                            Row {
                                id: rr
                                anchors.centerIn: parent
                                spacing: 4
                                Image { visible: !!modelData.emo.url; source: modelData.emo.url || ""; width: 18; height: 18; sourceSize: Qt.size(36, 36); anchors.verticalCenter: parent.verticalCenter }
                                Lbl { visible: !modelData.emo.url; text: modelData.emo.name; font.pixelSize: 15; anchors.verticalCenter: parent.verticalCenter }
                                Lbl { text: modelData.n; color: modelData.me ? Th.ac : Th.tx2; font.pixelSize: 12; font.weight: Font.DemiBold; anchors.verticalCenter: parent.verticalCenter }
                            }
                            MouseArea {
                                id: rm
                                anchors.fill: parent
                                hoverEnabled: true
                                cursorShape: Qt.PointingHandCursor
                                onClicked: S.c.react(r.cid, S.ridOf(r.m.id), JSON.stringify(modelData.emo), !modelData.me)
                            }
                        }
                    }
                }
            }
        }
    }

    Component {
        id: imgA
        Image {
            property real k: a.w && a.h ? Math.min(1, 420 / a.w, 320 / a.h) : 1
            width: a.w ? a.w * k : 320
            height: a.h ? a.h * k : 200
            source: S.u(a.url)
            fillMode: Image.PreserveAspectFit
            asynchronous: true
            sourceSize: Qt.size(width * 2, height * 2)
            Rectangle { anchors.fill: parent; color: Th.bg3; radius: Th.rs; visible: parent.status !== Image.Ready; z: -1 }
            MouseArea { anchors.fill: parent; cursorShape: Qt.PointingHandCursor; onClicked: S.view(a.url) }
        }
    }
    Component {
        id: vidA
        Rectangle {
            width: 360
            height: 56
            radius: Th.rs
            color: Th.bg3
            Row {
                anchors.verticalCenter: parent.verticalCenter
                x: 12
                spacing: 10
                Ico { n: "play"; sz: 22; col: Th.tx; anchors.verticalCenter: parent.verticalCenter }
                Lbl { text: a.name; width: 280; anchors.verticalCenter: parent.verticalCenter }
            }
            MouseArea { anchors.fill: parent; cursorShape: Qt.PointingHandCursor; onClicked: S.view(a.url, true) }
        }
    }
    Component {
        id: fileA
        Rectangle {
            width: Math.min(body.width, 420)
            height: 56
            radius: Th.rs
            color: Th.bg3
            border.color: Th.ln
            Row {
                anchors.verticalCenter: parent.verticalCenter
                x: 12
                spacing: 10
                Ico { n: "file"; sz: 26; col: Th.tx2; anchors.verticalCenter: parent.verticalCenter }
                Column {
                    anchors.verticalCenter: parent.verticalCenter
                    Lbl { text: a.name; color: Th.ac; width: 300 }
                    Lbl { text: F.size(a.size || 0); color: Th.tx3; font.pixelSize: 11 }
                }
            }
            MouseArea { anchors.fill: parent; cursorShape: Qt.PointingHandCursor; onClicked: Qt.openUrlExternally(a.url) }
        }
    }

    Rectangle {
        id: bar
        property bool hov: bm.containsMouse
        visible: r.hov && !r.m.del
        anchors.right: parent.right
        anchors.rightMargin: 16
        y: -12
        z: 10
        height: 32
        width: br.implicitWidth + 4
        radius: Th.rs
        color: Th.bg2
        border.color: Th.ln
        MouseArea { id: bm; anchors.fill: parent; hoverEnabled: true }
        Row {
            id: br
            anchors.centerIn: parent
            IBtn { n: "smile"; sz: 28; isz: 16; tip: "Add reaction"; onClicked: S.emoji(bar, e => S.c.react(r.cid, S.ridOf(r.m.id), JSON.stringify(e), true)) }
            IBtn { n: "reply"; sz: 28; isz: 16; tip: "Reply"; onClicked: S.reply = S.ridOf(r.m.id) }
            IBtn { visible: r.mine; n: "edit"; sz: 28; isz: 16; tip: "Edit"; onClicked: S.editing = S.ridOf(r.m.id) }
            IBtn { n: "dots"; sz: 28; isz: 16; tip: "More"; onClicked: menu.popup() }
        }
    }

    Menu {
        id: menu
        MenuItem { text: "Reply"; onTriggered: S.reply = S.ridOf(r.m.id) }
        MenuItem { text: "Edit"; visible: r.mine; height: visible ? implicitHeight : 0; onTriggered: S.editing = S.ridOf(r.m.id) }
        MenuItem { text: "Copy text"; onTriggered: S.copy(tx.getText(0, tx.length)) }
        MenuItem { text: "Copy raw"; visible: S.f("copyraw"); height: visible ? implicitHeight : 0; onTriggered: S.copy(r.m.body) }
        MenuItem { text: "Copy message ID"; onTriggered: S.copy(r.m.id.raw) }
        MenuItem { text: "Copy link"; visible: S.svc(r.cid) === "dc"; height: visible ? implicitHeight : 0; onTriggered: { const c = S.chans[r.cid]; S.copy("https://discord.com/channels/" + (c && c.grid ? S.raw(c.grid) : "@me") + "/" + S.raw(r.cid) + "/" + r.m.id.raw) } }
        MenuItem { text: "Translate"; visible: S.f("translate"); height: visible ? implicitHeight : 0; onTriggered: Qt.openUrlExternally("https://translate.google.com/?sl=auto&tl=" + Qt.locale().name.split("_")[0] + "&text=" + encodeURIComponent(r.m.body)) }
        MenuSeparator {}
        MenuItem { text: "Delete"; visible: r.mine; height: visible ? implicitHeight : 0; onTriggered: S.c.del(r.cid, S.ridOf(r.m.id)) }
    }
}
