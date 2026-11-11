import QtQuick
import QtQuick.Controls
import Cord

Rectangle {
    id: r
    property var o: ({})
    implicitWidth: 600
    implicitHeight: 400
    color: Th.bg
    property var cs: S.call || ({})
    property string cid: cs.chan ? S.ridOf(cs.chan) : (S.chans[S.selC] && (S.chans[S.selC].k === "Voice" || S.chans[S.selC].k === "Stage") ? S.selC : "")
    property var ppl: cid ? S.vsIn(cid) : []
    property string foc: ""
    property bool shown: U.inCall
    property var vcs: r.cid ? [] : S.chList(S.sel).filter(c => c.k === "Voice" || c.k === "Stage").slice(0, 5)
    Column {
        anchors.centerIn: parent
        visible: !r.cid
        spacing: 10
        width: Math.min(320, parent.width - 32)
        Ico { anchors.horizontalCenter: parent.horizontalCenter; n: "voice"; sz: 32; col: Th.tx3 }
        Lbl { width: parent.width; horizontalAlignment: Text.AlignHCenter; text: S.t("No active call"); font.weight: Font.DemiBold }
        Lbl { width: parent.width; horizontalAlignment: Text.AlignHCenter; text: S.t("Join a voice channel to see participants and streams here."); color: Th.tx2; wrapMode: Text.Wrap; elide: Text.ElideNone }
        Repeater {
            model: r.vcs
            Btn {
                anchors.horizontalCenter: parent.horizontalCenter
                width: parent.width
                text: S.t("Join ") + modelData.name
                ico: "voice"
                onClicked: S.c.join(modelData.rid)
            }
        }
    }
    Item {
        anchors.fill: parent
        anchors.margins: 16
        anchors.bottomMargin: 72
        visible: !!r.cid
        Loader {
            anchors.fill: parent
            active: r.foc !== ""
            sourceComponent: Rectangle {
                color: "#000"
                radius: Th.r
                Image {
                    anchors.fill: parent
                    fillMode: Image.PreserveAspectFit
                    readonly property bool fx: typeof S.frame === "function"
                    source: fx ? S.frame(r.foc) : "image://cordv/" + r.foc + "?" + fr
                    cache: false
                    property int fr: 0
                    Timer { interval: 16; repeat: true; running: !parent.fx; onTriggered: parent.fr++ }
                }
                IBtn { anchors.right: parent.right; anchors.margins: 8; y: 8; n: "x"; onClicked: { S.c.watch(r.foc, false); r.foc = "" } }
            }
        }
        Grid {
            id: gr
            visible: r.foc === ""
            anchors.centerIn: parent
            property int n: Math.max(1, r.ppl.length)
            columns: Math.ceil(Math.sqrt(n))
            spacing: 8
            property real tw: Math.min((parent.width - (columns - 1) * spacing) / columns, ((parent.height - (Math.ceil(n / columns) - 1) * spacing) / Math.ceil(n / columns)) * 16 / 9)
            Repeater {
                model: r.ppl
                Rectangle {
                    property string uid: modelData.usr.id ? S.ridOf(modelData.usr.id) : ""
                    width: gr.tw
                    height: gr.tw * 9 / 16
                    radius: Th.r
                    color: Th.bg3
                    border.color: S.speaking[uid] ? Th.ok : "transparent"
                    border.width: 2
                    Av { anchors.centerIn: parent; sz: Math.min(96, parent.height * 0.4); src: modelData.usr.av || ""; name: modelData.usr.name }
                    Rectangle {
                        anchors.left: parent.left
                        anchors.bottom: parent.bottom
                        anchors.margins: 8
                        width: nl.implicitWidth + 16
                        height: 24
                        radius: 6
                        color: "#aa000000"
                        Row {
                            id: nl
                            anchors.centerIn: parent
                            spacing: 4
                            Lbl { text: modelData.usr.disp || modelData.usr.name; color: Th.wh; font.pixelSize: Th.fsS }
                            Ico { visible: modelData.mute || modelData.smute; n: "micoff"; sz: 14; col: Th.wh }
                        }
                    }
                    Btn {
                        visible: modelData.stream
                        anchors.centerIn: parent
                        anchors.verticalCenterOffset: parent.height * 0.3
                        text: S.t("Watch stream")
                        pri: true
                        onClicked: { S.c.watch(parent.uid, true); r.foc = parent.uid }
                    }
                    MouseArea {
                        anchors.fill: parent
                        z: -1
                        acceptedButtons: Qt.RightButton
                        onClicked: m => vm.show(parent, m.x, m.y)
                    }
                    Ctx {
                        id: vm
                        model: [{ slider: true, text: S.t("User volume"), from: 0, to: 2, value: 1, run: v => S.c.vol(uid, v) }]
                    }
                }
            }
        }
    }
    Row {
        visible: !!r.cid
        anchors.bottom: parent.bottom
        anchors.horizontalCenter: parent.horizontalCenter
        anchors.bottomMargin: 16
        spacing: 10
        IBtn { sz: 44; n: r.cs.mute ? "micoff" : "mic"; red: !!r.cs.mute; onClicked: S.c.vset(!r.cs.mute, !!r.cs.deaf) }
        IBtn { sz: 44; n: r.cs.deaf ? "headoff" : "head"; red: !!r.cs.deaf; onClicked: S.c.vset(!!r.cs.mute, !r.cs.deaf) }
        IBtn { sz: 44; n: "screen"; act: !!r.cs.live; tip: S.t("Share screen"); onClicked: r.cs.live ? S.c.golive(false, "{}") : S.goLive() }
        IBtn { visible: !r.cs.chan; sz: 44; n: "phone"; tip: S.t("Join"); onClicked: S.c.join(r.cid) }
        IBtn { visible: !!r.cs.chan; sz: 44; n: "hang"; red: true; tip: S.t("Leave"); onClicked: S.c.leave() }
    }
}
