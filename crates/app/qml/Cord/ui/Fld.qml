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
    leftPadding: 10
    rightPadding: pw ? 38 : 10
    implicitHeight: 36 * Th.dens
    Keys.onShortcutOverride: e => { if ((e.modifiers & Qt.ControlModifier) && (e.key === Qt.Key_K || e.key === Qt.Key_Comma || e.key === Qt.Key_E)) e.accepted = false }
    background: Rectangle {
        radius: Th.rs
        color: Th.bg
        border.color: r.activeFocus ? Th.ac : Th.ln
        border.width: 1
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
