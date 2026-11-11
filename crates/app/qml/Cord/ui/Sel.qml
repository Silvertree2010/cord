import QtQuick
import Cord

Rectangle {
    id: r
    property var opts: []
    property string cur: ""
    property real mw: 260
    signal pick(string v)
    readonly property string label: {
        const o = opts.find(x => x[0] === cur)
        return S.t(o ? o[1] : opts.length ? opts[0][1] : "")
    }
    implicitWidth: mw
    implicitHeight: 32
    radius: Th.rs
    color: ma.containsMouse || m.opened ? Th.bg4 : Th.bg3
    border.color: m.opened ? Th.ac : Th.ln
    border.width: 1
    Behavior on color { ColorAnimation { duration: Th.mF } }
    activeFocusOnTab: true
    Accessible.role: Accessible.ComboBox
    Accessible.name: label
    Keys.onReturnPressed: ma.clicked(null)
    Keys.onSpacePressed: ma.clicked(null)
    Lbl { anchors.left: parent.left; anchors.leftMargin: 10; anchors.right: ch.left; anchors.rightMargin: 6; anchors.verticalCenter: parent.verticalCenter; text: r.label; font.pixelSize: Th.fsS + 1; elide: Text.ElideRight }
    Ico { id: ch; anchors.right: parent.right; anchors.rightMargin: 8; anchors.verticalCenter: parent.verticalCenter; n: "chev"; sz: 16; col: Th.tx2; rotation: m.opened ? 180 : 0; Behavior on rotation { NumberAnimation { duration: Th.mF; easing.type: Easing.OutCubic } } }
    Rectangle { anchors.fill: parent; radius: parent.radius; color: "transparent"; border.color: Th.ac; border.width: 2; visible: parent.activeFocus }
    MouseArea { id: ma; anchors.fill: parent; hoverEnabled: true; cursorShape: Qt.PointingHandCursor; onClicked: m.show(r, 0, r.height + 4) }
    Ctx {
        id: m
        w: r.width - 8
        model: r.opts.map(o => ({ text: S.t(o[1]), run: () => r.pick(o[0]) }))
    }
}
