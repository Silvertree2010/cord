const fs = require("fs")
const src = fs.readFileSync(__dirname + "/Lay.js", "utf8").replace(".pragma library", "") + "\nmodule.exports={move,side,find,presets,valid,safe,sideOf};"
const m = { exports: {} }
new Function("module", src)(m)
const L = m.exports
const a = require("assert")
const r = L.presets.classic
const pc = L.find(r, "chat"), pm = L.find(r, "mems")
let o = L.move(r, pm, pc, "l")
a.strictEqual(L.sideOf(o, "mems"), "l")
o = L.side(r, "rail", "r")
a.strictEqual(L.sideOf(o, "rail"), "r")
o = L.side(r, "side", "r")
a.strictEqual(L.sideOf(o, "side"), "r")
a.ok(JSON.stringify(o).indexOf("_mk") < 0)
a.deepStrictEqual(L.move(r, pc, pc, "l"), r)
a.ok(L.valid(r, ["rail", "side", "chat", "mems", "me", "call", "bar"]))
a.ok(!L.valid({ t: "row", ch: [{ b: "zzz" }] }, ["chat"]))
a.ok(!L.valid({ t: "x", ch: [] }))
a.strictEqual(L.safe({ bad: 1 }).name, "Classic")
console.log("lay ok")
