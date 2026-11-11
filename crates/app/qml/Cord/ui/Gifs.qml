import QtQuick
import QtQuick.Controls
import Cord

Popup {
    id: r
    enter: PopIn {}
    exit: PopOut {}
    transformOrigin: Popup.Bottom
    property string cid: ""
    property var cb: null
    width: 440
    height: 480
    padding: 12
    closePolicy: Popup.CloseOnEscape | Popup.CloseOnPressOutside
    background: Pane {}
    property var gifs: []
    property var cats: []
    property bool busy: false
    property string err: ""
    property int seq: 0
    readonly property int acc: S.acc(r.cid)
    function load(q) {
        const n = ++r.seq
        r.busy = true
        r.err = ""
        S.api(r.acc, "gifs", { q: q }, (ok, v, e) => {
            if (n !== r.seq) return
            r.busy = false
            if (!ok) { r.err = e; r.gifs = []; return }
            if (Array.isArray(v)) { r.gifs = v; r.cats = [] }
            else { r.gifs = v.gifs || []; r.cats = v.categories || [] }
        })
    }
    onAboutToShow: { U.reg(r); gf.text = ""; load("") }
    onOpened: gf.forceActiveFocus()
    onClosed: U.unreg(r)
    Timer { id: deb; interval: 300; onTriggered: r.load(gf.text.trim()) }
    Column {
        anchors.fill: parent
        spacing: 10
        Fld {
            id: gf
            width: parent.width
            placeholderText: "Search Tenor"
            onTextChanged: deb.restart()
        }
        Flow {
            visible: !gf.text && r.cats.length > 0
            width: parent.width
            spacing: 6
            Repeater {
                model: r.cats.slice(0, 12)
                Rectangle {
                    height: 26
                    width: cl.implicitWidth + 18
                    radius: 13
                    color: cm.containsMouse ? Th.bg4 : Th.bg3
                    Lbl { id: cl; anchors.centerIn: parent; text: modelData.name; font.pixelSize: Th.fsXS; font.weight: Font.Medium }
                    MouseArea { id: cm; anchors.fill: parent; hoverEnabled: true; cursorShape: Qt.PointingHandCursor; onClicked: gf.text = modelData.name }
                }
            }
        }
        Item {
            width: parent.width
            height: parent.height - y
            GridView {
                id: gv
                anchors.fill: parent
                clip: true
                cellWidth: width / 2
                cellHeight: 130
                model: r.gifs
                ScrollBar.vertical: Scr {}
                Smooth {}
                delegate: Item {
                    width: gv.cellWidth
                    height: gv.cellHeight
                    Rectangle {
                        anchors.fill: parent
                        anchors.margins: 3
                        radius: Th.rs
                        color: Th.bg3
                        clip: true
                        AnimatedImage {
                            anchors.fill: parent
                            source: modelData.gif_src || ""
                            fillMode: Image.PreserveAspectCrop
                            asynchronous: true
                            playing: gm.containsMouse || Th.motion
                            cache: true
                        }
                        Rectangle {
                            anchors.fill: parent
                            radius: Th.rs
                            color: "transparent"
                            border.color: Th.ac
                            border.width: 2
                            visible: gm.containsMouse
                        }
                        MouseArea {
                            id: gm
                            anchors.fill: parent
                            hoverEnabled: true
                            cursorShape: Qt.PointingHandCursor
                            onClicked: { if (r.cb) r.cb(modelData.url); r.close() }
                        }
                    }
                }
            }
            BusyIndicator { anchors.centerIn: parent; running: r.busy && r.gifs.length === 0; visible: running; width: 32; height: 32 }
            Lbl { anchors.centerIn: parent; width: parent.width - 32; visible: r.err !== ""; text: r.err; color: Th.err; wrapMode: Text.Wrap; elide: Text.ElideNone; horizontalAlignment: Text.AlignHCenter }
            Lbl { anchors.centerIn: parent; visible: !r.busy && r.err === "" && r.gifs.length === 0 && gf.text !== ""; text: "No GIFs found"; color: Th.tx2 }
        }
    }
}
