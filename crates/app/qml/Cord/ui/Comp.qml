import QtQuick
import QtQuick.Controls
import QtQuick.Dialogs
import QtQuick.Layouts
import Cord

Item {
    id: r
    property string cid: ""
    property int slot: 0
    property var ch: null
    property var fnd: null
    property var lastMine: null
    property var files: []
    property real lastTy: 0
    property bool prog: false
    property bool pending: false
    property string sentTxt: ""
    property string errMsg: ""
    property string saved: ""
    property string st_reply: S.slot(slot).reply || ""
    property string st_edit: S.slot(slot).editing || ""
    property var rep: st_reply && fnd ? fnd(st_reply) : null
    implicitHeight: cl.implicitHeight + 8
    readonly property real pad: Math.max(8, (48 - fm.height) / 2)
    FontMetrics { id: fm; font: ta.font }

    function setText(t) {
        prog = true
        ta.text = t
        prog = false
    }
    function clearMode() {
        if (S.slot(slot).editing) setText(saved)
        saved = ""
        S.setSlot(slot, "editing", "")
        S.setSlot(slot, "reply", "")
    }

    onSt_editChanged: {
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
            if (chan !== r.cid || !r.pending) return
            r.pending = false
            if (!ok) { r.errMsg = err || "Could not send"; return }
            r.errMsg = ""
            const wasEdit = !!S.slot(r.slot).editing
            if (ta.text.trim() === r.sentTxt) r.setText(wasEdit ? r.saved : "")
            else if (wasEdit) r.saved = ""
            r.saved = ""
            r.files = []
            S.setSlot(r.slot, "editing", "")
            S.setSlot(r.slot, "reply", "")
            const d = Object.assign({}, S.drafts)
            delete d[r.cid]
            S.drafts = d
        }
    }
    onCidChanged: {
        pending = false
        errMsg = ""
        saved = ""
        setText(S.drafts[cid] || "")
        files = []
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
            IBtn { anchors.right: parent.right; anchors.rightMargin: 6; anchors.verticalCenter: parent.verticalCenter; n: "x"; sz: 24; isz: 14; onClicked: r.clearMode() }
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
                        onClicked: { const f = r.files.slice(); f.splice(index, 1); r.files = f }
                    }
                }
            }
        }

        Rectangle {
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
                            else if (e.key === Qt.Key_Escape && (r.st_edit || r.st_reply)) e.accepted = true
                        }
                        Keys.onReleased: e => {
                            if (e.key === Qt.Key_QuoteLeft && !e.isAutoRepeat) { S.c.ptt(false); e.accepted = true }
                        }
                        onTextChanged: {
                            if (r.prog) return
                            r.errMsg = ""
                            if (!r.st_edit) {
                                const d = Object.assign({}, S.drafts)
                                d[r.cid] = text
                                S.drafts = d
                            }
                            if (text.length && !S.f("silenttyping") && Date.now() - r.lastTy > 8000 && !r.st_edit) {
                                r.lastTy = Date.now()
                                S.c.typing(r.cid)
                            }
                        }
                        Keys.onPressed: e => {
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
                                r.clearMode()
                                e.accepted = true
                            }
                        }
                    }
                }
                IBtn { n: "smile"; tip: "Emoji"; Layout.alignment: Qt.AlignTop; Layout.topMargin: 8; onClicked: S.emoji(this, e => { ta.insert(ta.cursorPosition, e.id ? "<" + (e.anim ? "a" : "") + ":" + e.name + ":" + e.id.raw + ">" : e.name); ta.forceActiveFocus() }) }
                IBtn { n: "send"; tip: "Send"; visible: ta.text.length > 0 || r.files.length > 0; Layout.alignment: Qt.AlignTop; Layout.topMargin: 8; onClicked: r.go() }
            }
            DropArea {
                anchors.fill: parent
                onDropped: d => { if (d.hasUrls) r.files = r.files.concat(d.urls.map(u => u.toString()).filter(u => u.startsWith("file:"))) }
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
                width: parent.width - 8 - (r.errMsg ? rb.width + 8 : 0)
                text: r.errMsg ? r.errMsg : parent.hid ? "You do not have access to this channel" : parent.ty.length === 0 ? "" : parent.ty.length > 3 ? "Several people are typing…" : parent.ty.map(t => t.n).join(", ") + (parent.ty.length > 1 ? " are typing…" : " is typing…")
                color: r.errMsg ? Th.err : parent.hid ? Th.tx3 : Th.tx2
                font.pixelSize: 12
                elide: Text.ElideRight
            }
            Lbl {
                id: rb
                visible: !!r.errMsg
                anchors.verticalCenter: parent.verticalCenter
                anchors.right: parent.right
                anchors.rightMargin: 4
                text: "Retry"
                color: Th.ac
                font.pixelSize: 12
                font.weight: Font.DemiBold
                MouseArea { anchors.fill: parent; anchors.margins: -4; cursorShape: Qt.PointingHandCursor; onClicked: r.go() }
            }
        }
    }

    FileDialog {
        id: fd
        fileMode: FileDialog.OpenFiles
        onAccepted: r.files = r.files.concat(selectedFiles.map(u => u.toString()))
    }

    function go() {
        if (pending) return
        const t = ta.text.trim()
        if (!t && !files.length) return
        errMsg = ""
        sentTxt = t
        pending = S.send(cid, t, files.map(f => decodeURIComponent(f)), slot)
    }
}
