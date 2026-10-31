import QtQuick
import QtQuick.Controls
import QtQuick.Window
import Cord
import "Cord/Lay.js" as L

ApplicationWindow {
    id: win
    width: 1440
    height: 900
    minimumWidth: 720
    minimumHeight: 480
    visible: true
    title: S.selC ? S.chName(S.selC) + " - cord" : "cord"
    color: Th.bg
    font.family: Th.font
    font.pixelSize: Th.fs

    Component.onCompleted: {
        S.boot()
        const sh = Qt.application.arguments.indexOf("--shot")
        if (sh >= 0) shotT.out = Qt.application.arguments[sh + 1] || "/tmp/cord.png"
    }

    Timer {
        id: shotT
        property string out: ""
        property string preset: ""
        interval: 2500
        running: out !== ""
        onTriggered: win.contentItem.grabToImage(r => { r.saveToFile(out); Qt.quit() })
    }

    Node {
        id: root
        anchors.fill: parent
        n: S.lay
        p: []
        visible: !S.setup
    }

    Rectangle {
        anchors.top: parent.top
        anchors.horizontalCenter: parent.horizontalCenter
        anchors.topMargin: 8
        visible: S.edit
        z: 50
        width: er.implicitWidth + 16
        height: 40
        radius: Th.r
        color: Th.bg3
        border.color: Th.ln
        Row {
            id: er
            anchors.centerIn: parent
            spacing: 8
            Lbl { text: "Edit layout: drag blocks onto an edge of another block"; color: Th.tx2; anchors.verticalCenter: parent.verticalCenter }
            Btn { text: "Add block"; ico: "plus"; onClicked: addM.popup() }
            Btn { text: "Done"; pri: true; onClicked: S.edit = false }
        }
        Menu {
            id: addM
            Repeater {
                model: S.allBlks()
                MenuItem {
                    text: modelData.name
                    onTriggered: S.setLay(L.ins(S.lay, [], "r", { b: modelData.id, w: 280 }))
                }
            }
        }
    }

    Loader {
        anchors.fill: parent
        active: S.setup
        source: "Cord/Setup.qml"
        z: 80
    }
    Loader {
        anchors.fill: parent
        active: S.showSet
        source: "Cord/Settings.qml"
        z: 70
    }
    Loader {
        id: qs
        anchors.fill: parent
        active: S.qk
        source: "Cord/Quick.qml"
        z: 75
    }

    Column {
        anchors.bottom: parent.bottom
        anchors.right: parent.right
        anchors.margins: 16
        spacing: 8
        z: 90
        Repeater {
            model: S.errs.filter(e => Date.now() - e.at < 8000)
            Rectangle {
                width: Math.min(420, tx.implicitWidth + 24)
                height: tx.implicitHeight + 16
                radius: Th.rs
                color: Th.bg3
                border.color: Th.err
                Lbl {
                    id: tx
                    anchors.centerIn: parent
                    width: Math.min(396, implicitWidth)
                    text: modelData.msg
                    wrapMode: Text.Wrap
                    color: Th.tx
                }
            }
        }
    }
    Timer {
        interval: 2000
        repeat: true
        running: S.errs.length > 0
        onTriggered: S.errs = S.errs.filter(e => Date.now() - e.at < 8000)
    }

    Instantiator {
        model: S.mods.filter(m => m.on)
        delegate: Loader {
            source: modelData.url
            onStatusChanged: if (status === Loader.Error) console.warn("mod failed:", modelData.id)
        }
    }

    Shortcut { sequence: "Ctrl+K"; onActivated: S.qk = !S.qk }
    Shortcut { sequence: "Ctrl+,"; onActivated: S.showSet = !S.showSet }
    Shortcut { sequence: "Ctrl+E"; onActivated: S.edit = !S.edit }
    Shortcut { sequence: "Escape"; onActivated: { if (S.qk) S.qk = false; else if (S.showSet) S.showSet = false; else if (S.edit) S.edit = false; else { S.reply = ""; S.editing = "" } } }
    Shortcut { sequence: "Shift+Escape"; onActivated: S.c.ack(S.selC) }
    Shortcut {
        sequence: "Alt+Down"
        onActivated: win.step(1)
    }
    Shortcut {
        sequence: "Alt+Up"
        onActivated: win.step(-1)
    }
    function step(d) {
        const l = (S.sel === "home" ? S.dms() : S.chList(S.sel)).filter(c => c.k !== "Cat" && c.k !== "Voice" && c.k !== "Stage")
        const i = l.findIndex(c => c.rid === S.selC)
        const n = l[(i + d + l.length) % l.length]
        if (n) S.open(n.rid)
    }
}
