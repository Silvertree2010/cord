import QtQuick
import QtQuick.Controls
import QtQuick.Dialogs
import QtQuick.Layouts
import Cord

Item {
    id: r
    property string cid: ""
    property var ch: null
    property var fnd: null
    property var lastMine: null
    property var files: []
    property real lastTy: 0
    property var rep: S.reply && fnd ? fnd(S.reply) : null
    implicitHeight: cl.implicitHeight + 8

    Connections {
        target: S
        function onEditingChanged() {
            if (!S.editing || !r.fnd) return
            const m = r.fnd(S.editing)
            if (m) { ta.text = m.body; ta.cursorPosition = ta.length; ta.forceActiveFocus() }
        }
        function onReplyChanged() { if (S.reply) ta.forceActiveFocus() }
    }
    onCidChanged: { ta.text = S.drafts[cid] || ""; files = []; ta.forceActiveFocus() }

    Column {
        id: cl
        x: 16
        width: parent.width - 32
        spacing: 0

        Rectangle {
            visible: !!S.reply || !!S.editing
            width: parent.width
            height: 34
            radius: Th.r
            color: Th.bg3
            Rectangle { anchors.bottom: parent.bottom; width: parent.width; height: Th.r; color: Th.bg3 }
            Lbl {
                x: 14
                anchors.verticalCenter: parent.verticalCenter
                width: parent.width - 60
                text: S.editing ? "Editing message" : "Replying to " + (r.rep ? (r.rep.auth.disp || r.rep.auth.name) : "")
                color: Th.tx2
                font.pixelSize: Th.fsS
            }
            IBtn { anchors.right: parent.right; anchors.rightMargin: 6; anchors.verticalCenter: parent.verticalCenter; n: "x"; sz: 24; isz: 14; onClicked: { if (S.editing) ta.text = ""; S.reply = ""; S.editing = "" } }
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
            height: Math.max(46, Math.min(ta.implicitHeight + 16, 300))
            radius: Th.r
            color: Th.bg3
            RowLayout {
                anchors.fill: parent
                anchors.leftMargin: 6
                anchors.rightMargin: 6
                spacing: 2
                IBtn { n: "plus"; tip: "Upload a file"; Layout.alignment: Qt.AlignTop; Layout.topMargin: 7; onClicked: fd.open() }
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
                        topPadding: 13
                        bottomPadding: 13
                        enabled: !(r.ch && r.ch.hidden)
                        onTextChanged: {
                            const d = Object.assign({}, S.drafts)
                            d[r.cid] = text
                            S.drafts = d
                            if (text.length && !S.f("silenttyping") && Date.now() - r.lastTy > 8000 && !S.editing) {
                                r.lastTy = Date.now()
                                S.c.typing(r.cid)
                            }
                        }
                        Keys.onPressed: e => {
                            if ((e.key === Qt.Key_Return || e.key === Qt.Key_Enter) && !(e.modifiers & Qt.ShiftModifier)) {
                                e.accepted = true
                                r.go()
                            } else if (e.key === Qt.Key_Up && text === "" && r.lastMine) {
                                const m = r.lastMine()
                                if (m) { S.editing = S.ridOf(m.id); e.accepted = true }
                            } else if (e.key === Qt.Key_Escape && (S.editing || S.reply)) {
                                if (S.editing) text = ""
                                S.editing = ""
                                S.reply = ""
                                e.accepted = true
                            }
                        }
                    }
                }
                IBtn { n: "smile"; tip: "Emoji"; Layout.alignment: Qt.AlignTop; Layout.topMargin: 7; onClicked: S.emoji(this, e => { ta.insert(ta.cursorPosition, e.id ? "<" + (e.anim ? "a" : "") + ":" + e.name + ":" + e.id.raw + ">" : e.name); ta.forceActiveFocus() }) }
                IBtn { n: "send"; tip: "Send"; visible: ta.text.length > 0 || r.files.length > 0; Layout.alignment: Qt.AlignTop; Layout.topMargin: 7; onClicked: r.go() }
            }
            DropArea {
                anchors.fill: parent
                onDropped: d => { if (d.hasUrls) r.files = r.files.concat(d.urls.map(u => u.toString())) }
            }
        }

        Item {
            width: parent.width
            height: 22
            property var ty: Object.values(S.typing[r.cid] || {})
            Lbl {
                visible: parent.ty.length > 0
                anchors.verticalCenter: parent.verticalCenter
                x: 4
                text: parent.ty.length > 3 ? "Several people are typing…" : parent.ty.map(t => t.n).join(", ") + (parent.ty.length > 1 ? " are typing…" : " is typing…")
                color: Th.tx2
                font.pixelSize: 11
            }
            Lbl {
                visible: !!(r.ch && r.ch.hidden)
                anchors.verticalCenter: parent.verticalCenter
                text: "You do not have access to this channel"
                color: Th.tx3
                font.pixelSize: 11
            }
        }
    }

    FileDialog {
        id: fd
        fileMode: FileDialog.OpenFiles
        onAccepted: r.files = r.files.concat(selectedFiles.map(u => u.toString()))
    }

    function go() {
        const t = ta.text.trim()
        if (!t && !files.length) return
        S.send(cid, t, files.map(f => decodeURIComponent(f)))
        ta.text = ""
        files = []
    }
}
