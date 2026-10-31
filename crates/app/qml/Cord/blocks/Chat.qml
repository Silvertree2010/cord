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
    property bool failed: false
    property string failMsg: ""
    property string failKind: ""
    property int mark: 0
    property string markId: ""
    property int below: 0
    property var arr: []
    readonly property var nil: ({ auth: { name: "", bot: false }, body: "", ts: 0, atts: [], embs: [], reacts: [], hist: [], stks: [] })
    implicitWidth: 600
    implicitHeight: 400
    color: Th.bg2

    ListModel { id: mm }

    HoverHandler { onHoveredChanged: if (hovered) S.focusSlot = r.slot }

    function mk(m, p) {
        const mid = S.ridOf(m.id)
        const g = !!p && p.aname === m.auth.name && m.ts - p.ts < 420000 && !m.reply && !p.sys && !m.sys
        const d = !p || new Date(p.ts).toDateString() !== new Date(m.ts).toDateString()
        return { m: m, mid: mid, g: g && !d, d: d, aname: m.auth.name, aid: m.auth.id ? m.auth.id.raw : "", ts: m.ts, sys: !!m.sys }
    }
    function entry(w, sec) { return { mid: w.mid, rev: 0, sec: sec } }
    function fill(list, stick) {
        const rows = []
        const secs = []
        let p = null
        let seen = false
        if (!r.markId && r.mark > 0 && r.mark < list.length) r.markId = S.ridOf(list[list.length - r.mark].id)
        for (const m of list) {
            const w = mk(m, p)
            if (w.mid === r.markId) seen = true
            rows.push(w)
            secs.push(seen ? "1" : "0")
            p = w
        }
        mm.clear()
        arr = rows
        for (let i = 0; i < rows.length; i++) mm.append(entry(rows[i], secs[i]))
        if (stick) Qt.callLater(() => lv.positionViewAtEnd())
    }
    function prepend(list) {
        if (!list.length) return
        const have = {}
        for (const x of arr) have[x.mid] = true
        const rows = []
        let p = null
        for (const m of list) {
            if (have[S.ridOf(m.id)]) continue
            const w = mk(m, p)
            rows.push(w)
            p = w
        }
        if (!rows.length) return
        const top = lv.contentHeight - lv.contentY
        const first = arr.length ? arr[0] : null
        if (first) {
            const w = mk(first.m, rows[rows.length - 1])
            first.g = w.g
            first.d = w.d
        }
        arr = rows.concat(arr)
        for (let i = 0; i < rows.length; i++) mm.insert(i, entry(rows[i], "0"))
        if (first) mm.setProperty(rows.length, "rev", mm.get(rows.length).rev + 1)
        Qt.callLater(() => lv.contentY = lv.contentHeight - top)
    }
    function relink() {
        let p = null
        let seen = false
        for (let i = 0; i < arr.length; i++) {
            const w = arr[i]
            const f = mk(w.m, p)
            if (w.mid === r.markId) seen = true
            const sec = seen ? "1" : "0"
            let ch = false
            if (w.g !== f.g || w.d !== f.d) { w.g = f.g; w.d = f.d; ch = true }
            if (mm.get(i).sec !== sec) mm.setProperty(i, "sec", sec)
            if (ch) mm.setProperty(i, "rev", mm.get(i).rev + 1)
            p = w
        }
    }
    function merge(list, stick) {
        if (!arr.length) { fill(list, stick); return }
        if (!list.length) return
        const ids = {}
        let lo = list[0].ts
        for (const m of list) {
            ids[S.ridOf(m.id)] = true
            if (m.ts < lo) lo = m.ts
        }
        for (let i = arr.length - 1; i >= 0; i--) {
            if (arr[i].ts >= lo && !ids[arr[i].mid]) {
                arr.splice(i, 1)
                mm.remove(i)
            }
        }
        const sorted = list.slice().sort((a, b) => a.ts - b.ts)
        for (const m of sorted) {
            const id = S.ridOf(m.id)
            const i = idx(id)
            if (i >= 0) {
                arr[i].m = m
                arr[i].ts = m.ts
                mm.setProperty(i, "rev", mm.get(i).rev + 1)
                continue
            }
            let j = arr.length
            while (j > 0 && arr[j - 1].ts > m.ts) j--
            const w = mk(m, j > 0 ? arr[j - 1] : null)
            arr.splice(j, 0, w)
            mm.insert(j, entry(w, "0"))
        }
        relink()
        if (stick) Qt.callLater(() => lv.positionViewAtEnd())
    }
    function idx(id) {
        for (let i = arr.length - 1; i >= 0; i--) if (arr[i].mid === id) return i
        return -1
    }
    function find(id) {
        const i = idx(id)
        return i >= 0 ? arr[i].m : null
    }
    function pinned() {
        return arr.filter(x => x.m.pin).map(x => x.m)
    }
    function mineLast() {
        const me = S.me(S.acc(cid))
        if (!me || !me.id) return null
        for (let i = arr.length - 1; i >= 0; i--) if (arr[i].aid === me.id.raw && !arr[i].m.del) return arr[i].m
        return null
    }
    function more_() {
        if (!more || loading || failed || !cid || arr.length === 0) return
        loading = true
        S.c.hist(cid, arr[0].mid)
    }
    function retry() {
        failed = false
        loading = true
        if (r.failKind === "old" && arr.length) S.c.hist(cid, arr[0].mid)
        else S.c.hist(cid, "")
    }
    function reset() {
        more = true
        loading = false
        failed = false
        failMsg = ""
        below = 0
        markId = ""
        mm.clear()
        arr = []
    }

    onCidChanged: {
        reset()
        if (!cid || cid === "inbox") return
        const u = S.unr[cid]
        mark = u ? u.n : 0
        loading = true
        S.c.load_msgs(cid)
    }
    Component.onCompleted: cidChanged()

    Connections {
        target: S
        function onLoaded(chan, list, synced) {
            if (chan !== r.cid) return
            r.fill(list, true)
            if (synced) {
                r.loading = false
                if (list.length < 50) r.more = false
                else Qt.callLater(() => { if (lv.contentHeight < lv.height) r.more_() })
            } else {
                r.loading = true
                S.c.hist(r.cid, "")
            }
        }
        function onMsg(chan, m, isNew) {
            if (chan !== r.cid) return
            const id = S.ridOf(m.id)
            const i = r.idx(id)
            if (i >= 0) {
                r.arr[i].m = m
                mm.setProperty(i, "rev", mm.get(i).rev + 1)
                return
            }
            if (!isNew) return
            const end = lv.nearEnd
            const w = r.mk(m, r.arr.length ? r.arr[r.arr.length - 1] : null)
            r.arr.push(w)
            mm.append(r.entry(w, r.markId ? "1" : "0"))
            if (end || S.isMe(m.auth, S.acc(chan))) Qt.callLater(() => lv.positionViewAtEnd())
            else r.below++
        }
        function onMsgDel(chan, id, gone) {
            if (chan !== r.cid) return
            const i = r.idx(id)
            if (i < 0) return
            if (gone) {
                r.arr.splice(i, 1)
                mm.remove(i)
            } else {
                r.arr[i].m = Object.assign({}, r.arr[i].m, { del: true })
                mm.setProperty(i, "rev", mm.get(i).rev + 1)
            }
        }
        function onHist(chan, n, mr, ok, list, err, noop) {
            if (chan !== r.cid) return
            if (noop) return
            r.loading = false
            if (!ok) {
                r.failed = true
                r.failMsg = err || "Could not load messages"
                r.failKind = mr ? "old" : "head"
                return
            }
            r.failed = false
            if (n < 50) r.more = false
            if (mr) r.prepend(list)
            else if (list.length) r.merge(list, lv.nearEnd)
        }
        function onResync() {
            if (!r.cid || r.cid === "inbox") return
            r.loading = true
            S.c.load_msgs(r.cid)
        }
        function onUpChanged() {
            if (!r.cid || r.cid === "inbox" || r.loading || S.up.indexOf(S.acc(r.cid)) < 0) return
            S.c.hist(r.cid, "")
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
            topMargin: 12
            bottomMargin: 4
            boundsBehavior: Flickable.StopAtBounds
            ScrollBar.vertical: Scr {}
            section.property: "sec"
            section.criteria: ViewSection.FullString
            section.labelPositioning: ViewSection.InlineLabels
            section.delegate: Item {
                required property string section
                width: lv.width
                height: section === "1" ? 28 : 0
                visible: section === "1"
                Rectangle { anchors.verticalCenter: parent.verticalCenter; x: 16; width: parent.width - 32; height: 1; color: Th.err }
                Rectangle {
                    anchors.centerIn: parent
                    width: nl.implicitWidth + 16
                    height: 18
                    radius: 9
                    color: Th.err
                    Lbl { id: nl; anchors.centerIn: parent; text: "New messages"; color: "white"; font.pixelSize: Th.fsS; font.weight: Font.DemiBold }
                }
            }
            header: Item {
                width: lv.width
                height: r.failed ? 64 : r.more ? 60 : wl.implicitHeight + 40
                BusyIndicator { anchors.centerIn: parent; running: r.loading; visible: r.more && !r.failed; width: 28; height: 28 }
                Row {
                    visible: r.failed
                    anchors.centerIn: parent
                    spacing: 12
                    Lbl { text: r.failMsg; color: Th.err; anchors.verticalCenter: parent.verticalCenter; width: Math.min(implicitWidth, lv.width - 140); elide: Text.ElideRight }
                    Btn { text: "Retry"; onClicked: r.retry() }
                }
                Column {
                    id: wl
                    visible: !r.more && !r.failed
                    x: 16
                    y: 24
                    spacing: 6
                    Lbl { text: r.ch && (r.ch.k === "Dm" || r.ch.k === "Grp") ? S.chName(r.cid) : "Welcome to #" + S.chName(r.cid); font.pixelSize: Th.fsXL + 6; font.weight: Font.Bold }
                    Lbl { text: "This is the start of the conversation."; color: Th.tx2 }
                }
            }
            footer: Item { width: 1; height: 18 }
            delegate: Msg {
                required property int index
                required property int rev
                readonly property var x_: { void rev; return r.arr[index] || null }
                m: x_ ? x_.m : r.nil
                grp: x_ ? x_.g : false
                day: x_ ? x_.d : false
                cid: r.cid
                rep: m.reply ? r.find(S.ridOf(m.reply)) : null
                onJump: id => { const i = r.idx(id); if (i >= 0) lv.positionViewAtIndex(i, ListView.Center) }
            }
            onContentYChanged: {
                if ((moving || dragging) && contentY - originY < 400) r.more_()
            }
            readonly property bool nearEnd: atYEnd || (originY + contentHeight + bottomMargin - height - contentY) < 30
            onNearEndChanged: if (nearEnd) r.below = 0
        }

        Item {
            Layout.fillWidth: true
            Layout.preferredHeight: 0
            z: 5
            Rectangle {
                visible: !lv.nearEnd && r.arr.length > 0
                anchors.bottom: parent.bottom
                anchors.horizontalCenter: parent.horizontalCenter
                anchors.bottomMargin: 12
                width: jl.implicitWidth + 24
                height: 30
                radius: 15
                color: r.below > 0 ? Th.ac : Th.bg4
                Lbl { id: jl; anchors.centerIn: parent; text: r.below > 0 ? r.below + " new " + (r.below === 1 ? "message" : "messages") + " - Jump to present" : "Jump to present"; color: r.below > 0 ? Th.acTx : Th.tx; font.pixelSize: Th.fsS }
                MouseArea { anchors.fill: parent; cursorShape: Qt.PointingHandCursor; onClicked: { lv.positionViewAtEnd(); r.below = 0 } }
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
