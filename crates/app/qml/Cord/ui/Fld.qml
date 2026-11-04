import QtQuick
import QtQuick.Controls
import Cord

TextField {
    id: r
    property bool pw: false
    property bool show: false
    color: Th.tx
    placeholderTextColor: Th.tx3
    font.family: Th.font
    font.pixelSize: Th.fs
    echoMode: pw && !show ? TextInput.Password : TextInput.Normal
    selectByMouse: true
    selectionColor: Th.ac
    selectedTextColor: Th.acTx
    leftPadding: 12
    rightPadding: pw ? 40 : 12
    implicitHeight: 38 * Th.dens
    Keys.onShortcutOverride: e => { if ((e.modifiers & Qt.ControlModifier) && (e.key === Qt.Key_K || e.key === Qt.Key_Comma || e.key === Qt.Key_E)) e.accepted = false }
    background: Rectangle {
        radius: Th.rs
        color: r.activeFocus ? Th.bg3 : Qt.darker(Th.bg3, Th.dark ? 1.12 : 1.0)
        border.color: r.activeFocus ? Th.a(Th.ac, 0.7) : Th.ln
        border.width: 1
        Behavior on border.color { ColorAnimation { duration: 120 } }
    }
    IBtn {
        visible: r.pw
        anchors.right: parent.right
        anchors.rightMargin: 4
        anchors.verticalCenter: parent.verticalCenter
        n: "eye"
        sz: 28
        isz: 16
        act: r.show
        tip: r.show ? "Hide" : "Show"
        activeFocusOnTab: false
        onClicked: r.show = !r.show
    }
}
