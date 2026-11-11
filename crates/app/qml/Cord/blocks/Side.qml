import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import Cord

Rectangle {
    id: r
    property var o: ({})
    implicitWidth: 248
    implicitHeight: 400
    color: Th.bg2
    property bool home: S.sel === "home"
    property var g: S.gmap[S.sel] || null
    property var col: S.ui.col || ({})

    ColumnLayout {
        anchors.fill: parent
        spacing: 0
        Rectangle {
            Layout.fillWidth: true
            Layout.preferredHeight: 50
            color: "transparent"
            Rectangle { anchors.bottom: parent.bottom; x: 8; width: parent.width - 16; height: 1; color: Th.ln }
            Loader {
                id: hdL
                anchors.fill: parent
                sourceComponent: r.home ? homeHd : gHd
                onLoaded: hdA.restart()
                NumberAnimation { id: hdA; target: hdL; property: "opacity"; from: 0; to: 1; duration: Th.mM; easing.type: Easing.OutCubic }
            }
        }
        ListView {
            id: lv
            Layout.fillWidth: true
            Layout.fillHeight: true
            clip: true
            topMargin: 8
            bottomMargin: 8
            model: r.home ? S.dms() : S.chList(S.sel).filter(c => c.k === "Cat" || !r.col[c.prid] || c.rid === S.selC || (!c.mute && S.unr[c.rid] && (S.unr[c.rid].n || S.unr[c.rid].ment)))
            ScrollBar.vertical: Scr {}
            header: r.home ? frHd : null
            Smooth {}
            transform: Translate { id: lvT }
            Connections {
                target: S
                function onSelChanged() { lvA.restart() }
            }
            ParallelAnimation {
                id: lvA
                NumberAnimation { target: lv; property: "opacity"; from: 0; to: 1; duration: Th.mM; easing.type: Easing.OutCubic }
                NumberAnimation { target: lvT; property: "x"; from: -12; to: 0; duration: Th.mS; easing.type: Easing.OutCubic }
            }
            delegate: Item {
                width: lv.width
                height: ld.height
                Loader {
                    id: ld
                    property var ch: modelData
                    sourceComponent: modelData.k === "Cat" ? catD : r.home ? dmD : chD
                }
            }
        }
    }

    Component {
        id: homeHd
        Item {
            Rectangle {
                anchors.fill: parent
                anchors.margins: 8
                anchors.leftMargin: 10
                anchors.rightMargin: 10
                radius: 6
                color: hm.containsMouse ? Qt.lighter(Th.bg3, 1.12) : Th.bg3
                Lbl {
                    anchors.centerIn: parent
                    width: Math.min(implicitWidth, parent.width - 16)
                    text: S.t("Find or start a conversation")
                    color: Th.tx
                    font.pixelSize: Th.fsS + 1
                    font.weight: Font.Medium
                }
                MouseArea { id: hm; anchors.fill: parent; hoverEnabled: true; cursorShape: Qt.PointingHandCursor; onClicked: S.quick() }
            }
        }
    }
    Component {
        id: gHd
        Item {
            Row {
                anchors.verticalCenter: parent.verticalCenter
                x: 14
                spacing: 8
                width: parent.width - 28
                Badge {
                    id: hb
                    visible: S.multi
                    sv: S.svc(S.sel)
                    txt: true
                    anchors.verticalCenter: parent.verticalCenter
                }
                Lbl {
                    width: Math.min(implicitWidth, parent.width - 24 - (hb.visible ? hb.width + 8 : 0))
                    text: r.g ? r.g.name : ""
                    font.weight: Font.DemiBold
                    font.pixelSize: Th.fs + 1
                    anchors.verticalCenter: parent.verticalCenter
                }
                Ico { n: "chev"; sz: 14; col: Th.tx3; anchors.verticalCenter: parent.verticalCenter }
            }
            Rectangle {
                visible: false
                anchors.right: parent.right
                anchors.rightMargin: 14
                anchors.verticalCenter: parent.verticalCenter
                width: dc.implicitWidth + 12
                height: 16
                radius: Th.rc
                color: Th.bg4
                Lbl { id: dc; anchors.centerIn: parent; text: S.t("Demo"); color: Th.tx2; font.pixelSize: Th.fsP; font.weight: Font.DemiBold }
            }
            MouseArea {
                anchors.fill: parent
                acceptedButtons: Qt.RightButton
                onClicked: m => gm.show(parent, m.x, m.y)
            }
            Ctx {
                id: gm
                model: [
                    { text: S.t("Mark server as read"), run: () => { for (const c of S.chList(S.sel)) if (S.unr[c.rid]) S.c.ack(c.rid) } },
                    { text: S.t("Copy server ID"), run: () => S.copy(S.raw(S.sel)) },
                    { text: S.t("Leave server"), danger: true, when: typeof S.c.leave_gld === "function", run: () => { const g = S.sel; U.ask({ title: S.t("Leave server"), body: S.t("Leave ") + (r.g ? r.g.name : "this server") + "? You need a new invite to join again.", ok: S.t("Leave"), danger: true, run: () => S.c.leave_gld(g) }) } }
                ]
            }
        }
    }
    Component {
        id: frHd
        Column {
            width: lv.width
            spacing: 0
            Row_ { ico: "users"; text: S.t("Friends"); cur: S.selC === ""; onPick: S.open("") }
            Row_ { ico: "inbox"; text: S.t("Inbox"); cur: S.selC === "inbox"; onPick: S.open("inbox"); visible: !S.hasBlk("inbox") }
            Item { width: 1; height: 10 }
            Rectangle { x: 8; width: lv.width - 16; height: 1; color: Th.ln }
            Item {
                width: lv.width
                height: 40
                Lbl {
                    x: 16
                    anchors.verticalCenter: parent.verticalCenter
                    text: S.t("DIRECT MESSAGES")
                    color: dh.hovered ? Th.tx : Th.tx2
                    font.pixelSize: Th.fsS - 1
                    font.weight: Font.Bold
                    font.letterSpacing: 0.6
                }
                Ico {
                    anchors.right: parent.right
                    anchors.rightMargin: 16
                    anchors.verticalCenter: parent.verticalCenter
                    n: "plus"
                    sz: 16
                    col: pm.containsMouse ? Th.tx : Th.tx2
                    MouseArea { id: pm; anchors.fill: parent; anchors.margins: -6; hoverEnabled: true; cursorShape: Qt.PointingHandCursor; onClicked: S.quick() }
                }
                HoverHandler { id: dh }
            }
        }
    }
    component Row_: Rectangle {
        id: rw
        property string ico
        property string text
        property bool cur
        signal pick
        x: 8
        width: lv.width - 16
        height: 40 * Th.dens
        radius: 6
        color: cur ? Th.sel : ma.containsMouse ? Th.hov : "transparent"
        Behavior on color { ColorAnimation { duration: Th.mF } }
        Row {
            x: 9
            spacing: 12
            anchors.verticalCenter: parent.verticalCenter
            Ico { n: rw.ico; sz: 20; col: rw.cur || ma.containsMouse ? Th.tx : Th.tx2; anchors.verticalCenter: parent.verticalCenter }
            Lbl { text: rw.text; color: rw.cur || ma.containsMouse ? Th.tx : Th.tx2; font.pixelSize: Th.fs + 1; font.weight: Font.Medium; anchors.verticalCenter: parent.verticalCenter }
        }
        MouseArea { id: ma; anchors.fill: parent; hoverEnabled: true; cursorShape: Qt.PointingHandCursor; onClicked: rw.pick() }
    }

    Component {
        id: catD
        Item {
            height: 30
            Row {
                x: 8
                y: 12
                spacing: 2
                Ico {
                    n: r.col[ch.rid] ? "chevr" : "chev"
                    sz: 12
                    col: Th.tx3
                    anchors.verticalCenter: parent.verticalCenter
                }
                Lbl {
                    text: ch.name.toUpperCase()
                    color: cm.containsMouse ? Th.tx : Th.tx2
                    font.pixelSize: Th.fsS - 1
                    font.weight: Font.Bold
                    font.letterSpacing: 0.6
                    Behavior on color { ColorAnimation { duration: Th.mF } }
                    anchors.verticalCenter: parent.verticalCenter
                }
            }
            MouseArea {
                id: cm
                anchors.fill: parent
                hoverEnabled: true
                cursorShape: Qt.PointingHandCursor
                onClicked: {
                    const c = Object.assign({}, r.col)
                    if (c[ch.rid]) delete c[ch.rid]; else c[ch.rid] = true
                    S.setUi("col", c)
                }
            }
        }
    }

    Component {
        id: chD
        Column {
            property bool vc: ch.k === "Voice" || ch.k === "Stage"
            property var u: S.unr[ch.rid] || null
            property bool cur: S.selC === ch.rid
            property var inV: vc ? S.vsIn(ch.rid) : []
            property var ty: S.f("typingtweaks") ? Object.values(S.typing[ch.rid] || {}) : []
            Rectangle {
                x: ch.sub ? 30 : 8
                width: lv.width - (ch.sub ? 38 : 16)
                height: (ch.sub ? 30 : 34) * Th.dens
                radius: Th.rs
                color: cur ? Th.sel : ma.containsMouse ? Th.hov : "transparent"
                Rectangle {
                    visible: !!u && !!u.n && !cur && !ch.mute
                    x: -parent.x - 2
                    width: 6
                    height: 8
                    radius: 3
                    color: Th.tx
                    anchors.verticalCenter: parent.verticalCenter
                }
                Rectangle {
                    visible: !!ch.sub
                    x: -12
                    y: -parent.height / 2
                    width: 10
                    height: parent.height
                    color: "transparent"
                    border.color: Th.ln
                    border.width: 0
                    Rectangle { x: 0; width: 1.5; height: parent.height; color: Th.ln }
                    Rectangle { x: 0; y: parent.height - 1.5; width: 10; height: 1.5; color: Th.ln }
                }
                Behavior on color { ColorAnimation { duration: Th.mF } }
                opacity: ch.hidden ? 0.55 : 1
                Row {
                    x: 8
                    spacing: 6
                    anchors.verticalCenter: parent.verticalCenter
                    width: parent.width - 16
                    Ico {
                        visible: !ch.sub
                        n: ch.hidden ? "lock" : ch.k === "Voice" ? "voice" : ch.k === "Stage" ? "stage" : ch.k === "Forum" ? "forum" : ch.k === "Ann" ? "ann" : "hash"
                        col: Th.tx3
                        anchors.verticalCenter: parent.verticalCenter
                    }
                    Lbl {
                        width: parent.width - 60
                        text: ch.name
                        color: cur ? Th.tx : ch.mute ? Th.a(Th.tx3, ma.containsMouse ? 0.8 : 0.5) : (u && u.n) ? Th.tx : ma.containsMouse ? Th.tx2 : Th.tx3
                        font.weight: u && u.n && !ch.mute ? Font.DemiBold : Font.Medium
                        anchors.verticalCenter: parent.verticalCenter
                    }
                }
                Row {
                    anchors.right: parent.right
                    anchors.rightMargin: 8
                    anchors.verticalCenter: parent.verticalCenter
                    spacing: 4
                    Lbl {
                        visible: vc && S.call.chan && S.ridOf(S.call.chan) === ch.rid && !S.call.up
                        text: S.t("Connecting")
                        color: Th.tx2
                        font.pixelSize: Th.fsS
                        anchors.verticalCenter: parent.verticalCenter
                    }
                    Ico {
                        visible: vc && S.call.chan && S.ridOf(S.call.chan) === ch.rid && !!S.call.up
                        n: "voice"
                        sz: 14
                        col: Th.ok
                        anchors.verticalCenter: parent.verticalCenter
                    }
                    Lbl {
                        visible: ty.length > 0
                        text: "···"
                        color: Th.tx2
                        font.weight: Font.Bold
                    }
                    Cnt {
                        visible: !!u && u.ment > 0
                        n: u ? u.ment : 0
                        anchors.verticalCenter: parent.verticalCenter
                    }

                }
                MouseArea {
                    id: ma
                    anchors.fill: parent
                    hoverEnabled: true
                    cursorShape: Qt.PointingHandCursor
                    acceptedButtons: Qt.LeftButton | Qt.RightButton | Qt.MiddleButton
                    onClicked: m => {
                        if (m.button === Qt.RightButton) { chm.show(ma, m.x, m.y); return }
                        if (vc) { S.c.join(ch.rid); return }
                        S.open(ch.rid, (m.button === Qt.MiddleButton || (m.modifiers & Qt.ControlModifier)) ? 1 : 0)
                    }
                    onDoubleClicked: if (vc) S.open(ch.rid)
                }
                Ctx {
                    id: chm
                    model: [
                        { text: S.t("Mark as read"), run: () => S.c.ack(ch.rid) },
                        { text: S.t("Open in second chat"), run: () => S.open(ch.rid, 1) },
                        { text: S.t("Copy"), items: [
                            { text: S.t("Channel ID"), run: () => S.copy(S.raw(ch.rid)) },
                            { text: S.t("Link"), when: !!ch.permalink, run: () => S.copy(ch.permalink) }
                        ] }
                    ]
                }
            }
            Repeater {
                model: inV
                Row {
                    x: 36
                    height: 28
                    spacing: 8
                    property string uid: modelData.usr.id ? S.ridOf(modelData.usr.id) : ""
                    Av {
                        sz: 22
                        src: modelData.usr.av || ""
                        name: modelData.usr.name
                        speak: !!S.speaking[parent.uid]
                        anchors.verticalCenter: parent.verticalCenter
                    }
                    Lbl {
                        text: modelData.usr.disp || modelData.usr.name
                        color: Th.tx2
                        font.pixelSize: Th.fsS
                        anchors.verticalCenter: parent.verticalCenter
                    }
                    Ico { visible: modelData.mute || modelData.smute; n: "micoff"; sz: 14; col: Th.tx3; anchors.verticalCenter: parent.verticalCenter }
                    Ico { visible: modelData.deaf || modelData.sdeaf; n: "headoff"; sz: 14; col: Th.tx3; anchors.verticalCenter: parent.verticalCenter }
                    Rectangle {
                        visible: modelData.stream
                        width: lt.implicitWidth + 8
                        height: 16
                        radius: Th.rc
                        color: Th.err
                        anchors.verticalCenter: parent.verticalCenter
                        Lbl { id: lt; anchors.centerIn: parent; text: S.t("LIVE"); color: Th.wh; font.pixelSize: Th.fsP; font.weight: Font.Bold }
                        MouseArea { anchors.fill: parent; cursorShape: Qt.PointingHandCursor; onClicked: { S.c.watch(parent.parent.uid, true); S.open(ch.rid) } }
                    }
                }
            }
        }
    }

    Component {
        id: dmD
        Rectangle {
            property var u: S.unr[ch.rid] || null
            property bool cur: S.selC === ch.rid
            property var usr: ch.k === "Dm" && ch.rcp && ch.rcp.length ? ch.rcp[0] : null
            property var p: usr && usr.id ? S.presOf(usr.id) : null
            x: 8
            width: lv.width - 16
            height: 44 * Th.dens
            radius: 6
            color: cur ? Th.sel : ma.containsMouse ? Th.hov : "transparent"
            Behavior on color { ColorAnimation { duration: Th.mF } }
            Loader {
                anchors.fill: parent
                active: !!(parent.usr && parent.usr.plate) && S.ui.plates !== false && (parent.cur || ma.containsMouse)
                sourceComponent: Plate { src: usr.plate; radius: 6; anim: ma.containsMouse }
            }
            Av {
                id: av
                x: 8
                anchors.verticalCenter: parent.verticalCenter
                sz: 32
                src: parent.usr ? (parent.usr.av || "") : (ch.icon || "")
                name: S.chName(ch.rid)
                st: ch.k === "Dm" ? (parent.p ? parent.p.st : "Off") : ""
                ring: parent.cur ? Th.bg4 : Th.bg2
                sv: S.svc(ch.rid)
                deco: parent.usr && parent.usr.deco ? parent.usr.deco : ""
                anim: ma.containsMouse
            }
            Column {
                anchors.left: av.right
                anchors.leftMargin: 10
                anchors.right: parent.right
                anchors.rightMargin: 30
                anchors.verticalCenter: parent.verticalCenter
                spacing: -1
                Row {
                    width: parent.width
                    spacing: 6
                    Lbl {
                        width: Math.min(implicitWidth, parent.width - (dtp.visible ? dtp.width + 6 : 0))
                        text: S.chName(ch.rid)
                        color: cur || (u && u.n) || ma.containsMouse ? Th.tx : Th.tx2
                        font.pixelSize: Th.fs + 1
                        font.weight: u && u.n ? Font.DemiBold : Font.Normal
                        anchors.verticalCenter: parent.verticalCenter
                    }
                    TagPill { id: dtp; tag: usr && usr.tag ? usr.tag : null; fs: 10; anchors.verticalCenter: parent.verticalCenter }
                }
                Lbl {
                    width: parent.width
                    visible: text !== ""
                    text: ch.k === "Grp" ? (ch.rcp || []).length + 1 + " Members" : (p && p.act ? p.act : "")
                    color: cur ? Th.tx : Th.tx2
                    font.pixelSize: Th.fsS
                    font.weight: cur ? Font.DemiBold : Font.Normal
                }
            }
            Rectangle {
                visible: !!u && !!u.n && !(u.ment > 0) && !cur
                width: 6
                height: 6
                radius: 3
                color: Th.tx
                anchors.right: parent.right
                anchors.rightMargin: 12
                anchors.verticalCenter: parent.verticalCenter
            }
            Cnt {
                visible: !!u && u.ment > 0
                n: u ? u.ment : 0
                anchors.right: parent.right
                anchors.rightMargin: 8
                anchors.verticalCenter: parent.verticalCenter
            }
            Ico {
                id: xb
                visible: (cur || ma.containsMouse || xm.containsMouse) && !(u && u.ment > 0)
                z: 2
                n: "x"
                sz: 16
                col: xm.containsMouse ? Th.tx : Th.tx2
                anchors.right: parent.right
                anchors.rightMargin: 12
                anchors.verticalCenter: parent.verticalCenter
                MouseArea { id: xm; anchors.fill: parent; anchors.margins: -6; hoverEnabled: true; cursorShape: Qt.PointingHandCursor; onClicked: S.hideDm(ch.rid) }
            }
            Ico {
                visible: !xb.visible && (S.ui.pins || []).indexOf(ch.rid) >= 0 && !(u && (u.ment || u.n))
                n: "pin"
                sz: 14
                col: Th.tx3
                anchors.right: parent.right
                anchors.rightMargin: 10
                anchors.verticalCenter: parent.verticalCenter
            }
            MouseArea {
                id: ma
                anchors.fill: parent
                hoverEnabled: true
                cursorShape: Qt.PointingHandCursor
                acceptedButtons: Qt.LeftButton | Qt.RightButton | Qt.MiddleButton
                onClicked: m => {
                    if (m.button === Qt.RightButton) { dm.show(ma, m.x, m.y); return }
                    S.open(ch.rid, (m.button === Qt.MiddleButton || (m.modifiers & Qt.ControlModifier)) ? 1 : 0)
                }
            }
            Ctx {
                id: dm
                model: [
                    { text: (S.ui.pins || []).indexOf(ch.rid) >= 0 ? S.t("Unpin") : S.t("Pin to top"), when: S.f("pinDms"), run: () => {
                        const p = (S.ui.pins || []).slice()
                        const i = p.indexOf(ch.rid)
                        if (i >= 0) p.splice(i, 1); else p.unshift(ch.rid)
                        S.setUi("pins", p)
                        S.ver++
                    } },
                    { text: S.t("Mark as read"), run: () => S.c.ack(ch.rid) },
                    { text: S.t("Call"), when: S.caps(ch.rid).call, run: () => S.c.join(ch.rid) },
                    { text: S.t("Copy user ID"), run: () => S.copy(usr && usr.id ? usr.id.raw : "") }
                ]
            }
        }
    }
}
