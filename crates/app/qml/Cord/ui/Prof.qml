import QtQuick
import QtQuick.Controls
import Cord

Popup {
    id: r
    property var u: null
    objectName: "prof"
    property string uid: u && u.id ? S.ridOf(u.id) : ""
    property var p: S.pres[uid] || null
    width: 300
    padding: 0
    closePolicy: Popup.CloseOnEscape | Popup.CloseOnPressOutside
    onAboutToShow: U.reg(r)
    onClosed: U.unreg(r)
    background: Pane {}
    contentItem: Item {
        implicitWidth: 300
        implicitHeight: col.implicitHeight + 32
        IBtn { anchors.right: parent.right; anchors.top: parent.top; anchors.margins: 6; n: "x"; sz: 28; isz: 16; tip: "Close"; onClicked: r.close() }
        Av {
            id: av
            x: 16
            y: 16
            sz: 64
            src: r.u ? (r.u.av || "") : ""
            name: r.u ? r.u.name : ""
            st: r.p ? r.p.st : "Off"
            ring: Th.bg3
        }
        Column {
            id: col
            x: 16
            y: 16
            width: 268
            spacing: 4
            Item { width: 1; height: av.sz + 8 }
            Row {
                spacing: 8
                width: parent.width
                Lbl { width: Math.min(implicitWidth, parent.width - bg.width - 8); text: r.u ? (r.u.disp || r.u.name) : ""; font.pixelSize: Th.fsL; font.weight: Font.Bold; anchors.verticalCenter: parent.verticalCenter }
                Rectangle {
                    id: bg
                    width: bl.implicitWidth + 12
                    height: 18
                    radius: 4
                    color: "transparent"
                    border.color: Th.ln
                    visible: S.ui.badges !== false
                    anchors.verticalCenter: parent.verticalCenter
                    Lbl { id: bl; anchors.centerIn: parent; text: Th.svcName(S.svc(r.uid)); color: Th.tx2; font.pixelSize: 11; font.weight: Font.Medium }
                }
            }
            Lbl { width: parent.width; visible: !!(r.u && r.u.disp && r.u.disp !== r.u.name); text: r.u ? r.u.name : ""; color: Th.tx2 }
            Lbl { visible: !!(r.p && r.p.act); width: parent.width; text: r.p && r.p.act ? r.p.act : ""; color: Th.tx2; font.pixelSize: Th.fsS; topPadding: 4 }
            Item { width: 1; height: 8 }
            Row {
                spacing: 8
                Btn {
                    width: 130
                    height: 34
                    text: "Message"
                    onClicked: { if (r.u) S.openDm(r.u); r.close() }
                }
                Btn { width: 130; height: 34; text: "Copy ID"; onClicked: { S.copy(r.u && r.u.id ? r.u.id.raw : ""); r.close() } }
            }
        }
    }
}
