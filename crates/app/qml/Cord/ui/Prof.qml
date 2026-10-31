import QtQuick
import QtQuick.Controls
import Cord

Popup {
    id: r
    property var u: null
    property string uid: u && u.id ? S.ridOf(u.id) : ""
    property var p: S.pres[uid] || null
    width: 300
    padding: 0
    closePolicy: Popup.CloseOnEscape | Popup.CloseOnPressOutside
    onAboutToShow: U.reg(r)
    onClosed: U.unreg(r)
    background: Rectangle { color: Th.bg3; radius: Th.r; border.color: Th.ln }
    contentItem: Column {
        Item {
            width: 300
            height: 60
            Rectangle { anchors.fill: parent; anchors.margins: 1; radius: Th.r - 1; color: Th.bg4 }
            Rectangle { anchors.bottom: parent.bottom; x: 1; width: parent.width - 2; height: Th.r; color: Th.bg4 }
            IBtn { anchors.right: parent.right; anchors.top: parent.top; anchors.margins: 6; n: "x"; sz: 28; isz: 16; tip: "Close"; onClicked: r.close() }
        }
        Item {
            width: 300
            height: 48
            Av { x: 16; y: -36; sz: 80; src: r.u ? (r.u.av || "") : ""; name: r.u ? r.u.name : ""; st: r.p ? r.p.st : "Off"; ring: Th.bg3 }
            Badge { anchors.right: parent.right; anchors.rightMargin: 14; y: 10; sv: S.svc(r.uid); txt: true; ring: Th.bg3 }
        }
        Column {
            x: 16
            width: 268
            spacing: 4
            bottomPadding: 16
            Lbl { width: parent.width; text: r.u ? (r.u.disp || r.u.name) : ""; font.pixelSize: Th.fsXL; font.weight: Font.Bold }
            Lbl { width: parent.width; text: r.u ? r.u.name : ""; color: Th.tx2 }
            Lbl { visible: !!(r.p && r.p.act); width: parent.width; text: r.p && r.p.act ? r.p.act : ""; color: Th.tx2; font.pixelSize: Th.fsS; topPadding: 8 }
            Item { width: 1; height: 8 }
            Row {
                spacing: 8
                Btn {
                    width: 130
                    height: 34
                    text: "Message"
                    pri: true
                    onClicked: {
                        const d = S.dms().find(c => c.k === "Dm" && c.rcp && c.rcp[0] && c.rcp[0].id && S.ridOf(c.rcp[0].id) === r.uid)
                        if (d) { S.sel = "home"; S.open(d.rid) }
                        r.close()
                    }
                }
                Btn { width: 130; height: 34; text: "Copy ID"; onClicked: { S.copy(r.u.id ? r.u.id.raw : ""); r.close() } }
            }
        }
    }
}
