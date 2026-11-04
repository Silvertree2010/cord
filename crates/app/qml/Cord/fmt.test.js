const fs = require("fs")
const a = require("assert")
const src = fs.readFileSync(__dirname + "/Fmt.js", "utf8").replace(".pragma library", "") + "\nmodule.exports={emoEncode,emoDecode,html,onlyEmoji};"
const m = { exports: {} }
new Function("module", "Qt", src)(m, {})
const F = m.exports
const s1 = "hi <:wave:123456789> and <a:dance:987654321>!"
const e1 = F.emoEncode(s1, {})
a.ok(e1.indexOf("<") < 0 && e1.indexOf("cdn.discordapp.com/emojis/123456789.webp") > 0 && e1.indexOf("987654321.gif") > 0)
a.strictEqual(F.emoDecode(e1), s1)
a.strictEqual(F.emoEncode("<:own:111>", { 111: true }), "<:own:111>")
a.ok(F.emoEncode("<a:own:111>", { 111: true }).indexOf("111.gif") > 0)
a.strictEqual(F.emoDecode(F.emoEncode("<a:x:5>", {})), "<a:x:5>")
a.strictEqual(F.emoDecode("  " + F.emoEncode("<:x:5>", {}) + " "), "<:x:5>")
a.strictEqual(F.emoDecode("plain text"), "plain text")
a.ok(F.onlyEmoji("<:x:5> <a:y:6>"))
a.ok(!F.onlyEmoji("word <:x:5>"))
a.ok(F.html("**b** <:x:5>", {}).indexOf("<b>b</b>") >= 0)
for (const t of ["<:a:1>", "x<a:b:22>y<:c:333>z", "no emoji"]) a.strictEqual(F.emoDecode(F.emoEncode(t, {})), t)
const gh = F.html("[`0c90ef7`](https://github.com/a/b/commit/0c90ef7) chore(models): use x", {})
a.ok(gh.indexOf("\u0000") < 0 && gh.indexOf("0c90ef7") > 0 && gh.indexOf('href="https://github.com/a/b/commit/0c90ef7"') > 0)
a.ok(F.html("<@42> hi", { names: { 42: "Kai" } }).indexOf("@Kai") > 0)
console.log("fmt ok")
