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
    property var rows: null
    function gName(id) {
        if (id === "online") return "Online"
        if (id === "offline") return "Offline"
        const gd = S.gmap[r.g]
        const ro = gd && gd.roles ? gd.roles.find(x => S.raw(S.ridOf(x.id)) === id) : null
        return ro ? ro.name : id
    }

    function key(u) { return u && u.id ? S.ridOf(u.id) : "" }
    function pr(k, u) {
        if (S.isMe(u, S.acc(ch ? ch.rid : ""))) return { st: S.ui.st || "On" }
        return S.pres[k] || null
    }

    function reset() {
        byId = ({})
        grp = []
        total = 0
        rows = null
        if (g) {
            S.c.load_mems(g)
            return
        }
        const o = {}
        if (ch && ch.rcp) for (const u of ch.rcp) { const k = key(u); if (k) o[k] = { usr: u, nick: null, roles: [] } }
        const mu = ch ? S.me(S.acc(ch.rid)) : null
        if (mu && ch.k === "Grp") { const k = key(mu); if (k) o[k] = { usr: mu, nick: null, roles: [] } }
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
        function onMemList(gld, rs) {
            if (gld !== r.g) return
            r.rows = rs
            regroup.restart()
        }
        function onPresChanged() { regroup.restart() }
        function onUiChanged() { regroup.restart() }
    }

    Timer {
        id: regroup
        interval: 200
        onTriggered: {
            if (r.rows) {
                const out = []
                let n = 0
                for (const x of r.rows) {
                    if (x.G) { if (x.G.n > 0) out.push({ h: r.gName(x.G.id) + "  ·  " + x.G.n }); continue }
                    const k = S.ridOf(x.M)
                    const m = r.byId[k]
                    if (m) { out.push({ m: m, uid: k }); n++ }
                }
                r.total = n
                r.grp = out
                return
            }
            const on = [], off = []
            for (const k in r.byId) {
                const p = r.pr(k, r.byId[k].usr)
                const e = { m: r.byId[k], uid: k }
                if (p && p.st !== "Off") on.push(e); else off.push(e)
            }
            const nm = e => (e.m.nick || e.m.usr.disp || e.m.usr.name || "").toLowerCase()
            const cmp = (a, b) => nm(a) < nm(b) ? -1 : nm(a) > nm(b) ? 1 : 0
            on.sort(cmp)
            off.sort(cmp)
            const out = []
            if (on.length) { out.push({ h: "Online  ·  " + on.length }); out.push(...on) }
            if (off.length) { out.push({ h: "Offline  ·  " + off.length }); out.push(...off) }
            r.total = on.length + off.length
            r.grp = out
        }
    }

    readonly property var dmU: ch && ch.k === "Dm" && ch.rcp && ch.rcp.length ? ch.rcp[0] : null
    Item {
        id: dmP
        visible: !!r.dmU
        anchors.fill: parent
        readonly property string uid: r.dmU ? r.key(r.dmU) : ""
        readonly property var p: S.presOf(uid)
        readonly property string since: {
            if (!r.dmU || !r.dmU.id || S.svc(uid) !== "dc") return ""
            const ms = Math.floor(Number(r.dmU.id.raw) / 4194304) + 1420070400000
            return new Date(ms).toLocaleDateString(Qt.locale(), "d MMM yyyy")
        }
        Rectangle { width: parent.width; height: 96; color: r.dmU && r.dmU.col ? "#" + ("000000" + r.dmU.col.toString(16)).slice(-6) : Th.bg4 }
        Av {
            id: dav
            x: 16
            y: 52
            sz: 80
            src: r.dmU ? (r.dmU.av || "") : ""
            name: r.dmU ? r.dmU.name : ""
            st: dmP.p ? dmP.p.st : "Off"
            ring: Th.bg2
        }
        Column {
            x: 16
            y: dav.y + dav.sz + 12
            width: parent.width - 32
            spacing: 4
            Lbl { width: parent.width; text: r.dmU ? (r.dmU.disp || r.dmU.name) : ""; font.pixelSize: Th.fsL; font.weight: Font.Bold }
            Lbl { width: parent.width; text: r.dmU ? r.dmU.name : ""; color: Th.tx2 }
            Item { width: 1; height: 8 }
            Rectangle {
                width: parent.width
                height: ic.implicitHeight + 24
                radius: Th.rs
                color: Th.bg3
                Column {
                    id: ic
                    x: 12
                    y: 12
                    width: parent.width - 24
                    spacing: 10
                    Column {
                        visible: !!(dmP.p && dmP.p.act)
                        width: parent.width
                        spacing: 2
                        Lbl { text: "Activity"; color: Th.tx3; font.pixelSize: 11; font.weight: Font.DemiBold }
                        Lbl { width: parent.width; text: dmP.p && dmP.p.act ? dmP.p.act : ""; color: Th.tx; wrapMode: Text.Wrap; elide: Text.ElideNone }
                    }
                    Column {
                        visible: dmP.since !== ""
                        width: parent.width
                        spacing: 2
                        Lbl { text: "Member since"; color: Th.tx3; font.pixelSize: 11; font.weight: Font.DemiBold }
                        Lbl { text: dmP.since; color: Th.tx }
                    }
                }
            }
            Item { width: 1; height: 4 }
            Btn { width: parent.width; height: 34; text: "Copy user ID"; onClicked: S.copy(r.dmU && r.dmU.id ? r.dmU.id.raw : "") }
        }
    }

    ListView {
        id: lv
        visible: !r.dmU
        anchors.fill: parent
        anchors.topMargin: 8
        clip: true
        model: r.grp
        ScrollBar.vertical: Scr {}
        delegate: Item {
            width: lv.width
            height: ld.height
            Loader {
                id: ld
                property var it: modelData
                sourceComponent: it.h ? hd : row
            }
        }
        Lbl { anchors.centerIn: parent; visible: r.total === 0; text: "Loading members"; color: Th.tx3 }
    }
    Component {
        id: hd
        Item {
            width: lv.width
            height: 32
            Lbl { x: 16; y: 12; text: it.h; color: Th.tx3; font.pixelSize: 12; font.weight: Font.DemiBold }
        }
    }
    Component {
        id: row
        Rectangle {
            id: rw
            property var p: r.pr(it.uid, it.m.usr)
            property bool onl: !!p && p.st !== "Off"
            height: 42 * Th.dens
            x: 8
            width: lv.width - 20
            radius: Th.rs
            color: ma.containsMouse ? Th.hov : "transparent"
            Av {
                id: av
                x: 8
                anchors.verticalCenter: parent.verticalCenter
                sz: 32
                src: it.m.usr.av || ""
                name: it.m.usr.name
                st: rw.p ? rw.p.st : "Off"
                gray: !rw.onl
            }
            Column {
                anchors.left: av.right
                anchors.leftMargin: 10
                anchors.right: pl.left
                anchors.verticalCenter: parent.verticalCenter
                Lbl {
                    width: parent.width
                    text: it.m.nick || it.m.usr.disp || it.m.usr.name
                    color: !rw.onl ? Th.tx3 : it.m.usr.col ? Th.lift("#" + ("000000" + it.m.usr.col.toString(16)).slice(-6), Th.bg2, 4.5) : Th.tx
                    font.weight: Font.Medium
                }
                Lbl { visible: !!(rw.p && rw.p.act); width: parent.width; text: rw.p && rw.p.act ? rw.p.act : ""; color: Th.tx3; font.pixelSize: 11 }
            }
            Row {
                id: pl
                visible: S.f("platform") && !!rw.p
                anchors.right: parent.right
                anchors.rightMargin: 8
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
