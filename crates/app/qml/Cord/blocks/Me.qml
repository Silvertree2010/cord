import QtQuick
import QtQuick.Controls
import Cord

Rectangle {
    id: r
    property var o: ({})
    implicitWidth: 248
    implicitHeight: 56
    color: Th.bg3
    property var al: Object.values(S.accs)
    property int cur: S.ui.meAcc && S.accs[S.ui.meAcc] ? S.ui.meAcc : (al.length ? al[0].id : 0)
    property var a: S.accs[cur] || null
    property var mu: S.call.mute || false
    property var de: S.call.deaf || false
    Row {
        x: 8
        anchors.verticalCenter: parent.verticalCenter
        spacing: 8
        width: parent.width - btns.width - 16
        Av {
            id: av
            sz: 34
            src: r.a ? (r.a.me.av || "") : ""
            name: r.a ? r.a.me.name : "?"
            st: S.ui.st || "On"
            ring: Th.bg3
            sv: r.a ? S.tag(r.a.svc) : ""
            anchors.verticalCenter: parent.verticalCenter
            MouseArea { anchors.fill: parent; cursorShape: Qt.PointingHandCursor; onClicked: sm.popup() }
        }
        Column {
            anchors.verticalCenter: parent.verticalCenter
            width: parent.width - 44
            Lbl { width: parent.width; text: r.a ? (r.a.me.disp || r.a.me.name) : "No account"; font.weight: Font.DemiBold; font.pixelSize: Th.fsS }
            Lbl {
                width: parent.width
                text: r.al.length > 1 ? r.al.length + " accounts" : (r.a ? r.a.me.name : "")
                color: Th.tx3
                font.pixelSize: 11
            }
        }
    }
    Row {
        id: btns
        anchors.right: parent.right
        anchors.rightMargin: 6
        anchors.verticalCenter: parent.verticalCenter
        IBtn { n: r.mu ? "micoff" : "mic"; red: r.mu; tip: r.mu ? "Unmute" : "Mute"; onClicked: S.c.vset(!r.mu, r.de) }
        IBtn { n: r.de ? "headoff" : "head"; red: r.de; tip: r.de ? "Undeafen" : "Deafen"; onClicked: S.c.vset(r.mu, !r.de) }
        IBtn { n: "cog"; tip: "Settings  (Ctrl+,)"; onClicked: S.showSet = true }
    }
    Menu {
        id: sm
        Repeater {
            model: [["On", "Online"], ["Idle", "Idle"], ["Dnd", "Do not disturb"], ["Off", "Invisible"]]
            MenuItem {
                text: modelData[1]
                onTriggered: { S.setUi("st", modelData[0]); S.c.set_st(modelData[0].toLowerCase() === "on" ? "on" : modelData[0].toLowerCase(), "") }
            }
        }
        MenuSeparator {}
        Repeater {
            model: r.al
            MenuItem {
                text: (modelData.me.disp || modelData.me.name) + "  ·  " + Th.svcName(modelData.svc)
                checkable: true
                checked: modelData.id === r.cur
                onTriggered: S.setUi("meAcc", modelData.id)
            }
        }
        MenuItem { text: "Add account"; onTriggered: { S.setPage = "accounts"; S.showSet = true } }
    }
}
