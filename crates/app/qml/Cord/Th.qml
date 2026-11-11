pragma Singleton
import QtQuick

QtObject {
    id: t
    property string name: "Midnight"
    property bool dark: true
    property color bg: "#0d0f13"
    property color bg2: "#181a20"
    property color bgc: "#181a20"
    property color bg3: "#1d1f26"
    property color bg4: "#272930"
    property color hov: "#0cffffff"
    property color sel: "#16ffffff"
    property color ln: "#292b32"
    property color tx: "#dbdbe6"
    property color tx2: "#6f717b"
    property color tx3: "#6f717b"
    property color ac: "#a2b0e0"
    property color acTx: "#333e5e"
    property color ok: "#3fb984"
    property color warn: "#f0b232"
    property color err: "#ec5e73"
    property color ment: "#1aa2b0e0"
    property color dc: "#5865f2"
    property color mx: "#2fa585"
    property color st: "#c0689f"
    property real r: 12
    property real gap: 12
    property bool motion: true
    readonly property int mF: motion ? 120 : 0
    readonly property int mM: motion ? 200 : 0
    readonly property int mS: motion ? 260 : 0
    property color lnk: "#b3bde4"
    property real rs: 8
    property real rc: 4
    readonly property color wh: "#ffffff"
    readonly property color scrim: "#99000000"
    readonly property color shade: "#99000000"
    property string font: "Figtree"
    property string mono: "JetBrains Mono"
    readonly property string mod: Qt.platform.os === "osx" ? "⌘" : "Ctrl+"
    property real fs: 15
    property real dens: 1.0
    property real ui: 1.0
    readonly property color mr: Qt.darker(err, 1.35)
    readonly property real sp: 8 * dens
    readonly property real fsS: fs - 2
    readonly property real fsXS: fs - 3
    readonly property real fsT: fs - 4
    readonly property real fsP: fs - 5
    readonly property real fsL: fs + 2
    readonly property real fsXL: fs + 6

    function svcCol(s) { return s === "Mx" || s === "mx" ? mx : s === "St" || s === "st" ? st : dc }
    function svcName(s) { return s === "Mx" || s === "mx" ? "Matrix" : s === "St" || s === "st" ? S.t("Stoat") : S.t("Discord") }
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
