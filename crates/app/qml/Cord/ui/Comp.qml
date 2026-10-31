import QtQuick
import QtQuick.Controls
import QtQuick.Dialogs
import QtQuick.Layouts
import Cord
import "emo.js" as E

Item {
    id: r
    property string cid: ""
    property int slot: 0
    property var ch: null
    property var fnd: null
    property var lastMine: null
    property var fm: ({})
    property var out: ({})
    property real lastTy: 0
    property bool prog: false
    property bool emptyEdit: false
    property string saved: ""
    property string st_reply: S.slot(slot).reply || ""
    property string st_edit: S.slot(slot).editing || ""
    property var rep: st_reply && fnd ? fnd(st_reply) : null
    property var cmp: null
    property int cmpI: 0
    readonly property var files: fm[cid] || []
    readonly property var cur: out[cid] || null
    readonly property bool editFail: !!cur && cur.edit && cur.st === "fail"
    readonly property string stat: emptyEdit ? "Message can't be empty. Delete it?" : editFail ? cur.err : ""
    implicitHeight: cl.implicitHeight + 8
    readonly property real pad: Math.max(8, (48 - fm_.height) / 2)
    FontMetrics { id: fm_; font: ta.font }

    function setText(t) {
        prog = true
        ta.text = t
        prog = false
    }
    function setFiles(id, l) {
        const o = Object.assign({}, fm)
        o[id] = l
        fm = o
    }
    function setOut(id, v) {
        const o = Object.assign({}, out)
        if (v) o[id] = v
        else delete o[id]
        out = o
    }
    function clearDraft(id) {
        const d = Object.assign({}, S.drafts)
        delete d[id]
        S.drafts = d
    }
    function clearMode() {
        if (S.slot(slot).editing) setText(saved)
        saved = ""
        emptyEdit = false
        if (cur && cur.edit) setOut(cid, null)
        S.setSlot(slot, "editing", "")
        S.setSlot(slot, "reply", "")
    }
    function orig() {
        const m = st_edit && fnd ? fnd(st_edit) : null
        return m ? m.body : ""
    }
    function esc() {
        if (st_edit && ta.text !== orig()) {
            U.ask({ title: "Discard changes?", body: "Your edits to this message are not saved.", ok: "Discard", danger: true, run: () => r.clearMode() })
            return
        }
        clearMode()
    }

    onSt_editChanged: {
        emptyEdit = false
        if (!st_edit || !r.fnd) return
        const m = r.fnd(st_edit)
        if (!m) return
        if (!saved) saved = ta.text
        setText(m.body)
        ta.cursorPosition = ta.length
        ta.forceActiveFocus()
    }
    onSt_replyChanged: if (st_reply) ta.forceActiveFocus()

    Connections {
        target: S
        function onSent(chan, ok, err) {
            const e = r.out[chan]
            if (!e) return
            if (!ok) {
                r.setOut(chan, Object.assign({}, e, { st: "fail", err: err || "Could not send" }))
                return
            }
            r.setOut(chan, null)
            if (S.drafts[chan] === e.txt) r.clearDraft(chan)
            if (e.edit && chan === r.cid) {
                r.setText(r.saved)
                r.saved = ""
                S.setSlot(r.slot, "editing", "")
                S.setSlot(r.slot, "reply", "")
            }
        }
    }
    onCidChanged: {
        saved = ""
        emptyEdit = false
        cmp = null
        setText(S.drafts[cid] || "")
        ta.forceActiveFocus()
    }

    function upd() {
        if (r.prog || !r.cid) { cmp = null; return }
        const b = ta.getText(0, ta.cursorPosition)
        let m = /(^|\s)@([^\s@]*)$/.exec(b)
        if (m) {
            const q = m[2].toLowerCase()
            const dc = S.svc(r.cid) === "dc"
            const l = U.memList(r.cid).filter(x => !q || x.name.toLowerCase().indexOf(q) >= 0)
                .sort((a, c) => (a.name.toLowerCase().indexOf(q) === 0 ? 0 : 1) - (c.name.toLowerCase().indexOf(q) === 0 ? 0 : 1) || a.name.localeCompare(c.name))
                .slice(0, 8)
                .map(x => ({ t: x.name, av: x.usr.av || "", ins: dc && x.usr.id ? "<@" + x.usr.id.raw + "> " : "@" + x.name + " " }))
            cmp = l.length ? { st: b.length - m[2].length - 1, l: l, k: "@" } : null
            cmpI = 0
            return
        }
        m = /(^|\s):([\w+\-]{2,})$/.exec(b)
        if (m) {
            const q = m[2].toLowerCase()
            const l = []
            for (const g of S.glds) for (const e of g.emos || []) {
                if (l.length >= 8) break
                if (e.name.toLowerCase().indexOf(q) >= 0) l.push({ t: ":" + e.name + ":", url: e.url || "", ins: e.id ? "<" + (e.anim ? "a" : "") + ":" + e.name + ":" + e.id.raw + "> " : "" })
            }
            const ws = x => x[1].split(" ").some(w => w.indexOf(q) === 0)
            for (const x of E.all.filter(ws).concat(E.all.filter(x => !ws(x) && x[1].indexOf(q) >= 0))) {
                if (l.length >= 8) break
                l.push({ t: x[1], g: x[0], ins: x[0] + " " })
            }
            cmp = l.length ? { st: b.length - m[2].length - 1, l: l, k: ":" } : null
            cmpI = 0
            return
        }
        cmp = null
    }
    function accept(i) {
        if (!cmp || !cmp.l[i]) return
        const x = cmp.l[i]
        const st = cmp.st
        const en = ta.cursorPosition
        cmp = null
        ta.remove(st, en)
        ta.insert(st, x.ins)
        ta.forceActiveFocus()
    }

    Column {
        id: cl
        x: 16
        width: parent.width - 32
        spacing: 0

        Rectangle {
            visible: !!r.st_reply || !!r.st_edit
            width: parent.width
            height: 34
            radius: Th.r
            color: Th.bg3
            Rectangle { anchors.bottom: parent.bottom; width: parent.width; height: Th.r; color: Th.bg3 }
            Lbl {
                x: 14
                anchors.verticalCenter: parent.verticalCenter
                width: parent.width - 60
                text: r.st_edit ? "Editing message" : "Replying to " + (r.rep ? (r.rep.auth.disp || r.rep.auth.name) : "")
                color: Th.tx2
                font.pixelSize: Th.fsS
            }
            IBtn { anchors.right: parent.right; anchors.rightMargin: 6; anchors.verticalCenter: parent.verticalCenter; n: "x"; sz: 24; isz: 14; onClicked: r.esc() }
        }

        Rectangle {
            visible: !!r.cur && !r.cur.edit
            width: parent.width
            height: visible ? pr.implicitHeight + 16 : 0
            radius: Th.r
            color: Th.bg3
            opacity: r.cur && r.cur.st === "fail" ? 1 : 0.6
            border.color: r.cur && r.cur.st === "fail" ? Th.err : "transparent"
            Row {
                id: pr
                x: 14
                y: 8
                width: parent.width - 28
                spacing: 10
                Column {
                    width: parent.width - (r.cur && r.cur.st === "fail" ? 140 : 0)
                    spacing: 2
                    Lbl { width: parent.width; text: r.cur ? (r.cur.txt || (r.cur.files.length + " attachment" + (r.cur.files.length === 1 ? "" : "s"))) : ""; elide: Text.ElideRight; maximumLineCount: 2; wrapMode: Text.Wrap }
                    Lbl { width: parent.width; text: r.cur && r.cur.st === "fail" ? r.cur.err : "Sending…"; color: r.cur && r.cur.st === "fail" ? Th.err : Th.tx3; font.pixelSize: 12 }
                }
                Row {
                    visible: !!r.cur && r.cur.st === "fail"
                    spacing: 4
                    anchors.verticalCenter: parent.verticalCenter
                    Btn { text: "Retry"; height: 28; pri: true; onClicked: r.retry() }
                    Btn { text: "Discard"; height: 28; out: true; onClicked: r.discard() }
                }
            }
        }

        Flow {
            visible: r.files.length > 0
            width: parent.width
            spacing: 8
            topPadding: 8
            bottomPadding: 8
            Repeater {
                model: r.files
                Rectangle {
                    width: 120
                    height: 120
                    radius: Th.rs
                    color: Th.bg3
                    Image {
                        anchors.fill: parent
                        anchors.margins: 6
                        source: /\.(png|jpe?g|gif|webp)$/i.test(modelData) ? modelData : ""
                        fillMode: Image.PreserveAspectCrop
                        sourceSize: Qt.size(220, 220)
                    }
                    Lbl {
                        anchors.bottom: parent.bottom
                        anchors.margins: 6
                        x: 6
                        width: parent.width - 12
                        text: decodeURIComponent(modelData.split("/").pop())
                        font.pixelSize: 11
                    }
                    IBtn {
                        anchors.right: parent.right
                        n: "trash"
                        sz: 26
                        isz: 14
                        red: true
                        onClicked: { const f = r.files.slice(); f.splice(index, 1); r.setFiles(r.cid, f) }
                    }
                }
            }
        }

        Rectangle {
            id: bx
            width: parent.width
            height: Math.max(48, Math.min(ta.contentHeight + r.pad * 2, 300))
            radius: Th.r
            color: Th.bg3
            border.color: Th.ln
            border.width: 1
            RowLayout {
                anchors.fill: parent
                anchors.leftMargin: 7
                anchors.rightMargin: 7
                spacing: 2
                IBtn { n: "plus"; tip: "Upload a file"; Layout.alignment: Qt.AlignTop; Layout.topMargin: 8; onClicked: fd.open() }
                ScrollView {
                    Layout.fillWidth: true
                    Layout.fillHeight: true
                    TextArea {
                        id: ta
                        wrapMode: TextEdit.Wrap
                        color: Th.tx
                        font.family: Th.font
                        font.pixelSize: Th.fs
                        placeholderText: r.cid ? "Message " + (r.ch && (r.ch.k === "Dm" || r.ch.k === "Grp") ? "@" : "#") + S.chName(r.cid) : ""
                        placeholderTextColor: Th.tx3
                        selectionColor: Th.ac
                        selectedTextColor: Th.acTx
                        background: null
                        topPadding: r.pad
                        bottomPadding: r.pad
                        enabled: !(r.ch && r.ch.hidden)
                        Keys.onShortcutOverride: e => {
                            if ((e.modifiers & Qt.ControlModifier) && (e.key === Qt.Key_K || e.key === Qt.Key_Comma || e.key === Qt.Key_E)) e.accepted = false
                            else if (e.key === Qt.Key_Escape && (r.st_edit || r.st_reply || r.cmp)) e.accepted = true
                        }
                        Keys.onReleased: e => {
                            if (e.key === Qt.Key_QuoteLeft && !e.isAutoRepeat) { S.c.ptt(false); e.accepted = true }
                        }
                        onCursorPositionChanged: r.upd()
                        onTextChanged: {
                            if (r.prog) return
                            r.emptyEdit = false
                            if (!r.st_edit) {
                                const d = Object.assign({}, S.drafts)
                                d[r.cid] = text
                                S.drafts = d
                            }
                            if (text.length && !S.f("silenttyping") && Date.now() - r.lastTy > 8000 && !r.st_edit) {
                                r.lastTy = Date.now()
                                S.c.typing(r.cid)
                            }
                            r.upd()
                        }
                        Keys.onPressed: e => {
                            if (r.cmp) {
                                if (e.key === Qt.Key_Down) { r.cmpI = (r.cmpI + 1) % r.cmp.l.length; e.accepted = true; return }
                                if (e.key === Qt.Key_Up) { r.cmpI = (r.cmpI + r.cmp.l.length - 1) % r.cmp.l.length; e.accepted = true; return }
                                if (e.key === Qt.Key_Tab || e.key === Qt.Key_Return || e.key === Qt.Key_Enter) { r.accept(r.cmpI); e.accepted = true; return }
                                if (e.key === Qt.Key_Escape) { r.cmp = null; e.accepted = true; return }
                            }
                            if ((e.modifiers & Qt.ControlModifier) && e.key === Qt.Key_QuoteLeft) { if (!e.isAutoRepeat) S.c.ptt(true); e.accepted = true }
                            else if ((e.modifiers & Qt.ControlModifier) && e.key === Qt.Key_K) { S.qk = !S.qk; e.accepted = true }
                            else if ((e.modifiers & Qt.ControlModifier) && e.key === Qt.Key_Comma) { S.showSet = !S.showSet; e.accepted = true }
                            else if ((e.modifiers & Qt.ControlModifier) && e.key === Qt.Key_E) { S.edit = !S.edit; e.accepted = true }
                            else if ((e.key === Qt.Key_Return || e.key === Qt.Key_Enter) && !(e.modifiers & Qt.ShiftModifier)) {
                                e.accepted = true
                                r.go()
                            } else if (e.key === Qt.Key_Up && text === "" && r.lastMine) {
                                const m = r.lastMine()
                                if (m) { S.setSlot(r.slot, "editing", S.ridOf(m.id)); e.accepted = true }
                            } else if (e.key === Qt.Key_Escape && (r.st_edit || r.st_reply)) {
                                r.esc()
                                e.accepted = true
                            }
                        }
                    }
                }
                IBtn { n: "smile"; tip: "Emoji"; Layout.alignment: Qt.AlignTop; Layout.topMargin: 8; onClicked: S.emoji(bx, e => { ta.insert(ta.cursorPosition, e.id ? "<" + (e.anim ? "a" : "") + ":" + e.name + ":" + e.id.raw + ">" : e.name); ta.forceActiveFocus() }) }
                IBtn { n: "send"; tip: "Send"; visible: ta.text.length > 0 || r.files.length > 0; Layout.alignment: Qt.AlignTop; Layout.topMargin: 8; onClicked: r.go() }
            }
            DropArea {
                anchors.fill: parent
                onDropped: d => { if (d.hasUrls) r.setFiles(r.cid, r.files.concat(d.urls.map(u => u.toString()).filter(u => u.startsWith("file:")))) }
            }

            Rectangle {
                id: pop
                visible: !!r.cmp
                z: 30
                x: 0
                y: -height - 6
                width: Math.min(parent.width, 320)
                height: (r.cmp ? r.cmp.l.length : 0) * 30 + 8
                radius: Th.r
                color: Th.bg3
                border.color: Th.ln
                Column {
                    x: 4
                    y: 4
                    width: parent.width - 8
                    Repeater {
                        model: r.cmp ? r.cmp.l : []
                        Rectangle {
                            width: parent.width
                            height: 30
                            radius: 4
                            color: index === r.cmpI || pm.containsMouse ? Th.sel : "transparent"
                            Item {
                                id: pi
                                x: 8
                                width: 20
                                height: parent.height
                                Image { visible: !!modelData.url; anchors.centerIn: parent; width: 20; height: 20; source: modelData.url || ""; sourceSize: Qt.size(40, 40); asynchronous: true }
                                Text { visible: !!modelData.g; anchors.centerIn: parent; text: modelData.g || ""; font.pixelSize: 16 }
                                Av { visible: !!modelData.av || (r.cmp && r.cmp.k === "@"); anchors.centerIn: parent; sz: 20; src: modelData.av || ""; name: modelData.t || "" }
                            }
                            Lbl { anchors.left: pi.right; anchors.leftMargin: 8; anchors.right: parent.right; anchors.rightMargin: 8; anchors.verticalCenter: parent.verticalCenter; text: modelData.t }
                            MouseArea { id: pm; anchors.fill: parent; hoverEnabled: true; cursorShape: Qt.PointingHandCursor; onEntered: r.cmpI = index; onClicked: r.accept(index) }
                        }
                    }
                }
            }
        }

        Item {
            width: parent.width
            height: 18
            property var ty: Object.values(S.typing[r.cid] || {})
            property bool hid: !!(r.ch && r.ch.hidden)
            Lbl {
                id: sl
                anchors.verticalCenter: parent.verticalCenter
                x: 4
                width: parent.width - 8 - (r.stat ? rb.width + 8 : 0)
                text: r.stat ? r.stat : parent.hid ? "You do not have access to this channel" : parent.ty.length === 0 ? "" : parent.ty.length > 3 ? "Several people are typing…" : parent.ty.map(t => t.n).join(", ") + (parent.ty.length > 1 ? " are typing…" : " is typing…")
                color: r.stat ? (r.emptyEdit ? Th.tx2 : Th.err) : parent.hid ? Th.tx3 : Th.tx2
                font.pixelSize: 12
                elide: Text.ElideRight
            }
            Lbl {
                id: rb
                visible: !!r.stat
                anchors.verticalCenter: parent.verticalCenter
                anchors.right: parent.right
                anchors.rightMargin: 4
                text: r.emptyEdit ? "Delete" : "Retry"
                color: r.emptyEdit ? Th.err : Th.ac
                font.pixelSize: 12
                font.weight: Font.DemiBold
                MouseArea { anchors.fill: parent; anchors.margins: -4; cursorShape: Qt.PointingHandCursor; onClicked: r.emptyEdit ? r.delEdit() : r.retry() }
            }
        }
    }

    FileDialog {
        id: fd
        fileMode: FileDialog.OpenFiles
        onAccepted: r.setFiles(r.cid, r.files.concat(selectedFiles.map(u => u.toString())))
    }

    function delEdit() {
        const id = st_edit
        const c = cid
        if (!id) return
        U.ask({ title: "Delete message", body: "This message will be deleted for everyone in the conversation.", ok: "Delete", danger: true, run: () => { S.c.del(c, id); r.clearMode() } })
    }
    function discard() {
        const e = cur
        if (!e) return
        if (!ta.text && !e.edit) setText(e.txt)
        if (!r.files.length && e.files.length) setFiles(cid, e.files.map(f => f.startsWith("/") ? "file://" + f : f))
        setOut(cid, null)
        ta.forceActiveFocus()
    }
    function retry() {
        const e = cur
        if (!e || e.st !== "fail") return
        const id = cid
        const keepRep = S.slot(slot).reply
        if (!e.edit) S.setSlot(slot, "reply", e.reply)
        const ok = S.send(id, e.txt, e.files, slot)
        if (!e.edit) S.setSlot(slot, "reply", keepRep || "")
        if (ok) setOut(id, Object.assign({}, e, { st: "sending", err: "" }))
    }
    function go() {
        if (cur) return
        const t = ta.text.trim()
        if (!t && !files.length) {
            if (st_edit) emptyEdit = true
            return
        }
        const id = cid
        const ed = !!st_edit
        const fd_ = files.map(f => decodeURIComponent(f))
        const rp = S.slot(slot).reply || ""
        if (!S.send(id, t, fd_, slot)) return
        setOut(id, { txt: t, files: fd_, reply: rp, st: "sending", err: "", edit: ed })
        if (!ed) {
            setText("")
            clearDraft(id)
            setFiles(id, [])
            S.setSlot(slot, "reply", "")
        }
    }
}
