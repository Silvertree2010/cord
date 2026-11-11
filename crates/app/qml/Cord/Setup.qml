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
    readonly property bool hasDemo: S.cfgAccs.some(a => a.base === "demo")
    Rectangle {
        id: card
        anchors.centerIn: parent
        width: Math.min(640, parent.width - 32)
        height: Math.min(parent.height - 32, cc.implicitHeight + 64)
        radius: Th.r
        color: Th.bgc
        border.color: Th.ln
        ColumnLayout {
            id: cc
            x: 32
            y: 32
            width: parent.width - 64
            spacing: 20
            Column {
                Layout.fillWidth: true
                spacing: 6
                Lbl { text: r.step === 0 ? "Choose a layout" : "Connect accounts"; font.pixelSize: Th.fsXL + 2; font.weight: Font.Bold }
                Lbl {
                    text: r.step === 0 ? "Every panel is a block. You can change sides, hide blocks or rearrange everything later in Settings, Layout." : "Discord, Matrix and Stoat share one server bar and one friends list. Add as many accounts as you like."
                    color: Th.tx2
                    wrapMode: Text.Wrap
                    elide: Text.ElideNone
                    width: parent.width
                }
            }
            Grid {
                id: pg
                visible: r.step === 0
                Layout.fillWidth: true
                columns: 3
                spacing: 12
                Repeater {
                    model: Object.keys(L.presets)
                    Rectangle {
                        width: (pg.width - 24) / 3
                        height: 120
                        radius: Th.r
                        color: Th.bg2
                        border.color: r.pre === modelData || activeFocus ? Th.ac : pm.containsMouse ? Th.tx3 : Th.ln
                        border.width: r.pre === modelData || activeFocus ? 2 : 1
                        activeFocusOnTab: true
                        Accessible.role: Accessible.Button
                        Accessible.name: L.presets[modelData].name
                        Keys.onReturnPressed: r.pre = modelData
                        Keys.onSpacePressed: r.pre = modelData
                        LayPrev { x: 10; y: 10; width: parent.width - 20; height: 74; n: L.presets[modelData] }
                        Lbl { x: 12; y: 92; text: L.presets[modelData].name; font.pixelSize: Th.fsXS; font.weight: Font.DemiBold }
                        MouseArea { id: pm; anchors.fill: parent; hoverEnabled: true; cursorShape: Qt.PointingHandCursor; onClicked: r.pre = modelData; onDoubleClicked: { r.pre = modelData; S.preset(r.pre); r.step = 1 } }
                    }
                }
            }
            ColumnLayout {
                visible: r.step === 1
                Layout.fillWidth: true
                spacing: 12
                Column {
                    Layout.fillWidth: true
                    spacing: 6
                    visible: S.cfgAccs.length > 0
                    Lbl { text: "Connected"; color: Th.tx2; font.pixelSize: Th.fsS; font.weight: Font.DemiBold }
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
                            Rectangle {
                                visible: modelData.base === "demo"
                                width: dl.implicitWidth + 12
                                height: 16
                                radius: 4
                                color: Th.bg4
                                anchors.verticalCenter: parent.verticalCenter
                                Lbl { id: dl; anchors.centerIn: parent; text: "Demo"; color: Th.tx2; font.pixelSize: Th.fsP; font.weight: Font.DemiBold }
                            }
                        }
                    }
                }
                AccAdd { Layout.alignment: Qt.AlignTop; onDone: U.flash("Account added") }
                Row {
                    spacing: 8
                    Btn {
                        visible: !r.hasDemo
                        text: "Try with demo data"
                        height: 36
                        onClicked: { S.c.add("dc", '{"demo":true}'); S.c.add("mx", '{"demo":true}'); S.c.add("st", '{"demo":true}') }
                    }
                    Btn {
                        visible: r.hasDemo
                        text: "Exit demo"
                        height: 36
                        onClicked: U.exitDemo()
                    }
                }
            }
            RowLayout {
                Layout.fillWidth: true
                Btn { visible: r.step === 1; text: "Back"; height: 36; onClicked: r.step = 0 }
                Item { Layout.fillWidth: true }
                Btn {
                    text: r.step === 0 ? "Continue" : S.cfgAccs.length === 0 ? "Skip for now" : "Finish"
                    pri: r.step === 0 || S.cfgAccs.length > 0
                    height: 36
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
}
