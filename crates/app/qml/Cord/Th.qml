pragma Singleton
import QtQuick

QtObject {
    id: t
    property string name: "Graphite"
    property bool dark: true
    property color bg: "#0d0e10"
    property color bg2: "#121316"
    property color bgc: "#17181b"
    property color bg3: "#1e1f23"
    property color bg4: "#27292e"
    property color hov: "#0affffff"
    property color sel: "#12ffffff"
    property color ln: "#202125"
    property color tx: "#ededf0"
    property color tx2: "#a3a4ab"
    property color tx3: "#70727b"
    property color ac: "#8b9dff"
    property color acTx: "#0d0e10"
    property color ok: "#3fb984"
    property color warn: "#9d8cff"
    property color err: "#ef5350"
    property color ment: "#1f8b9dff"
    property color dc: "#5865f2"
    property color mx: "#2fa585"
    property color st: "#c0689f"
    property real r: 12
    property real rs: 8
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
            bg: h(c.surfaceContainerLowest || c.background), bg2: h(c.surfaceContainerLow || c.surface), bgc: h(c.surface), bg3: h(c.surfaceContainer), bg4: h(c.surfaceContainerHighest),
            ln: h(c.outlineVariant), tx: h(c.onSurface), tx2: h(c.onSurfaceVariant), tx3: h(c.outline),
            ac: h(c.primary), acTx: h(c.onPrimary), err: h(c.error), ment: a(h(c.primary), 0.15)
        })
    }
}
