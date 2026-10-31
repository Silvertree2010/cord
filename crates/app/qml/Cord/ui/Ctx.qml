import QtQuick
import QtQuick.Controls
import Cord

Popup {
    id: r
    property var model: []
    property var view: null
    property real w: 196
    property int mods: 0
    readonly property var rows: {
        const src = view ? [{ back: true, text: "Back" }].concat(view) : model
        return src.filter(x => x && x.when !== false)
    }
    parent: Overlay.overlay
    padding: 4
    margins: 8
    width: w + 8
    closePolicy: Popup.CloseOnEscape | Popup.CloseOnPressOutside
    onAboutToShow: U.reg(r)
    onClosed: { U.unreg(r); view = null }

    function show(item, mx, my, md) {
        view = null
        mods = md || 0
        const p = item.mapToItem(Overlay.overlay, mx || 0, my || 0)
        x = p.x
        y = p.y
        open()
    }

    background: Rectangle {
        color: Th.bg3
        radius: Th.r
        border.color: Th.ln
        border.width: 1
    }

    contentItem: Column {
        spacing: 0
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
                    radius: Th.rs
                    color: ma.containsMouse ? Th.sel : "transparent"
                    Lbl {
                        x: 10
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
                        onClicked: m => {
                            r.mods = m.modifiers
                            if (modelData.back) r.view = null
                            else if (modelData.items) r.view = modelData.items
                            else {
                                r.close()
                                if (modelData.run) modelData.run()
                            }
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
