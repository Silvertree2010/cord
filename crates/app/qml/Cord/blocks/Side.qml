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
            Layout.preferredHeight: 48
            color: "transparent"
            Rectangle { anchors.bottom: parent.bottom; width: parent.width; height: 1; color: Th.ln }
            Loader {
                anchors.fill: parent
                sourceComponent: r.home ? homeHd : gHd
            }
        }
        ListView {
            id: lv
            Layout.fillWidth: true
            Layout.fillHeight: true
            clip: true
            topMargin: 8
            bottomMargin: 8
            model: r.home ? S.dms() : S.chList(S.sel).filter(c => c.k === "Cat" || !r.col[c.prid])
            ScrollBar.vertical: Scr {}
            header: r.home ? frHd : null
            delegate: Loader {
                width: lv.width
                property var ch: modelData
                sourceComponent: modelData.k === "Cat" ? catD : r.home ? dmD : chD
            }
        }
    }

    Component {
        id: homeHd
        Item {
            Rectangle {
                anchors.fill: parent
                anchors.margins: 10
                radius: Th.rs
                color: Th.bg
                Lbl {
                    anchors.verticalCenter: parent.verticalCenter
                    x: 10
                    text: "Find or start a conversation"
                    color: Th.tx3
                    font.pixelSize: Th.fsS
                }
                MouseArea { anchors.fill: parent; cursorShape: Qt.PointingHandCursor; onClicked: S.quick() }
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
                    sv: S.svc(S.sel)
                    txt: true
                    anchors.verticalCenter: parent.verticalCenter
                }
                Lbl {
                    width: parent.width - 60
                    text: r.g ? r.g.name : ""
                    font.weight: Font.DemiBold
                    anchors.verticalCenter: parent.verticalCenter
                }
            }
            MouseArea {
                anchors.fill: parent
                acceptedButtons: Qt.RightButton
                onClicked: gm.popup()
            }
            Menu {
                id: gm
                MenuItem { text: "Mark server as read"; onTriggered: { for (const c of S.chList(S.sel)) if (S.unr[c.rid]) S.c.ack(c.rid) } }
                MenuItem { text: "Copy server ID"; onTriggered: S.copy(S.raw(S.sel)) }
            }
        }
    }
    Component {
        id: frHd
        Column {
            width: lv.width
            spacing: 2
            Row_ { ico: "users"; text: "Friends"; cur: S.selC === ""; onPick: S.open("") }
            Row_ { ico: "inbox"; text: "Inbox"; cur: S.selC === "inbox"; onPick: S.open("inbox"); visible: !S.hasBlk("inbox") }
            Item { width: 1; height: 10 }
            Lbl {
                x: 16
                text: "DIRECT MESSAGES"
                color: Th.tx3
                font.pixelSize: 11
                font.weight: Font.DemiBold
                font.letterSpacing: 0.4
            }
            Item { width: 1; height: 4 }
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
        radius: Th.rs
        color: cur ? Th.sel : ma.containsMouse ? Th.hov : "transparent"
        Row {
            x: 10
            spacing: 12
            anchors.verticalCenter: parent.verticalCenter
            Ico { n: rw.ico; col: rw.cur ? Th.tx : Th.tx2; anchors.verticalCenter: parent.verticalCenter }
            Lbl { text: rw.text; color: rw.cur ? Th.tx : Th.tx2; font.weight: Font.Medium; anchors.verticalCenter: parent.verticalCenter }
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
                    color: cm.containsMouse ? Th.tx2 : Th.tx3
                    font.pixelSize: 11
                    font.weight: Font.DemiBold
                    font.letterSpacing: 0.4
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
                x: 8
                width: lv.width - 16
                height: 34 * Th.dens
                radius: Th.rs
                color: cur ? Th.sel : ma.containsMouse ? Th.hov : "transparent"
                opacity: ch.hidden ? 0.55 : 1
                Rectangle {
                    visible: u && u.n && !cur
                    width: 4
                    height: 8
                    radius: 2
                    x: -10
                    anchors.verticalCenter: parent.verticalCenter
                    color: Th.tx
                }
                Row {
                    x: 8
                    spacing: 6
                    anchors.verticalCenter: parent.verticalCenter
                    width: parent.width - 16
                    Ico {
                        n: ch.hidden ? "lock" : ch.k === "Voice" ? "voice" : ch.k === "Stage" ? "stage" : ch.k === "Forum" ? "forum" : ch.k === "Ann" ? "ann" : "hash"
                        col: Th.tx3
                        anchors.verticalCenter: parent.verticalCenter
                    }
                    Lbl {
                        width: parent.width - 60
                        text: ch.name
                        color: cur || (u && u.n) ? Th.tx : ma.containsMouse ? Th.tx2 : Th.tx3
                        font.weight: u && u.n ? Font.DemiBold : Font.Medium
                        anchors.verticalCenter: parent.verticalCenter
                    }
                }
                Row {
                    anchors.right: parent.right
                    anchors.rightMargin: 8
                    anchors.verticalCenter: parent.verticalCenter
                    spacing: 4
                    Lbl {
                        visible: ty.length > 0
                        text: "···"
                        color: Th.tx2
                        font.weight: Font.Bold
                    }
                    Rectangle {
                        visible: u && u.ment > 0
                        height: 16
                        width: Math.max(16, mn.implicitWidth + 8)
                        radius: 8
                        color: Th.err
                        Lbl { id: mn; anchors.centerIn: parent; text: u ? u.ment : ""; color: "#fff"; font.pixelSize: 11; font.weight: Font.Bold }
                    }
                }
                MouseArea {
                    id: ma
                    anchors.fill: parent
                    hoverEnabled: true
                    cursorShape: Qt.PointingHandCursor
                    acceptedButtons: Qt.LeftButton | Qt.RightButton | Qt.MiddleButton
                    onClicked: m => {
                        if (m.button === Qt.RightButton) { chm.popup(); return }
                        if (vc) { S.c.join(ch.rid); return }
                        S.open(ch.rid, (m.button === Qt.MiddleButton || (m.modifiers & Qt.ControlModifier)) ? 1 : 0)
                    }
                    onDoubleClicked: if (vc) S.open(ch.rid)
                }
                Menu {
                    id: chm
                    MenuItem { text: "Mark as read"; onTriggered: S.c.ack(ch.rid) }
                    MenuItem { text: "Open in second chat"; onTriggered: S.open(ch.rid, 1) }
                    MenuItem { text: "Copy channel ID"; onTriggered: S.copy(S.raw(ch.rid)) }
                    MenuItem { text: "Copy link"; visible: S.svc(ch.rid) === "dc"; onTriggered: S.copy("https://discord.com/channels/" + S.raw(ch.grid) + "/" + S.raw(ch.rid)) }
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
                        radius: 4
                        color: Th.err
                        anchors.verticalCenter: parent.verticalCenter
                        Lbl { id: lt; anchors.centerIn: parent; text: "LIVE"; color: "#fff"; font.pixelSize: 10; font.weight: Font.Bold }
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
            property var usr: ch.rcp && ch.rcp.length ? ch.rcp[0] : null
            property var p: usr && usr.id ? S.presOf(usr.id) : null
            x: 8
            width: lv.width - 16
            height: 44 * Th.dens
            radius: Th.rs
            color: cur ? Th.sel : ma.containsMouse ? Th.hov : "transparent"
            Av {
                id: av
                x: 8
                anchors.verticalCenter: parent.verticalCenter
                sz: 32
                src: parent.usr ? (parent.usr.av || "") : (ch.icon || "")
                name: S.chName(ch.rid)
                st: ch.k === "Dm" ? (parent.p ? parent.p.st : "Off") : ""
                ring: parent.color.a > 0 ? Th.bg3 : Th.bg2
                sv: S.svc(ch.rid)
            }
            Column {
                anchors.left: av.right
                anchors.leftMargin: 10
                anchors.right: parent.right
                anchors.rightMargin: 30
                anchors.verticalCenter: parent.verticalCenter
                Lbl {
                    width: parent.width
                    text: S.chName(ch.rid)
                    color: cur || (u && u.n) ? Th.tx : Th.tx2
                    font.weight: u && u.n ? Font.DemiBold : Font.Medium
                }
                Lbl {
                    width: parent.width
                    visible: text !== ""
                    text: ch.k === "Grp" ? (ch.rcp || []).length + 1 + " members" : (p && p.act ? p.act : "")
                    color: Th.tx3
                    font.pixelSize: Th.fsS - 1
                }
            }
            Rectangle {
                visible: u && u.ment > 0
                height: 16
                width: Math.max(16, mn2.implicitWidth + 8)
                radius: 8
                color: Th.err
                anchors.right: parent.right
                anchors.rightMargin: 8
                anchors.verticalCenter: parent.verticalCenter
                Lbl { id: mn2; anchors.centerIn: parent; text: u ? u.ment : ""; color: "#fff"; font.pixelSize: 11; font.weight: Font.Bold }
            }
            Ico {
                visible: (S.ui.pins || []).indexOf(ch.rid) >= 0 && !(u && u.ment)
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
                    if (m.button === Qt.RightButton) { dm.popup(); return }
                    S.open(ch.rid, (m.button === Qt.MiddleButton || (m.modifiers & Qt.ControlModifier)) ? 1 : 0)
                }
            }
            Menu {
                id: dm
                MenuItem {
                    text: (S.ui.pins || []).indexOf(ch.rid) >= 0 ? "Unpin" : "Pin to top"
                    visible: S.f("pinDms")
                    onTriggered: {
                        const p = (S.ui.pins || []).slice()
                        const i = p.indexOf(ch.rid)
                        if (i >= 0) p.splice(i, 1); else p.unshift(ch.rid)
                        S.setUi("pins", p)
                        S.ver++
                    }
                }
                MenuItem { text: "Mark as read"; onTriggered: S.c.ack(ch.rid) }
                MenuItem { text: "Call"; visible: S.svc(ch.rid) === "dc"; onTriggered: S.c.join(ch.rid) }
                MenuItem { text: "Copy user ID"; onTriggered: S.copy(usr && usr.id ? usr.id.raw : "") }
            }
        }
    }
}
