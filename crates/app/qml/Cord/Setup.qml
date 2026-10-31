import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import Cord
import "Lay.js" as L

Rectangle {
    id: r
    color: Th.bg
    property int step: 0
    property string pre: "classic"
    Column {
        anchors.centerIn: parent
        spacing: 28
        width: Math.min(parent.width - 64, 960)
        Column {
            spacing: 6
            Lbl { text: r.step === 0 ? "Choose a layout" : "Connect your accounts"; font.pixelSize: Th.fsXL + 8; font.weight: Font.Bold }
            Lbl {
                text: r.step === 0 ? "Every panel is a block. You can change sides, hide blocks or rearrange everything later in Settings, Layout." : "Discord, Matrix and Stoat share one server bar and one friends list. Add as many accounts as you like."
                color: Th.tx2
                wrapMode: Text.Wrap
                width: parent.width
            }
        }
        Grid {
            visible: r.step === 0
            columns: 3
            spacing: 16
            Repeater {
                model: Object.keys(L.presets)
                Rectangle {
                    width: 296
                    height: 210
                    radius: Th.r
                    color: Th.bg2
                    border.color: r.pre === modelData ? Th.ac : pm.containsMouse ? Th.tx3 : Th.ln
                    border.width: r.pre === modelData ? 2 : 1
                    LayPrev { x: 14; y: 14; width: parent.width - 28; height: 150; n: L.presets[modelData] }
                    Lbl { x: 16; y: 176; text: L.presets[modelData].name; font.weight: Font.DemiBold }
                    MouseArea { id: pm; anchors.fill: parent; hoverEnabled: true; cursorShape: Qt.PointingHandCursor; onClicked: r.pre = modelData; onDoubleClicked: { r.pre = modelData; r.step = 1 } }
                }
            }
        }
        RowLayout {
            visible: r.step === 1
            spacing: 40
            width: parent.width
            AccAdd { Layout.alignment: Qt.AlignTop }
            Column {
                Layout.alignment: Qt.AlignTop
                Layout.fillWidth: true
                spacing: 8
                Lbl { text: "Connected"; color: Th.tx3; font.weight: Font.DemiBold }
                Repeater {
                    model: S.cfgAccs
                    Row {
                        spacing: 8
                        Badge { sv: modelData.svc; txt: true; anchors.verticalCenter: parent.verticalCenter }
                        Lbl {
                            property var a: S.accs[modelData.id]
                            text: a ? (a.me.disp || a.me.name) : modelData.base === "demo" ? "Demo account" : "Connecting"
                            anchors.verticalCenter: parent.verticalCenter
                        }
                    }
                }
                Lbl { visible: S.cfgAccs.length === 0; text: "No accounts yet"; color: Th.tx3 }
                Item { width: 1; height: 16 }
                Btn {
                    text: "Try with demo data"
                    flat: true
                    onClicked: { S.c.add("dc", '{"demo":true}'); S.c.add("mx", '{"demo":true}'); S.c.add("st", '{"demo":true}') }
                }
            }
        }
        Row {
            spacing: 8
            Btn { visible: r.step === 1; text: "Back"; flat: true; onClicked: r.step = 0 }
            Btn {
                text: r.step === 0 ? "Continue" : "Finish"
                pri: true
                onClicked: {
                    if (r.step === 0) { S.preset(r.pre); r.step = 1; return }
                    S.setUi("done", true)
                    S.setup = false
                    S.refresh()
                }
            }
        }
    }
}
