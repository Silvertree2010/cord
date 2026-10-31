.pragma library

function cp(o) { return JSON.parse(JSON.stringify(o)) }

const presets = {
    classic: { name: "Classic", t: "row", ch: [{ b: "rail", w: 72 }, { t: "col", w: 248, ch: [{ b: "side", grow: true }, { b: "call" }, { b: "me" }] }, { b: "chat", grow: true }, { b: "mems", w: 248 }] },
    compact: { name: "Compact", t: "row", ch: [{ b: "rail", w: 56 }, { t: "col", w: 208, ch: [{ b: "side", grow: true }, { b: "call" }, { b: "me" }] }, { b: "chat", grow: true }] },
    minimal: { name: "Minimal", t: "row", ch: [{ t: "col", w: 260, ch: [{ b: "rail", o: { horiz: true }, h: 56 }, { b: "side", grow: true }, { b: "call" }, { b: "me" }] }, { b: "chat", grow: true }] },
    stream: { name: "Streamer", t: "row", ch: [{ b: "rail", w: 72 }, { t: "col", w: 232, ch: [{ b: "side", grow: true }, { b: "call" }, { b: "me" }] }, { b: "stage", grow: true }, { b: "chat", w: 380 }] },
    multi: { name: "Multi", t: "row", ch: [{ b: "rail", w: 64 }, { t: "col", w: 232, ch: [{ b: "side", grow: true }, { b: "call" }, { b: "me" }] }, { b: "chat", grow: true }, { b: "chat", grow: true, o: { slot: 1 } }, { b: "inbox", w: 300 }] },
    right: { name: "Mirrored", t: "row", ch: [{ b: "mems", w: 248 }, { b: "chat", grow: true }, { t: "col", w: 248, ch: [{ b: "side", grow: true }, { b: "call" }, { b: "me" }] }, { b: "rail", w: 72 }] }
}

function at(n, p) {
    for (const i of p) n = n.ch[i]
    return n
}

function find(n, b, p) {
    p = p || []
    if (n.b === b) return p
    for (let i = 0; n.ch && i < n.ch.length; i++) {
        const r = find(n.ch[i], b, p.concat([i]))
        if (r) return r
    }
    return null
}

function has(n, b) { return find(n, b) !== null }

function norm(n) {
    if (!n.ch) return n
    n.ch = n.ch.map(norm).filter(c => c && (c.b || (c.ch && c.ch.length)))
    const out = []
    for (const c of n.ch) {
        if (c.t === n.t && !c.w && !c.h) out.push(...c.ch)
        else out.push(c)
    }
    n.ch = out
    if (n.ch.length === 1 && n.ch[0].t && n.t !== "tabs") {
        const c = n.ch[0]
        if (n.w) c.w = n.w
        if (n.h) c.h = n.h
        if (n.grow) c.grow = n.grow
        return c
    }
    return n
}

function rm(root, p) {
    root = cp(root)
    const par = at(root, p.slice(0, -1))
    const [x] = par.ch.splice(p[p.length - 1], 1)
    return { root: norm(root), node: x }
}

function ins(root, p, side, node) {
    root = cp(root)
    const tgt = at(root, p)
    const ax = side === "l" || side === "r" ? "row" : side === "t" || side === "b" ? "col" : "tabs"
    const pre = side === "l" || side === "t"
    if (p.length) {
        const par = at(root, p.slice(0, -1))
        const i = p[p.length - 1]
        if (par.t === ax) {
            par.ch.splice(pre ? i : i + 1, 0, node)
            return norm(root)
        }
        const w = { t: ax, ch: pre ? [node, tgt] : [tgt, node] }
        if (tgt.w) { w.w = tgt.w; delete tgt.w }
        if (tgt.h) { w.h = tgt.h; delete tgt.h }
        if (tgt.grow) { w.grow = true }
        tgt.grow = true
        par.ch[i] = w
        return norm(root)
    }
    return norm({ t: ax, ch: pre ? [node, root] : [root, node] })
}

function move(root, from, to, side) {
    if (JSON.stringify(from) === JSON.stringify(to)) return root
    const tp = at(root, to)
    const r = rm(root, from)
    const np = path(r.root, tp)
    if (!np) return root
    return ins(r.root, np, side, r.node)
}

function path(n, ref, p) {
    p = p || []
    if (n === ref) return p
    for (let i = 0; n.ch && i < n.ch.length; i++) {
        const r = path(n.ch[i], ref, p.concat([i]))
        if (r) return r
    }
    return null
}

function toggle(root, b, on, def) {
    const p = find(root, b)
    if (!on && p) return rm(root, p).root
    if (on && !p) {
        const cp_ = find(root, "chat")
        return ins(root, cp_ || [], def || "r", { b: b, w: 248 })
    }
    return root
}

function side(root, b, s) {
    const p = find(root, b)
    const c = find(root, "chat")
    if (!p || !c) return root
    return move(root, p, c, s)
}

function sideOf(root, b) {
    const flat = []
    const walk = n => { if (n.b) flat.push(n.b); (n.ch || []).forEach(walk) }
    walk(root)
    const i = flat.indexOf(b), j = flat.indexOf("chat")
    if (i < 0) return "off"
    return i < j ? "l" : "r"
}
