pragma Singleton
import QtQuick

QtObject {
    id: t
    property string name: "Graphite"
    property bool dark: true
    property color bg: "#0b0c0e"
    property color bg2: "#131518"
    property color bg3: "#262a30"
    property color bg4: "#30343b"
    readonly property color bgc: dark ? Qt.lighter(bg2, 1.55) : Qt.lighter(bg2, 1.05)
    property color hov: "#0dffffff"
    property color sel: "#17ffffff"
    property color ln: "#24272c"
    property color tx: "#e7e9ec"
    property color tx2: "#a3a8b0"
    property color tx3: "#8a909a"
    property color ac: "#8fb3ff"
    property color acTx: "#0f1012"
    property color ok: "#4fbf7f"
    property color warn: "#a99bff"
    property color err: "#e5605a"
    property color ment: "#268fb3ff"
    property color dc: "#5865f2"
    property color mx: "#2fa585"
    property color st: "#c0689f"
    property real r: 10
    property real rs: 6
    property string font: "Inter"
    property string mono: "JetBrains Mono"
    readonly property string mod: Qt.platform.os === "osx" ? "⌘" : "Ctrl+"
    property real fs: 14
    property real dens: 1.0
    property real ui: 1.0
    readonly property color mr: Qt.darker(err, 1.35)
    readonly property real sp: 8 * dens
    readonly property real fsS: fs - 2
    readonly property real fsL: fs + 2
    readonly property real fsXL: fs + 6

    function svcCol(s) { return s === "Mx" || s === "mx" ? mx : s === "St" || s === "st" ? st : dc }
    function svcName(s) { return s === "Mx" || s === "mx" ? "Matrix" : s === "St" || s === "st" ? "Stoat" : "Discord" }
    function stCol(s) { return s === "On" ? ok : s === "Idle" ? warn : s === "Dnd" ? err : tx3 }
    function lum(c) {
        const q = Qt.color(c)
        const f = v => v <= 0.03928 ? v / 12.92 : Math.pow((v + 0.055) / 1.055, 2.4)
        return 0.2126 * f(q.r) + 0.7152 * f(q.g) + 0.0722 * f(q.b)
    }
    function lift(c, b, m) {
        let q = Qt.color(c)
        const bl = lum(b)
        const tg = Qt.color(tx)
        for (let i = 0; i < 12; i++) {
            const l = lum(q)
            if ((Math.max(l, bl) + 0.05) / (Math.min(l, bl) + 0.05) >= m) break
            q = Qt.rgba(q.r + (tg.r - q.r) * 0.12, q.g + (tg.g - q.g) * 0.12, q.b + (tg.b - q.b) * 0.12, 1)
        }
        return q
    }
    function a(c, x) {
        const q = Qt.color(c)
        return Qt.rgba(q.r, q.g, q.b, x)
    }
    function cv(v) {
        return typeof v === "string" && /^#[0-9a-fA-F]{8}$/.test(v) ? "#" + v.slice(7) + v.slice(1, 7) : v
    }
    function apply(o) {
        if (!o) return
        for (const k in o) {
            if (k in t && typeof t[k] !== "function") t[k] = cv(o[k])
        }
        tx3 = lift(tx3, bg, 4.5)
    }
    function fromScheme(j) {
        const c = j && j.colours
        if (!c) return
        const h = v => "#" + v
        apply({
            name: "Caelestia", dark: j.mode !== "light",
            bg: h(c.surfaceContainerLowest || c.background), bg2: h(c.surface), bg3: h(c.surfaceContainer), bg4: h(c.surfaceContainerHighest),
            ln: h(c.outlineVariant), tx: h(c.onSurface), tx2: h(c.onSurfaceVariant), tx3: h(c.outline),
            ac: h(c.primary), acTx: h(c.onPrimary), err: h(c.error), ment: a(h(c.primary), 0.15)
        })
    }
}
