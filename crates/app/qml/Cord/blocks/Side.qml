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
                Rectangle {
                    anchors.right: parent.right
                    anchors.rightMargin: 6
                    anchors.verticalCenter: parent.verticalCenter
                    width: kh.implicitWidth + 10
                    height: 18
                    radius: 4
                    color: Th.bg3
                    Lbl { id: kh; anchors.centerIn: parent; text: "Ctrl+K"; color: Th.tx3; font.pixelSize: 10; font.family: Th.mono }
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
                    width: parent.width - 60 - (U.demo ? 56 : 0)
                    text: r.g ? r.g.name : ""
                    font.weight: Font.DemiBold
                    anchors.verticalCenter: parent.verticalCenter
                }
            }
            Rectangle {
                visible: U.demo
                anchors.right: parent.right
                anchors.rightMargin: 14
                anchors.verticalCenter: parent.verticalCenter
                width: dc.implicitWidth + 12
                height: 18
                radius: 4
                color: Th.bg4
                Lbl { id: dc; anchors.centerIn: parent; text: "Demo"; color: Th.tx2; font.pixelSize: 11; font.weight: Font.DemiBold }
            }
            MouseArea {
                anchors.fill: parent
                acceptedButtons: Qt.RightButton
                onClicked: m => gm.show(parent, m.x, m.y)
            }
            Ctx {
                id: gm
                model: [
                    { text: "Mark server as read", run: () => { for (const c of S.chList(S.sel)) if (S.unr[c.rid]) S.c.ack(c.rid) } },
                    { text: "Copy server ID", run: () => S.copy(S.raw(S.sel)) },
                    { text: "Leave server", danger: true, when: typeof S.c.leave_gld === "function", run: () => { const g = S.sel; U.ask({ title: "Leave server", body: "Leave " + (r.g ? r.g.name : "this server") + "? You need a new invite to join again.", ok: "Leave", danger: true, run: () => S.c.leave_gld(g) }) } }
                ]
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
                text: "Direct messages"
                color: Th.tx2
                font.pixelSize: Th.fsS
                font.weight: Font.DemiBold
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
                    text: ch.name
                    color: cm.containsMouse ? Th.tx : Th.tx2
                    font.pixelSize: Th.fsS
                    font.weight: Font.DemiBold
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
                        visible: vc && S.call.chan && S.ridOf(S.call.chan) === ch.rid && !S.call.up
                        text: "Connecting"
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
                    Rectangle {
                        visible: !!u && u.ment > 0
                        height: 16
                        width: Math.max(16, mn.implicitWidth + 8)
                        radius: 8
                        color: Th.mr
                        anchors.verticalCenter: parent.verticalCenter
                        Lbl { id: mn; anchors.centerIn: parent; text: u ? u.ment : ""; color: "#fff"; font.pixelSize: 11; font.weight: Font.Bold }
                    }
                    Rectangle {
                        visible: !!u && !!u.n && !(u.ment > 0) && !cur
                        width: 6
                        height: 6
                        radius: 3
                        color: Th.tx
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
                        { text: "Mark as read", run: () => S.c.ack(ch.rid) },
                        { text: "Open in second chat", run: () => S.open(ch.rid, 1) },
                        { text: "Copy", items: [
                            { text: "Channel ID", run: () => S.copy(S.raw(ch.rid)) },
                            { text: "Link", when: !!ch.permalink, run: () => S.copy(ch.permalink) }
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
                visible: !!u && !!u.n && !(u.ment > 0) && !cur
                width: 6
                height: 6
                radius: 3
                color: Th.tx
                anchors.right: parent.right
                anchors.rightMargin: 12
                anchors.verticalCenter: parent.verticalCenter
            }
            Rectangle {
                visible: !!u && u.ment > 0
                height: 16
                width: Math.max(16, mn2.implicitWidth + 8)
                radius: 8
                color: Th.mr
                anchors.right: parent.right
                anchors.rightMargin: 8
                anchors.verticalCenter: parent.verticalCenter
                Lbl { id: mn2; anchors.centerIn: parent; text: u ? u.ment : ""; color: "#fff"; font.pixelSize: 11; font.weight: Font.Bold }
            }
            Ico {
                visible: (S.ui.pins || []).indexOf(ch.rid) >= 0 && !(u && (u.ment || u.n))
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
                    { text: (S.ui.pins || []).indexOf(ch.rid) >= 0 ? "Unpin" : "Pin to top", when: S.f("pinDms"), run: () => {
                        const p = (S.ui.pins || []).slice()
                        const i = p.indexOf(ch.rid)
                        if (i >= 0) p.splice(i, 1); else p.unshift(ch.rid)
                        S.setUi("pins", p)
                        S.ver++
                    } },
                    { text: "Mark as read", run: () => S.c.ack(ch.rid) },
                    { text: "Call", when: S.caps(ch.rid).call, run: () => S.c.join(ch.rid) },
                    { text: "Copy user ID", run: () => S.copy(usr && usr.id ? usr.id.raw : "") }
                ]
            }
        }
    }
}
