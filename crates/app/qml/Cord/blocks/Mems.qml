import QtQuick
import QtQuick.Controls
import Cord

Rectangle {
    id: r
    property var o: ({})
    implicitWidth: 248
    implicitHeight: 400
    color: Th.bg2
    property var ch: S.chans[S.selC] || null
    property string g: ch && ch.grid ? ch.grid : ""
    property var ms: []
    property bool shown: !!ch && S.selC !== "inbox"
    visible: shown
    function ld() {
        if (!g) {
            ms = ch && ch.rcp ? ch.rcp.map(u => ({ m: { usr: u, nick: null, roles: [] }, p: S.presOf(u.id) || { st: "Off" } })) : []
            return
        }
        ms = JSON.parse(S.c.mems(g))
        if (!ms.length && !asked[g]) {
            asked[g] = true
            S.c.load_mems(g)
        }
    }
    property var asked: ({})
    onGChanged: ld()
    onChChanged: ld()
    Connections {
        target: S
        function onEv(d) { if (d.t === "Mems" && d.d === r.g) r.ld() }
    }
    property var grp: {
        const on = ms.filter(x => x.p && x.p.st !== "Off")
        const off = ms.filter(x => !x.p || x.p.st === "Off")
        const out = []
        if (on.length) { out.push({ h: "ONLINE  ·  " + on.length }); out.push(...on) }
        if (off.length) { out.push({ h: "OFFLINE  ·  " + off.length }); out.push(...off) }
        return out
    }
    ListView {
        id: lv
        anchors.fill: parent
        anchors.topMargin: 8
        clip: true
        model: r.grp
        ScrollBar.vertical: Scr {}
        delegate: Loader {
            width: lv.width
            property var it: modelData
            sourceComponent: it.h ? hd : row
        }
        Lbl { anchors.centerIn: parent; visible: r.ms.length === 0; text: "Loading members"; color: Th.tx3 }
    }
    Component {
        id: hd
        Item {
            height: 36
            Lbl { x: 16; y: 16; text: it.h; color: Th.tx3; font.pixelSize: 11; font.weight: Font.DemiBold }
        }
    }
    Component {
        id: row
        Rectangle {
            height: 42 * Th.dens
            x: 8
            width: lv.width - 22
            radius: Th.rs
            color: ma.containsMouse ? Th.hov : "transparent"
            opacity: it.p && it.p.st !== "Off" ? 1 : 0.5
            Av {
                id: av
                x: 8
                anchors.verticalCenter: parent.verticalCenter
                sz: 32
                src: it.m.usr.av || ""
                name: it.m.usr.name
                st: it.p ? it.p.st : "Off"
            }
            Column {
                anchors.left: av.right
                anchors.leftMargin: 10
                anchors.right: pl.left
                anchors.verticalCenter: parent.verticalCenter
                Lbl {
                    width: parent.width
                    text: it.m.nick || it.m.usr.disp || it.m.usr.name
                    color: it.m.usr.col ? "#" + ("000000" + it.m.usr.col.toString(16)).slice(-6) : Th.tx
                    font.weight: Font.Medium
                }
                Lbl { visible: !!(it.p && it.p.act); width: parent.width; text: it.p && it.p.act ? it.p.act : ""; color: Th.tx3; font.pixelSize: 11 }
            }
            Row {
                id: pl
                visible: S.f("platform") && !!it.p
                anchors.right: parent.right
                anchors.rightMargin: 10
                anchors.verticalCenter: parent.verticalCenter
                spacing: 3
                Ico { visible: !!(it.p && it.p.desk); n: "screen"; sz: 13; col: Th.tx3 }
                Ico { visible: !!(it.p && it.p.mob); n: "phone"; sz: 13; col: Th.tx3 }
                Ico { visible: !!(it.p && it.p.web); n: "globe"; sz: 13; col: Th.tx3 }
            }
            MouseArea { id: ma; anchors.fill: parent; hoverEnabled: true; cursorShape: Qt.PointingHandCursor; onClicked: S.prof(it.m.usr, av) }
        }
    }
}
