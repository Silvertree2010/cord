pragma Singleton
import QtQuick

QtObject {
    id: t
    property string name: "Graphite"
    property bool dark: true
    property color bg: "#0f1012"
    property color bg2: "#15171a"
    property color bg3: "#1c1f23"
    property color bg4: "#252930"
    property color hov: "#0dffffff"
    property color sel: "#17ffffff"
    property color ln: "#24272c"
    property color tx: "#e7e9ec"
    property color tx2: "#a3a8b0"
    property color tx3: "#6c727b"
    property color ac: "#8fb3ff"
    property color acTx: "#0f1012"
    property color ok: "#4fbf7f"
    property color warn: "#d9a441"
    property color err: "#e5605a"
    property color ment: "#268fb3ff"
    property color dc: "#5865f2"
    property color mx: "#0dbd8b"
    property color st: "#ff6b4a"
    property real r: 10
    property real rs: 6
    property string font: "Inter"
    property string mono: "JetBrains Mono"
    property real fs: 14
    property real dens: 1.0
    property real ui: 1.0
    readonly property real sp: 8 * dens
    readonly property real fsS: fs - 2
    readonly property real fsL: fs + 2
    readonly property real fsXL: fs + 6

    function svcCol(s) { return s === "Mx" || s === "mx" ? mx : s === "St" || s === "st" ? st : dc }
    function svcName(s) { return s === "Mx" || s === "mx" ? "Matrix" : s === "St" || s === "st" ? "Stoat" : "Discord" }
    function stCol(s) { return s === "On" ? ok : s === "Idle" ? warn : s === "Dnd" ? err : tx3 }
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
