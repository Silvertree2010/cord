import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import Cord

Rectangle {
    id: r
    property var o: ({})
    property int slot: o.slot || 0
    property string cid: slot === 1 ? S.selC2 : S.selC
    property var ch: S.chans[cid] || null
    property bool more: true
    property bool loading: false
    implicitWidth: 600
    implicitHeight: 400
    color: Th.bg2

    ListModel { id: mm }

    function rd(keep) {
        const a = JSON.parse(S.c.msgs(cid) || "[]")
        const top = lv.contentHeight - lv.contentY
        mm.clear()
        let p = null
        for (const m of a) {
            mm.append(it(m, p))
            p = m
        }
        if (keep) Qt.callLater(() => lv.contentY = lv.contentHeight - top)
        else Qt.callLater(() => lv.positionViewAtEnd())
    }
    function it(m, p) {
        const g = !!p && p.auth.name === m.auth.name && m.ts - p.ts < 420000 && !m.reply && !p.sys && !m.sys
        const d = !p || new Date(p.ts).toDateString() !== new Date(m.ts).toDateString()
        return { mid: S.ridOf(m.id), j: JSON.stringify(m), g: g && !d, d: d }
    }
    function idx(id) {
        for (let i = mm.count - 1; i >= 0; i--) if (mm.get(i).mid === id) return i
        return -1
    }
    function find(id) {
        const i = idx(id)
        return i >= 0 ? JSON.parse(mm.get(i).j) : null
    }

    onCidChanged: {
        more = true
        loading = false
        mm.clear()
        if (!cid || cid === "inbox") return
        rd(false)
        if (mm.count < 30) { loading = true; S.c.hist(cid, mm.count ? mm.get(0).mid : "") }
    }
    Component.onCompleted: cidChanged()

    Connections {
        target: S
        function onMsg(chan, m, isNew) {
            if (chan !== r.cid) return
            const id = S.ridOf(m.id)
            const i = r.idx(id)
            const end = lv.atYEnd
            if (i >= 0) {
                mm.set(i, { mid: id, j: JSON.stringify(m) })
                return
            }
            if (!isNew) return
            const p = mm.count ? JSON.parse(mm.get(mm.count - 1).j) : null
            mm.append(r.it(m, p))
            if (end || S.isMe(m.auth, S.acc(chan))) Qt.callLater(() => lv.positionViewAtEnd())
            if (Qt.application.state === Qt.ApplicationActive && !S.f("noread")) S.c.ack(chan)
        }
        function onMsgDel(chan, id, gone) {
            if (chan !== r.cid) return
            const i = r.idx(id)
            if (i < 0) return
            if (gone) mm.remove(i)
            else {
                const m = JSON.parse(mm.get(i).j)
                m.del = true
                mm.set(i, { j: JSON.stringify(m) })
            }
        }
        function onHist(chan, n, mr) {
            if (chan !== r.cid) return
            r.loading = false
            if (n < 50) r.more = false
            r.rd(mr)
        }
    }

    ColumnLayout {
        anchors.fill: parent
        spacing: 0
        visible: !!r.cid && r.cid !== "inbox"

        Rectangle {
            Layout.fillWidth: true
            Layout.preferredHeight: 48
            color: "transparent"
            Rectangle { anchors.bottom: parent.bottom; width: parent.width; height: 1; color: Th.ln }
            RowLayout {
                anchors.fill: parent
                anchors.leftMargin: 16
                anchors.rightMargin: 8
                spacing: 8
                Ico {
                    n: !r.ch ? "hash" : r.ch.k === "Dm" ? "at" : r.ch.k === "Grp" ? "users" : r.ch.k === "Voice" ? "voice" : "hash"
                    col: Th.tx3
                }
                Lbl {
                    text: S.chName(r.cid)
                    font.weight: Font.DemiBold
                    Layout.maximumWidth: 260
                }
                Badge { sv: S.svc(r.cid); txt: true }
                Rectangle { visible: !!(r.ch && r.ch.topic); width: 1; height: 20; color: Th.ln }
                Lbl {
                    Layout.fillWidth: true
                    text: r.ch && r.ch.topic ? r.ch.topic.replace(/\n/g, " ") : ""
                    color: Th.tx3
                    font.pixelSize: Th.fsS
                }
                Item { Layout.fillWidth: !(r.ch && r.ch.topic) }
                IBtn { visible: !!r.ch && (r.ch.k === "Dm" || r.ch.k === "Grp"); n: "phone"; tip: "Start call"; onClicked: S.c.join(r.cid) }
                IBtn { n: "pin"; tip: "Pinned messages"; onClicked: pins.open() }
                IBtn {
                    n: "users"
                    tip: "Member list"
                    act: S.hasBlk("mems")
                    onClicked: S.toggleBlk("mems", !S.hasBlk("mems"))
                }
                IBtn { n: "search"; tip: "Search  (Ctrl+K)"; onClicked: S.quick() }
            }
        }

        ListView {
            id: lv
            Layout.fillWidth: true
            Layout.fillHeight: true
            clip: true
            model: mm
            cacheBuffer: 2000
            boundsBehavior: Flickable.StopAtBounds
            ScrollBar.vertical: Scr {}
            header: Item {
                width: lv.width
                height: r.more ? 60 : wl.implicitHeight + 40
                BusyIndicator { anchors.centerIn: parent; running: r.loading; visible: r.more; width: 28; height: 28 }
                Column {
                    id: wl
                    visible: !r.more
                    x: 16
                    y: 24
                    spacing: 6
                    Lbl { text: r.ch && (r.ch.k === "Dm" || r.ch.k === "Grp") ? S.chName(r.cid) : "Welcome to #" + S.chName(r.cid); font.pixelSize: Th.fsXL + 6; font.weight: Font.Bold }
                    Lbl { text: "This is the start of the conversation."; color: Th.tx2 }
                }
            }
            footer: Item { width: 1; height: 18 }
            delegate: Msg {
                required property string j
                required property bool g
                required property bool d
                required property int index
                m: JSON.parse(j)
                grp: g
                day: d
                cid: r.cid
                rep: m.reply ? r.find(S.ridOf(m.reply)) : null
                onJump: id => { const i = r.idx(id); if (i >= 0) lv.positionViewAtIndex(i, ListView.Center) }
            }
            onContentYChanged: {
                if (r.more && !r.loading && mm.count > 0 && contentY - originY < 400) {
                    r.loading = true
                    S.c.hist(r.cid, mm.get(0).mid)
                }
            }
            Rectangle {
                visible: !lv.atYEnd && mm.count > 0
                anchors.bottom: parent.bottom
                anchors.horizontalCenter: parent.horizontalCenter
                anchors.bottomMargin: 12
                width: jl.implicitWidth + 24
                height: 30
                radius: 15
                color: Th.bg4
                Lbl { id: jl; anchors.centerIn: parent; text: "Jump to present"; font.pixelSize: Th.fsS }
                MouseArea { anchors.fill: parent; cursorShape: Qt.PointingHandCursor; onClicked: lv.positionViewAtEnd() }
            }
        }

        Comp {
            Layout.fillWidth: true
            cid: r.cid
            ch: r.ch
            fnd: id => r.find(id)
            lastMine: () => {
                for (let i = mm.count - 1; i >= 0; i--) {
                    const m = JSON.parse(mm.get(i).j)
                    if (S.isMe(m.auth, S.acc(r.cid))) return m
                }
                return null
            }
        }
    }

    Loader {
        anchors.fill: parent
        active: !r.cid && r.slot === 0
        sourceComponent: Friends {}
    }
    Loader {
        anchors.fill: parent
        active: r.cid === "inbox"
        source: "Inbox.qml"
    }
    Item {
        anchors.fill: parent
        visible: !r.cid && r.slot === 1
        Lbl { anchors.centerIn: parent; text: "Middle click or Ctrl+click a channel to open it here"; color: Th.tx3 }
    }

    Popup {
        id: pins
        x: r.width - width - 16
        y: 52
        width: 420
        height: Math.min(r.height - 80, 560)
        padding: 0
        background: Rectangle { color: Th.bg3; radius: Th.r; border.color: Th.ln }
        property var ms: []
        onOpened: ms = JSON.parse(S.c.msgs(r.cid)).filter(m => m.pin)
        ColumnLayout {
            anchors.fill: parent
            spacing: 0
            Lbl { text: "Pinned messages"; font.weight: Font.DemiBold; Layout.margins: 14 }
            Sep { Layout.fillWidth: true }
            ListView {
                Layout.fillWidth: true
                Layout.fillHeight: true
                clip: true
                model: pins.ms
                delegate: Msg { m: modelData; cid: r.cid; width: 420 }
                Lbl { anchors.centerIn: parent; visible: pins.ms.length === 0; text: "No pinned messages in the loaded history"; color: Th.tx3 }
            }
        }
    }
}
