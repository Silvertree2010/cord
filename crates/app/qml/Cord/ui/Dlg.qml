import QtQuick
import QtQuick.Controls
import Cord

Popup {
    id: r
    property string title: ""
    property string body: ""
    property string ok: "Confirm"
    property bool danger: false
    property var run: null
    parent: Overlay.overlay
    anchors.centerIn: Overlay.overlay
    modal: true
    width: 400
    padding: 20
    closePolicy: Popup.CloseOnEscape | Popup.CloseOnPressOutside
    onAboutToShow: U.reg(r)
    onOpened: (r.danger ? cb : okb).forceActiveFocus()
    onClosed: U.unreg(r)
    Overlay.modal: Rectangle { color: "#99000000" }
    function show(o) {
        title = o.title || ""
        body = o.body || ""
        ok = o.ok || "Confirm"
        danger = !!o.danger
        run = o.run || null
        open()
    }
    function go() {
        close()
        if (run) run()
    }
    background: Rectangle {
        color: Th.bg3
        radius: Th.r
        border.color: Th.ln
    }
    contentItem: Column {
        spacing: 12
        Keys.onEscapePressed: r.close()
        Lbl { width: parent.width; text: r.title; font.pixelSize: Th.fsL; font.weight: Font.DemiBold }
        Lbl { width: parent.width; visible: r.body !== ""; text: r.body; color: Th.tx2; wrapMode: Text.Wrap; elide: Text.ElideNone }
        Item { width: 1; height: 4 }
        Row {
            anchors.right: parent.right
            spacing: 8
            Btn { id: cb; text: "Cancel"; out: true; onClicked: r.close() }
            Btn { id: okb; text: r.ok; pri: !r.danger; dng: r.danger; onClicked: r.go() }
        }
    }
}
