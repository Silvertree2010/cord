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
    property string dm: !g && ch ? S.selC : ""
    property bool shown: !!ch && S.selC !== "inbox"
    visible: shown
    property var byId: ({})
    property var grp: []
    property int total: 0

    function key(u) { return u && u.id ? S.ridOf(u.id) : "" }

    function reset() {
        byId = ({})
        grp = []
        total = 0
        if (g) {
            S.c.load_mems(g)
            return
        }
        const o = {}
        if (ch && ch.rcp) for (const u of ch.rcp) { const k = key(u); if (k) o[k] = { usr: u, nick: null, roles: [] } }
        byId = o
        regroup.restart()
    }
    onGChanged: reset()
    onDmChanged: if (!g) reset()
    Component.onCompleted: reset()

    Connections {
        target: S
        function onMems(gld, list) {
            if (gld !== r.g) return
            const o = r.byId
            for (const m of list) { const k = r.key(m.usr); if (k) o[k] = m }
            r.byId = o
            regroup.restart()
        }
        function onPresChanged() { regroup.restart() }
    }

    Timer {
        id: regroup
        interval: 200
        onTriggered: {
            const on = [], off = []
            for (const k in r.byId) {
                const p = S.pres[k]
                const e = { m: r.byId[k], uid: k }
                if (p && p.st !== "Off") on.push(e); else off.push(e)
            }
            const nm = e => (e.m.nick || e.m.usr.disp || e.m.usr.name || "").toLowerCase()
            const cmp = (a, b) => nm(a) < nm(b) ? -1 : nm(a) > nm(b) ? 1 : 0
            on.sort(cmp)
            off.sort(cmp)
            const out = []
            if (on.length) { out.push({ h: "ONLINE  ·  " + on.length }); out.push(...on) }
            if (off.length) { out.push({ h: "OFFLINE  ·  " + off.length }); out.push(...off) }
            r.total = on.length + off.length
            r.grp = out
        }
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
        Lbl { anchors.centerIn: parent; visible: r.total === 0; text: "Loading members"; color: Th.tx3 }
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
            id: rw
            property var p: S.pres[it.uid] || null
            property bool onl: !!p && p.st !== "Off"
            height: 42 * Th.dens
            x: 8
            width: lv.width - 28
            radius: Th.rs
            color: ma.containsMouse ? Th.hov : "transparent"
            opacity: onl ? 1 : 0.45
            Av {
                id: av
                x: 8
                anchors.verticalCenter: parent.verticalCenter
                sz: 32
                src: it.m.usr.av || ""
                name: it.m.usr.name
                st: rw.p ? rw.p.st : "Off"
            }
            Column {
                anchors.left: av.right
                anchors.leftMargin: 10
                anchors.right: pl.left
                anchors.verticalCenter: parent.verticalCenter
                Lbl {
                    width: parent.width
                    text: it.m.nick || it.m.usr.disp || it.m.usr.name
                    color: rw.onl && it.m.usr.col ? "#" + ("000000" + it.m.usr.col.toString(16)).slice(-6) : (rw.onl ? Th.tx : Th.tx2)
                    font.weight: Font.Medium
                }
                Lbl { visible: !!(rw.p && rw.p.act); width: parent.width; text: rw.p && rw.p.act ? rw.p.act : ""; color: Th.tx3; font.pixelSize: 11 }
            }
            Row {
                id: pl
                visible: S.f("platform") && !!rw.p
                anchors.right: parent.right
                anchors.rightMargin: 10
                anchors.verticalCenter: parent.verticalCenter
                spacing: 3
                Ico { visible: !!(rw.p && rw.p.desk); n: "screen"; sz: 13; col: Th.tx3 }
                Ico { visible: !!(rw.p && rw.p.mob); n: "phone"; sz: 13; col: Th.tx3 }
                Ico { visible: !!(rw.p && rw.p.web); n: "globe"; sz: 13; col: Th.tx3 }
            }
            MouseArea { id: ma; anchors.fill: parent; hoverEnabled: true; cursorShape: Qt.PointingHandCursor; onClicked: S.prof(it.m.usr, av) }
        }
    }
}
