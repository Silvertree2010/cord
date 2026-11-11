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
    property int asked: 0
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
        asked = 0
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
    readonly property string rk: !g && ch ? ch.rid + ":" + (ch.rcp || []).map(u => key(u)).join(",") : ""
    onRkChanged: if (!g) reset()
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
                    if (x.G) { if (x.G.n > 0) out.push({ h: r.gName(x.G.id) + " — " + x.G.n }); continue }
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
            if (r.ch && r.ch.k === "Grp") {
                out.push({ h: "Members — " + (on.length + off.length) })
                out.push(...on, ...off)
            } else {
                if (on.length) { out.push({ h: "Online — " + on.length }); out.push(...on) }
                if (off.length) { out.push({ h: "Offline — " + off.length }); out.push(...off) }
            }
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
        onUidChanged: if (uid) dmA.restart()
        transform: Translate { id: dmT }
        ParallelAnimation {
            id: dmA
            NumberAnimation { target: dmP; property: "opacity"; from: 0; to: 1; duration: Th.mM; easing.type: Easing.OutCubic }
            NumberAnimation { target: dmT; property: "x"; from: 14; to: 0; duration: Th.mS; easing.type: Easing.OutCubic }
        }
        Flickable {
            id: dmF
            anchors.fill: parent
            contentHeight: pcard.implicitHeight
            clip: true
            boundsBehavior: Flickable.StopAtBounds
            ScrollBar.vertical: Scr {}
            Smooth {}
            ProfCard {
                id: pcard
                width: dmF.width
                height: Math.max(implicitHeight, dmF.height)
                u: r.dmU
                playing: dmP.visible
            }
        }
        Rectangle {
            anchors.right: parent.right
            anchors.top: parent.top
            anchors.margins: 10
            width: 32
            height: 32
            radius: 16
            color: Th.a("#000000", dm2.containsMouse ? 0.55 : 0.4)
            z: 60
            Ico { anchors.centerIn: parent; n: "dots"; sz: 18; col: "#ffffff" }
            MouseArea { id: dm2; anchors.fill: parent; hoverEnabled: true; cursorShape: Qt.PointingHandCursor; onClicked: pm2.show(parent, 0, parent.height + 4) }
            Ctx {
                id: pm2
                model: [
                    { text: "Copy user ID", run: () => S.copy(r.dmU && r.dmU.id ? r.dmU.id.raw : "") },
                    { text: "Copy username", run: () => S.copy(r.dmU ? r.dmU.name : "") }
                ]
            }
        }
    }

    ListView {
        id: lv
        visible: !r.dmU
        anchors.fill: parent
        Smooth {}
        onContentYChanged: {
            if (!r.rows || !r.g || r.rows.length < 100) return
            if (contentY + height < contentHeight - 600) return
            const at = r.rows.length
            if (at <= r.asked) return
            r.asked = at
            S.c.more_mems(r.g, at)
        }
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
        Timer { id: slowT; interval: 900; running: r.total === 0; onTriggered: lm.show = true; onRunningChanged: if (!running) lm.show = false }
        Lbl { id: lm; property bool show: false; anchors.centerIn: parent; visible: r.total === 0 && show; text: "Loading members"; color: Th.tx3 }
    }
    Component {
        id: hd
        Item {
            width: lv.width
            height: 32
            Lbl { x: 16; y: 14; text: it.h.toUpperCase(); color: Th.tx3; font.pixelSize: Th.fsS - 1; font.weight: Font.Bold; font.letterSpacing: 0.4 }
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
            Loader {
                anchors.fill: parent
                active: !!it.m.usr.plate && S.ui.plates !== false
                sourceComponent: Plate { src: it.m.usr.plate; radius: rw.radius; anim: ma.containsMouse; opacity: rw.onl ? 1 : 0.5 }
            }
            Av {
                id: av
                x: 8
                anchors.verticalCenter: parent.verticalCenter
                sz: 32
                src: it.m.usr.av || ""
                name: it.m.usr.name
                st: rw.p ? rw.p.st : "Off"
                gray: !rw.onl
                ring: ma.containsMouse ? Qt.tint(Th.bg, Th.hov) : Th.bg
                deco: it.m.usr.deco || ""
                anim: ma.containsMouse
            }
            Column {
                anchors.left: av.right
                anchors.leftMargin: 10
                anchors.right: pl.left
                anchors.verticalCenter: parent.verticalCenter
                Row {
                    width: parent.width
                    spacing: 5
                    Name {
                        width: Math.min(implicitWidth, parent.width - (mtp.visible ? mtp.width + 5 : 0))
                        usr: it.m.usr
                        text: it.m.nick || it.m.usr.disp || it.m.usr.name
                        col: !rw.onl ? Th.tx3 : it.m.usr.col ? Th.lift("#" + ("000000" + it.m.usr.col.toString(16)).slice(-6), Th.bg2, 4.5) : Th.tx
                        opacity: rw.onl ? 1 : 0.6
                        weight: Font.Medium
                        anchors.verticalCenter: parent.verticalCenter
                    }
                    RoleIco { role: r.g ? S.roleIcon(r.g, it.uid, it.m.roles) : null; sz: 16; anchors.verticalCenter: parent.verticalCenter }
                    TagPill { id: mtp; tag: it.m.usr.tag || null; fs: 10; opacity: rw.onl ? 1 : 0.6; anchors.verticalCenter: parent.verticalCenter }
                }
                Lbl { visible: !!(rw.p && rw.p.act); width: parent.width; text: rw.p && rw.p.act ? rw.p.act : ""; color: Th.tx3; font.pixelSize: Th.fsT }
            }
            Row {
                id: pl
                visible: S.f("platform") && !!rw.p && ma.containsMouse
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
