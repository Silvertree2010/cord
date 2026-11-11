import QtQuick
import QtQuick.Controls
import QtQuick.Effects
import Cord
import "../Fmt.js" as F

Item {
    id: r
    property var m: ({})
    property bool grp: false
    property bool day: false
    property bool nw: false
    property var rep: null
    property string cid: ""
    property bool mine: S.isMe(m.auth, S.acc(cid))
    property bool hov: hh.hovered || ma.containsMouse || bar.hov || activeFocus
    activeFocusOnTab: true
    Accessible.role: Accessible.ListItem
    Accessible.name: (r.m.auth ? (r.m.auth.disp || r.m.auth.name) : "") + ": " + (r.m.body || "")
    Keys.onReturnPressed: menu.show(r, 80, 24)
    Keys.onEnterPressed: menu.show(r, 80, 24)
    Keys.onMenuPressed: menu.show(r, 80, 24)
    property bool showH: false
    signal jump(string id)
    width: ListView.view ? ListView.view.width : 400
    height: col.implicitHeight + (grp ? 2 : 10)
    readonly property real gut: 72

    Rectangle {
        anchors.fill: parent
        anchors.leftMargin: 6
        anchors.rightMargin: 6
        anchors.topMargin: r.day ? 28 + 6 : 0
        radius: 6
        color: r.ment ? Th.ment : r.hov ? Th.a(Th.ac, 0.10) : "transparent"
        Behavior on color { ColorAnimation { duration: Th.mF } }
        Rectangle {
            opacity: r.hov && !r.ment && !r.m.del ? 1 : 0
            visible: opacity > 0.01
            width: 4
            height: (parent.height - 8) * (0.4 + 0.6 * opacity)
            anchors.verticalCenter: parent.verticalCenter
            x: 2
            radius: 2
            color: Th.ac
            Behavior on opacity { NumberAnimation { duration: Th.mF; easing.type: Easing.OutCubic } }
        }
        Rectangle { visible: r.ment && !r.m.del; width: 2; height: parent.height - 8; y: 4; x: 0; radius: 1; color: Th.ac }
        Rectangle { visible: !!r.m.del; width: 2; height: parent.height - 8; y: 4; x: 0; radius: 1; color: Th.a(Th.err, 0.6) }
    }
    readonly property var mu: {
        if (r.grp || !r.m.auth || !r.m.auth.id) return null
        const c = S.chans[r.cid]
        if (!c || !c.grid) return null
        void U.memC
        const m = (U.memC[c.grid] || {})[S.ridOf(r.m.auth.id)]
        return m ? m.usr : null
    }
    function nmap() {
        const b = r.m.body || ""
        if (b.indexOf("<@") < 0) return S.names
        const c = S.chans[r.cid]
        if (!c || !c.grid) return S.names
        void U.memC
        const mc = U.memC[c.grid] || {}
        const pre = S.svc(r.cid) + ":" + S.acc(r.cid) + ":"
        let o = null
        b.replace(/<@!?(\d+)>/g, (x, i) => {
            const m = mc[pre + i]
            const n = m ? (m.nick || m.usr.disp || m.usr.name) : ""
            if (n && n !== S.names[i]) { if (!o) o = Object.assign({}, S.names); o[i] = n }
            return x
        })
        return o || S.names
    }
    function usrOf(id) {
        const c = S.chans[r.cid]
        const hit = u => u && u.id && u.id.raw === id
        if (r.m.auth && hit(r.m.auth)) return r.m.auth
        if (c && c.grid) {
            const g = U.memC[c.grid] || {}
            for (const k in g) if (hit(g[k].usr)) return g[k].usr
        }
        for (const u of (c && c.rcp) || []) if (hit(u)) return u
        for (const x of S.rels) if (hit(x.usr)) return x.usr
        const sv = S.svc(r.cid)
        return { id: { svc: sv === "mx" ? "Mx" : sv === "st" ? "St" : "Dc", acc: S.acc(r.cid), raw: id }, name: S.names[id] || id }
    }
    function askDel(skip) {
        const go = () => S.c.del(r.cid, S.ridOf(r.m.id))
        if (skip) { go(); return }
        U.ask({ title: "Delete message", body: "This message will be deleted for everyone in the conversation.", ok: "Delete", danger: true, run: go })
    }
    readonly property bool ment: {
        const me = S.me(S.acc(cid))
        return !!(me && me.id && (m.body || "").indexOf(me.id.raw) >= 0 && !mine)
    }

    HoverHandler { id: hh }
    MouseArea {
        id: ma
        anchors.fill: parent
        hoverEnabled: true
        acceptedButtons: Qt.RightButton
        onClicked: m => menu.show(ma, m.x, m.y, m.modifiers)
    }

    Column {
        id: col
        x: 0
        y: r.grp ? 1 : 8
        width: parent.width
        spacing: 2

        Row {
            visible: r.day
            x: 16
            width: parent.width - 32
            height: 28
            spacing: 4
            Rectangle { width: (parent.width - dl.implicitWidth - 16 - (r.nw ? nwp.width + 8 : 0)) / 2; height: 2; radius: 1; color: r.nw ? Th.err : Th.ln; anchors.verticalCenter: parent.verticalCenter }
            Lbl { id: dl; text: F.day(r.m.ts || 0); color: r.nw ? Th.err : Th.tx2; font.pixelSize: Th.fsXS; font.weight: Font.Medium; anchors.verticalCenter: parent.verticalCenter }
            Rectangle { width: (parent.width - dl.implicitWidth - 16 - (r.nw ? nwp.width + 8 : 0)) / 2; height: 2; radius: 1; color: r.nw ? Th.err : Th.ln; anchors.verticalCenter: parent.verticalCenter }
            Rectangle {
                id: nwp
                visible: r.nw
                width: nwl.implicitWidth + 12
                height: 16
                radius: 8
                color: Th.err
                anchors.verticalCenter: parent.verticalCenter
                Lbl { id: nwl; anchors.centerIn: parent; text: "NEW"; color: Th.wh; font.pixelSize: Th.fsP; font.weight: Font.Bold }
            }
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
                    text: r.rep ? r.rep.body.replace(/\n/g, " ").replace(/<@!?(\d+)>/g, (m, i) => "@" + (S.names[i] || i)).replace(/<#(\d+)>/g, (m, i) => "#" + (S.names[i] || i)).replace(/<a?:(\w+):\d+>/g, ":$1:") : ""
                    color: Th.tx3
                    font.pixelSize: Th.fsS
                    anchors.verticalCenter: parent.verticalCenter
                }
            }
            MouseArea { anchors.fill: parent; cursorShape: Qt.PointingHandCursor; onClicked: if (r.m.reply) r.jump(S.ridOf(r.m.reply)) }
        }

        Item {
            width: parent.width
            height: Math.max(r.grp || r.m.sys ? 0 : 40, body.y + body.height)

            Ico {
                visible: !!r.m.sys
                x: r.gut - 30
                y: 3
                n: r.m.body && /pinned/.test(r.m.body) ? "pin" : /call/.test(r.m.body || "") ? "phone" : /thread/.test(r.m.body || "") ? "thread" : "chevr"
                sz: 16
                col: /boost/.test(r.m.body || "") ? "#ff73fa" : Th.ok
            }
            Av {
                visible: !r.grp && !r.m.sys
                x: 16
                y: 2
                sz: 40
                src: r.m.auth ? (r.m.auth.av || "") : ""
                name: r.m.auth ? r.m.auth.name : ""
                deco: r.m.auth && r.m.auth.deco ? r.m.auth.deco : r.mu && r.mu.deco ? r.mu.deco : ""
                anim: r.hov
                MouseArea { anchors.fill: parent; cursorShape: Qt.PointingHandCursor; onClicked: S.prof(r.m.auth, parent) }
            }
            Lbl {
                visible: r.grp && r.hov && !r.m.sys
                x: 8
                width: r.gut - 16
                y: 3
                horizontalAlignment: Text.AlignRight
                text: new Date(r.m.ts || 0).toLocaleTimeString(Qt.locale(), "HH:mm")
                color: Th.tx3
                font.pixelSize: Th.fsT
            }
            Row {
                id: hd
                visible: !r.grp && !r.m.sys
                x: r.gut
                spacing: 8
                height: visible ? 22 : 0
                Name {
                    usr: r.m.auth && r.m.auth.ncol && r.m.auth.ncol.length ? r.m.auth : (r.mu || r.m.auth || null)
                    text: r.m.auth ? (r.m.auth.disp || r.m.auth.name) : ""
                    col: r.m.auth && r.m.auth.col ? Th.lift("#" + ("000000" + r.m.auth.col.toString(16)).slice(-6), Th.bg2, 4.5) : Th.tx
                    weight: Font.DemiBold
                    anchors.baseline: tm.baseline
                    MouseArea { anchors.fill: parent; cursorShape: Qt.PointingHandCursor; onClicked: S.prof(r.m.auth, parent) }
                }
                RoleIco { role: { void U.memC; const c = S.chans[r.cid]; return c && c.grid && r.m.auth && r.m.auth.id ? S.roleIcon(c.grid, S.ridOf(r.m.auth.id), r.m.aroles && r.m.aroles.length ? r.m.aroles : null) : null }
                    sz: 18; anchors.verticalCenter: parent.verticalCenter }
                TagPill { tag: r.m.auth && r.m.auth.tag ? r.m.auth.tag : r.mu && r.mu.tag ? r.mu.tag : null; fs: 10; anchors.verticalCenter: parent.verticalCenter }
                Rectangle {
                    visible: r.m.auth && r.m.auth.bot
                    width: bt.implicitWidth + 8
                    height: 15
                    radius: Th.rc
                    color: Th.ac
                    anchors.verticalCenter: parent.verticalCenter
                    Lbl { id: bt; anchors.centerIn: parent; text: "BOT"; color: Th.acTx; font.pixelSize: Th.fsP - 1; font.weight: Font.Bold }
                }
                Lbl {
                    id: tm
                    text: F.time(r.m.ts || 0)
                    color: Th.tx3
                    font.pixelSize: Th.fsT
                    y: 4
                }
                Rectangle {
                    visible: !!r.m.del
                    width: dt.implicitWidth + 10
                    height: 15
                    radius: Th.rc
                    color: Th.bg4
                    anchors.verticalCenter: parent.verticalCenter
                    Lbl { id: dt; anchors.centerIn: parent; text: "deleted"; color: Th.tx2; font.pixelSize: Th.fsP }
                }
            }
            Column {
                id: body
                x: r.gut
                y: r.grp || r.m.sys ? 0 : 22
                width: parent.width - r.gut - 24
                spacing: 6

                Row {
                    visible: !!r.m.fwd
                    spacing: 6
                    Ico {
                        n: "reply"
                        sz: 14
                        col: Th.tx3
                        anchors.verticalCenter: parent.verticalCenter
                        transform: Scale { xScale: -1; origin.x: 6.5 }
                    }
                    Lbl { text: "Forwarded"; color: Th.tx3; font.pixelSize: Th.fsS; font.italic: true; anchors.verticalCenter: parent.verticalCenter }
                }
                TextEdit {
                    id: tx
                    width: parent.width
                    readonly property bool linkOnly: {
                        const b = (r.m.body || "").trim()
                        if (!b || /\s/.test(b) || !/^https?:\/\//.test(b)) return false
                        return (r.m.embs || []).some(e => (e.kind === "image" || e.kind === "gifv") && (e.url === b || e.img === b))
                    }
                    visible: (r.m.body || "") !== "" && !linkOnly
                    readOnly: true
                    selectByMouse: true
                    wrapMode: TextEdit.Wrap
                    textFormat: TextEdit.RichText
                    color: r.m.del ? Th.tx3 : r.m.sys ? Th.tx2 : Th.tx
                    font.strikeout: !!r.m.del
                    font.family: Th.font
                    font.pixelSize: F.onlyEmoji(r.m.body) ? Th.fs * 2.4 : Th.fs
                    font.italic: false
                    selectionColor: Th.ac
                    selectedTextColor: Th.acTx
                    property bool spoil: S.f("spoilers")
                    text: {
                        if (r.m.body && r.m.body.indexOf(":R>") >= 0) void S.tick
                        return r.m.sys ? F.esc(r.m.body || "") + ' <span style="color:' + Th.tx3 + ';font-size:11px;font-style:normal">' + F.time(r.m.ts || 0) + '</span>' : F.html(S.run("render", r.m).body || "", { ac: Th.ac, mt: Qt.lighter(Th.ac, 1.12).toString(), mb: Qt.tint(Th.bgc, Th.a(Th.ac, 0.24)).toString(), cb: Th.bg4, qb: Th.tx3, mono: Th.mono, sub: Th.tx3, fs: Th.fs, names: r.nmap(), spoil: spoil, sp: Th.bg4, big: F.onlyEmoji(r.m.body), em: 22, hk: Th.ac, hs: Th.ok, hc: Th.tx3, hn: Th.warn }) + (r.m.ed ? ' <span style="color:' + Th.tx2 + ';font-size:11px">(edited)</span>' : "")
                    }
                    onLinkActivated: l => {
                        if (l === "cord:spoil") { spoil = true; return }
                        if (l.startsWith("cord:chan/")) { const id = l.slice(10); const c = Object.keys(S.chans).find(k => S.raw(k) === id); if (c) { S.sel = S.chans[c].grid || "home"; S.open(c) } return }
                        if (l.startsWith("cord:usr/")) { const u = r.usrOf(l.slice(9)); if (u) S.prof(u, tx); return }
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
                        font.pixelSize: Th.fsT
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
                        sourceComponent: (a.mime || "").startsWith("image/") || /\.(png|jpe?g|gif|webp)$/i.test(a.name) ? imgA : (a.mime || "").startsWith("video/") || /\.(mp4|webm|mov)$/i.test(a.name) ? vidA : (a.mime || "").startsWith("audio/") || /\.(ogg|mp3|m4a|wav|opus|flac)$/i.test(a.name) ? audA : fileA
                    }
                }
                Repeater {
                    model: r.m.stks || []
                    Item {
                        width: 160
                        height: 160
                        readonly property bool lot: (modelData.mime || "") === "application/json"
                        readonly property bool gif: (modelData.mime || "") === "image/gif"
                        AnimatedImage { visible: parent.gif; anchors.fill: parent; source: parent.gif ? modelData.url : ""; fillMode: Image.PreserveAspectFit; asynchronous: true }
                        Image { id: sti; visible: !parent.lot && !parent.gif; anchors.fill: parent; source: !parent.lot && !parent.gif ? modelData.url : ""; fillMode: Image.PreserveAspectFit; asynchronous: true }
                        Rectangle {
                            visible: parent.lot || sti.status === Image.Error
                            anchors.fill: parent
                            radius: Th.r
                            color: Th.bg3
                            border.color: Th.ln
                            Column {
                                anchors.centerIn: parent
                                width: parent.width - 20
                                spacing: 6
                                Ico { anchors.horizontalCenter: parent.horizontalCenter; n: "smile"; sz: 28; col: Th.tx3 }
                                Lbl { width: parent.width; horizontalAlignment: Text.AlignHCenter; text: modelData.name || "Sticker"; color: Th.tx2; font.pixelSize: Th.fsS; wrapMode: Text.Wrap; elide: Text.ElideNone; maximumLineCount: 2 }
                            }
                        }
                    }
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
                            height: 24
                            width: rr.implicitWidth + 16
                            radius: 12
                            color: modelData.me ? Th.a(Th.ac, 0.2) : Th.bg3
                            border.color: modelData.me ? Th.ac : rm.containsMouse ? Th.tx3 : Th.ln
                            border.width: 1
                            Row {
                                id: rr
                                anchors.centerIn: parent
                                spacing: 4
                                Image { visible: !!modelData.emo.url; source: modelData.emo.url || ""; width: 18; height: 18; sourceSize: Qt.size(36, 36); anchors.verticalCenter: parent.verticalCenter }
                                Lbl { visible: !modelData.emo.url; text: modelData.emo.name; font.pixelSize: 15; anchors.verticalCenter: parent.verticalCenter }
                                Lbl { text: modelData.n; color: modelData.me ? Th.ac : Th.tx2; font.pixelSize: Th.fsXS; font.weight: Font.DemiBold; anchors.verticalCenter: parent.verticalCenter }
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
            id: ia
            property real k: a.w && a.h ? Math.min(1, 420 / a.w, 320 / a.h) : 1
            property bool sp: /^SPOILER_/.test(a.name || "") && !S.f("spoilers")
            width: a.w ? a.w * k : 320
            height: a.h ? a.h * k : 200
            source: /\.gif($|\?)/i.test(a.url) || (a.mime || "") === "image/gif" ? S.u(a.url) : S.thumb(a.url, width * 2, height * 2, false)
            fillMode: Image.PreserveAspectFit
            asynchronous: true
            sourceSize: Qt.size(width * 2, height * 2)
            layer.enabled: sp
            layer.effect: MultiEffect { blurEnabled: true; blur: 1.0; blurMax: 64; brightness: -0.25 }
            Rectangle { anchors.fill: parent; color: Th.bg3; radius: Th.rs; visible: parent.status !== Image.Ready; z: -1 }
            Rectangle {
                visible: ia.sp
                anchors.centerIn: parent
                width: spl.implicitWidth + 20
                height: 26
                radius: 13
                color: Th.shade
                z: 2
                Lbl { id: spl; anchors.centerIn: parent; text: "SPOILER"; color: Th.wh; font.pixelSize: Th.fsXS; font.weight: Font.Bold }
            }
            MouseArea { anchors.fill: parent; z: 3; cursorShape: Qt.PointingHandCursor; onClicked: ia.sp ? ia.sp = false : S.view(a.url) }
        }
    }
    Component {
        id: vidA
        Item {
            id: va
            readonly property string st: S.thumb(a.url, 0, 0, true)
            property real k: a.w && a.h ? Math.min(1, 420 / a.w, 320 / a.h) : 1
            width: st ? (a.w ? a.w * k : 400) : 360
            height: st ? (a.h ? a.h * k : 225) : 56
            Rectangle { anchors.fill: parent; radius: Th.rs; color: Th.bg3 }
            Image {
                visible: !!va.st
                anchors.fill: parent
                source: va.st ? va.st + "&width=" + Math.round(va.width * 2) + "&height=" + Math.round(va.height * 2) : ""
                fillMode: Image.PreserveAspectFit
                asynchronous: true
            }
            Rectangle {
                visible: !!va.st
                anchors.centerIn: parent
                width: 52
                height: 52
                radius: 26
                color: Th.shade
                Ico { anchors.centerIn: parent; n: "play"; sz: 24; col: Th.wh }
            }
            Row {
                visible: !va.st
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
        id: audA
        Loader {
            source: "Aud.qml"
            onLoaded: { item.src = S.u(a.url); item.name = a.name }
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
                    Lbl { text: F.size(a.size || 0); color: Th.tx3; font.pixelSize: Th.fsT }
                }
            }
            MouseArea { anchors.fill: parent; cursorShape: Qt.PointingHandCursor; onClicked: Qt.openUrlExternally(a.url) }
        }
    }

    Rectangle {
        id: bar
        property bool hov: bm.containsMouse
        visible: r.hov && !r.m.del && !(U.pop && U.pop.objectName === "prof")
        anchors.right: parent.right
        anchors.rightMargin: 18
        y: r.ListView.view && r.y - r.ListView.view.contentY < 18 ? 2 : -14
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
            Repeater {
                model: (S.ui.emor || []).slice(0, 3)
                Rectangle {
                    width: 28
                    height: 28
                    radius: Th.rs
                    color: qm.containsMouse ? Th.hov : "transparent"
                    scale: qm.pressed ? 0.88 : 1
                    Behavior on scale { NumberAnimation { duration: Th.mF } }
                    Image { visible: !!modelData.url; anchors.centerIn: parent; width: 18; height: 18; source: modelData.url || ""; sourceSize: Qt.size(36, 36); asynchronous: true }
                    Text { visible: !modelData.url; anchors.centerIn: parent; text: modelData.url ? "" : modelData.name; font.pixelSize: 16 }
                    MouseArea { id: qm; anchors.fill: parent; hoverEnabled: true; cursorShape: Qt.PointingHandCursor; onClicked: S.c.react(r.cid, S.ridOf(r.m.id), JSON.stringify(modelData), true) }
                }
            }
            IBtn { n: "smile"; sz: 28; isz: 16; tip: "Add reaction"; onClicked: S.emoji(bar, e => S.c.react(r.cid, S.ridOf(r.m.id), JSON.stringify(e), true)) }
            IBtn { n: "reply"; sz: 28; isz: 16; tip: "Reply"; onClicked: S.reply = S.ridOf(r.m.id) }
            IBtn { visible: r.mine; n: "edit"; sz: 28; isz: 16; tip: "Edit"; onClicked: S.editing = S.ridOf(r.m.id) }
            IBtn { visible: r.mine; n: "trash"; sz: 28; isz: 16; tip: "Delete  (Shift+click skips confirm)"; onClicked: r.askDel(!!(mods & Qt.ShiftModifier)) }
            IBtn { id: dotB; n: "dots"; sz: 28; isz: 16; tip: "More"; onClicked: menu.show(dotB, 0, dotB.height + 4) }
        }
    }

    Rectangle {
        anchors.fill: parent
        anchors.leftMargin: 8
        anchors.rightMargin: 8
        radius: Th.rs
        color: "transparent"
        border.color: Th.ac
        border.width: 2
        visible: r.activeFocus
    }

    Ctx {
        id: menu
        model: [
            { text: "Add Reaction", when: !r.m.del, run: () => S.emoji(r, e => S.c.react(r.cid, S.ridOf(r.m.id), JSON.stringify(e), true)) },
            { text: "Reply", when: !r.m.del, run: () => S.reply = S.ridOf(r.m.id) },
            { text: "Mention", when: !!(r.m.auth && r.m.auth.id) && !r.mine, run: () => S.insReq = "<@" + r.m.auth.id.raw + "> " },
            { text: "Copy Text", when: (r.m.body || "") !== "", run: () => S.copy(tx.getText(0, tx.length)) },
            { text: "Edit", when: r.mine && !r.m.del, run: () => S.editing = S.ridOf(r.m.id) },
            { text: "Copy", items: [
                { text: "Text", run: () => S.copy(tx.getText(0, tx.length)) },
                { text: "Raw text", when: S.f("copyraw"), run: () => S.copy(r.m.body) },
                { text: "Message ID", run: () => S.copy(r.m.id.raw) },
                { text: "Link", when: !!r.m.permalink, run: () => S.copy(r.m.permalink) }
            ] },
            { text: "Translate", when: S.f("translate"), run: () => Qt.openUrlExternally("https://translate.google.com/?sl=auto&tl=" + Qt.locale().name.split("_")[0] + "&text=" + encodeURIComponent(r.m.body)) },
            { text: r.m.pin ? "Unpin Message" : "Pin Message", when: S.svc(r.cid) === "dc" && !r.m.del, run: () => S.api(S.acc(r.cid), r.m.pin ? "unpin" : "pin", { chan: S.raw(r.cid), msg: r.m.id.raw }, (ok, v, e) => { if (!ok) S.pushErr("pin", e, S.acc(r.cid)) }) },
            { text: "Mark Unread", when: S.svc(r.cid) === "dc", run: () => S.api(S.acc(r.cid), "unread", { chan: S.raw(r.cid), msg: S.decId(r.m.id.raw) }) },
            { text: "View Profile", when: !!r.m.auth && !r.m.sys, run: () => S.prof(r.m.auth, r) },
            { sep: true, when: r.mine && !r.m.del },
            { text: "Delete", danger: true, when: r.mine && !r.m.del, run: () => r.askDel(!!(menu.mods & Qt.ShiftModifier)) }
        ]
    }
}
