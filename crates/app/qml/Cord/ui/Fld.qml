import QtQuick
import QtQuick.Controls
import Cord

TextField {
    id: r
    property bool pw: false
    color: Th.tx
    placeholderTextColor: Th.tx3
    font.family: Th.font
    font.pixelSize: Th.fs
    echoMode: pw ? TextInput.Password : TextInput.Normal
    selectByMouse: true
    selectionColor: Th.ac
    selectedTextColor: Th.acTx
    leftPadding: 10
    rightPadding: 10
    implicitHeight: 36 * Th.dens
    background: Rectangle {
        radius: Th.rs
        color: Th.bg
        border.color: r.activeFocus ? Th.ac : Th.ln
        border.width: 1
    }
}
