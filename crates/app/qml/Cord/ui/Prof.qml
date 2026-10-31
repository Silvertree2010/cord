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
    background: Rectangle { color: Th.bg3; radius: Th.r; border.color: Th.ln }
    contentItem: Column {
        Rectangle { width: 300; height: 60; radius: Th.r; color: Th.svcCol(S.svc(r.uid)); opacity: 0.5 }
        Item {
            width: 300
            height: 48
            Av { x: 16; y: -36; sz: 80; src: r.u ? (r.u.av || "") : ""; name: r.u ? r.u.name : ""; st: r.p ? r.p.st : "Off"; ring: Th.bg3 }
            Badge { anchors.right: parent.right; anchors.rightMargin: 14; y: 10; sv: S.svc(r.uid); txt: true }
        }
        Column {
            x: 16
            width: 268
            spacing: 4
            bottomPadding: 16
            Lbl { width: parent.width; text: r.u ? (r.u.disp || r.u.name) : ""; font.pixelSize: Th.fsXL; font.weight: Font.Bold }
            Lbl { width: parent.width; text: r.u ? r.u.name : ""; color: Th.tx2 }
            Lbl { visible: !!(r.p && r.p.act); width: parent.width; text: r.p && r.p.act ? r.p.act : ""; color: Th.tx2; font.pixelSize: Th.fsS; topPadding: 8 }
            Row {
                spacing: 6
                topPadding: 10
                Btn {
                    text: "Message"
                    pri: true
                    onClicked: {
                        const d = S.dms().find(c => c.k === "Dm" && c.rcp && c.rcp[0] && c.rcp[0].id && S.ridOf(c.rcp[0].id) === r.uid)
                        if (d) { S.sel = "home"; S.open(d.rid) }
                        r.close()
                    }
                }
                Btn { text: "Copy ID"; onClicked: { S.copy(r.u.id ? r.u.id.raw : ""); r.close() } }
            }
        }
    }
}
