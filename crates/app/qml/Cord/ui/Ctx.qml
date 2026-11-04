import QtQuick
import QtQuick.Controls
import Cord

Popup {
    id: r
    property var model: []
    property var view: null
    property real w: 196
    property int mods: 0
    property int cur: -1
    readonly property var rows: {
        const src = view ? [{ back: true, text: "Back" }].concat(view) : model
        const f = src.filter(x => x && x.when !== false)
        const o = []
        for (const x of f) {
            if (x.danger && o.length && !o[o.length - 1].sep) o.push({ sep: true })
            if (x.sep && (!o.length || o[o.length - 1].sep)) continue
            o.push(x)
        }
        while (o.length && o[o.length - 1].sep) o.pop()
        return o
    }
    parent: Overlay.overlay
    padding: 4
    margins: 8
    width: w + 8
    focus: true
    closePolicy: Popup.CloseOnEscape | Popup.CloseOnPressOutside
    onAboutToShow: U.reg(r)
    onClosed: { U.unreg(r); view = null; cur = -1 }
    onViewChanged: cur = -1

    function ok(i) {
        const x = rows[i]
        return !!x && !x.sep && !x.slider
    }
    function step(d) {
        let i = cur
        for (let k = 0; k < rows.length; k++) {
            i = (i + d + rows.length) % rows.length
            if (ok(i)) { cur = i; return }
        }
    }
    function act(x) {
        if (!x) return
        if (x.back) view = null
        else if (x.items) view = x.items
        else {
            close()
            if (x.run) x.run()
        }
    }

    function show(item, mx, my, md) {
        view = null
        cur = -1
        mods = md || 0
        const p = item.mapToItem(Overlay.overlay, mx || 0, my || 0)
        x = p.x
        y = p.y
        open()
    }

    background: Pane {}

    contentItem: Column {
        spacing: 0
        focus: true
        Keys.onPressed: e => {
            if (e.key === Qt.Key_Down) { r.step(1); e.accepted = true }
            else if (e.key === Qt.Key_Up) { r.step(-1); e.accepted = true }
            else if (e.key === Qt.Key_Right) { if (r.ok(r.cur) && r.rows[r.cur].items) { r.act(r.rows[r.cur]); r.step(1) } e.accepted = true }
            else if (e.key === Qt.Key_Left) { if (r.view) r.view = null; e.accepted = true }
            else if (e.key === Qt.Key_Return || e.key === Qt.Key_Enter || e.key === Qt.Key_Space) { if (r.ok(r.cur)) r.act(r.rows[r.cur]); e.accepted = true }
            else if (e.key === Qt.Key_Escape) { r.close(); e.accepted = true }
        }
        Repeater {
            model: r.rows
            delegate: Item {
                width: r.w
                height: modelData.sep ? 9 : modelData.slider ? 52 : 28
                Rectangle {
                    visible: !!modelData.sep
                    anchors.verticalCenter: parent.verticalCenter
                    x: 4
                    width: parent.width - 8
                    height: 1
                    color: Th.ln
                }
                Rectangle {
                    visible: !modelData.sep && !modelData.slider
                    anchors.fill: parent
                    radius: Th.rs - 2
                    color: index === r.cur || ma.containsMouse ? Th.sel : "transparent"
                    Lbl {
                        x: 8
                        width: parent.width - 36
                        anchors.verticalCenter: parent.verticalCenter
                        text: (modelData.back ? "‹  " : "") + modelData.text
                        color: modelData.danger ? Th.err : Th.tx
                    }
                    Ico {
                        visible: !!modelData.items
                        anchors.right: parent.right
                        anchors.rightMargin: 8
                        anchors.verticalCenter: parent.verticalCenter
                        n: "chevr"
                        sz: 14
                        col: Th.tx3
                    }
                    Ico {
                        visible: !!modelData.checked
                        anchors.right: parent.right
                        anchors.rightMargin: 8
                        anchors.verticalCenter: parent.verticalCenter
                        n: "check"
                        sz: 14
                        col: Th.ac
                    }
                    MouseArea {
                        id: ma
                        anchors.fill: parent
                        hoverEnabled: true
                        cursorShape: Qt.PointingHandCursor
                        onEntered: r.cur = index
                        onClicked: m => {
                            r.mods = m.modifiers
                            r.act(modelData)
                        }
                    }
                }
                Column {
                    visible: !!modelData.slider
                    x: 10
                    y: 6
                    width: parent.width - 20
                    spacing: 4
                    Lbl { text: modelData.text || ""; color: Th.tx2; font.pixelSize: Th.fsS }
                    Sld {
                        width: parent.width
                        from: modelData.from || 0
                        to: modelData.to || 1
                        value: modelData.value || 0
                        onMoved: if (modelData.run) modelData.run(value)
                    }
                }
            }
        }
    }
}
