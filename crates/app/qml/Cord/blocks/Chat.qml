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

    property bool failed: false
    property bool synced: false

    HoverHandler { onHoveredChanged: if (hovered) S.focusSlot = r.slot }

    function row(m, p) {
        const g = !!p && p.aname === m.auth.name && m.ts - p.ts < 420000 && !m.reply && !p.sys && !m.sys
        const d = !p || new Date(p.ts).toDateString() !== new Date(m.ts).toDateString()
        return {
            mid: S.ridOf(m.id), j: JSON.stringify(m), g: g && !d, d: d,
            aname: m.auth.name, aid: m.auth.id ? m.auth.id.raw : "", ts: m.ts, sys: !!m.sys, hasReply: !!m.reply, pin: !!m.pin
        }
    }
    function prow(i) {
        if (i < 0 || i >= mm.count) return null
        const x = mm.get(i)
        return { aname: x.aname, ts: x.ts, sys: x.sys }
    }
    function fill(list) {
        mm.clear()
        let p = null
        for (const m of list) {
            const w = row(m, p)
            mm.append(w)
            p = w
        }
        Qt.callLater(() => lv.positionViewAtEnd())
    }
    function prepend(list) {
        if (!list.length) return
        const top = lv.contentHeight - lv.contentY
        const seen = {}
        for (let i = 0; i < mm.count; i++) seen[mm.get(i).mid] = true
        const rows = []
        let p = null
        for (const m of list) {
            if (seen[S.ridOf(m.id)]) continue
            const w = row(m, p)
            rows.push(w)
            p = w
        }
        if (!rows.length) return
        for (let i = 0; i < rows.length; i++) mm.insert(i, rows[i])
        if (mm.count > rows.length) {
            const f = mm.get(rows.length)
            const w = row(JSON.parse(f.j), rows[rows.length - 1])
            mm.setProperty(rows.length, "g", w.g)
            mm.setProperty(rows.length, "d", w.d)
        }
        Qt.callLater(() => lv.contentY = lv.contentHeight - top)
    }
    function idx(id) {
        for (let i = mm.count - 1; i >= 0; i--) if (mm.get(i).mid === id) return i
        return -1
    }
    function find(id) {
        const i = idx(id)
        return i >= 0 ? JSON.parse(mm.get(i).j) : null
    }
    function pinned() {
        const o = []
        for (let i = 0; i < mm.count; i++) if (mm.get(i).pin) o.push(JSON.parse(mm.get(i).j))
        return o
    }
    function mineLast() {
        const me = S.me(S.acc(cid))
        if (!me || !me.id) return null
        for (let i = mm.count - 1; i >= 0; i--) if (mm.get(i).aid === me.id.raw) return JSON.parse(mm.get(i).j)
        return null
    }
    function more_() {
        if (!more || loading || failed || !cid || mm.count === 0) return
        loading = true
        S.c.hist(cid, mm.get(0).mid)
    }

    onCidChanged: {
        more = true
        loading = false
        failed = false
        synced = false
        mm.clear()
        if (!cid || cid === "inbox") return
        loading = true
        S.c.load_msgs(cid)
    }
    Component.onCompleted: cidChanged()

    Connections {
        target: S
        function onLoaded(chan, list) {
            if (chan !== r.cid) return
            r.fill(list)
            if (!r.synced) {
                r.synced = true
                r.loading = true
                S.c.hist(r.cid, "")
            } else {
                r.loading = false
                if (mm.count < 30) r.more_()
            }
        }
        function onMsg(chan, m, isNew) {
            if (chan !== r.cid) return
            const id = S.ridOf(m.id)
            const i = r.idx(id)
            const end = lv.atYEnd
            if (i >= 0) {
                mm.setProperty(i, "j", JSON.stringify(m))
                mm.setProperty(i, "pin", !!m.pin)
                return
            }
            if (!isNew) return
            const w = r.row(m, r.prow(mm.count - 1))
            mm.append(w)
            if (end || S.isMe(m.auth, S.acc(chan))) Qt.callLater(() => lv.positionViewAtEnd())
            if (Qt.application.state === Qt.ApplicationActive) S.c.ack(chan)
        }
        function onMsgDel(chan, id, gone) {
            if (chan !== r.cid) return
            const i = r.idx(id)
            if (i < 0) return
            if (gone) mm.remove(i)
            else {
                const m = JSON.parse(mm.get(i).j)
                m.del = true
                mm.setProperty(i, "j", JSON.stringify(m))
            }
        }
        function onHist(chan, n, mr, ok, list) {
            if (chan !== r.cid) return
            r.loading = false
            if (!ok) { r.failed = true; return }
            if (n < 50) r.more = false
            if (!mr) {
                if (list.length || mm.count === 0) S.c.load_msgs(r.cid)
            } else r.prepend(list)
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
                if ((moving || dragging) && contentY - originY < 400) r.more_()
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
            slot: r.slot
            lastMine: () => r.mineLast()
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
        onOpened: ms = r.pinned()
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
