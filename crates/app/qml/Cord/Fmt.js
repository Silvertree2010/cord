.pragma library

function esc(s) {
    return s.replace(/&/g, "&amp;").replace(/</g, "&lt;").replace(/>/g, "&gt;").replace(/"/g, "&quot;")
}

function rel(ms, now) {
    const d = Math.round((ms - now) / 1000), a = Math.abs(d)
    const u = a < 60 ? [a, "second"] : a < 3600 ? [Math.round(a / 60), "minute"] : a < 86400 ? [Math.round(a / 3600), "hour"] : a < 2592000 ? [Math.round(a / 86400), "day"] : a < 31536000 ? [Math.round(a / 2592000), "month"] : [Math.round(a / 31536000), "year"]
    const w = u[0] + " " + u[1] + (u[0] === 1 ? "" : "s")
    return d >= 0 ? "in " + w : w + " ago"
}

function stamp(ms, f, now) {
    const d = new Date(ms)
    const M = ["January", "February", "March", "April", "May", "June", "July", "August", "September", "October", "November", "December"]
    const W = ["Sunday", "Monday", "Tuesday", "Wednesday", "Thursday", "Friday", "Saturday"]
    const p = n => (n < 10 ? "0" : "") + n
    const hm = p(d.getHours()) + ":" + p(d.getMinutes())
    const dl = d.getDate() + " " + M[d.getMonth()] + " " + d.getFullYear()
    switch (f) {
    case "t": return hm
    case "T": return hm + ":" + p(d.getSeconds())
    case "d": return p(d.getDate()) + "/" + p(d.getMonth() + 1) + "/" + d.getFullYear()
    case "D": return dl
    case "F": return W[d.getDay()] + ", " + dl + " " + hm
    case "R": return rel(ms, now || Date.now())
    default: return dl + " " + hm
    }
}

function inl(s, o) {
    const ac = o.ac || "#8fb3ff", cb = o.cb || "rgba(255,255,255,0.08)", mono = o.mono || "monospace", nm = o.names || {}
    const em = o.em || 22
    const keep = []
    const k = h => { keep.push(h); return "\u0000" + (keep.length - 1) + "\u0000" }
    s = s.replace(/`([^`\n]+)`/g, (m, a) => k('<code style="background:' + cb + ';font-family:\'' + mono + '\'">&nbsp;' + a + '&nbsp;</code>'))
    s = s.replace(/&lt;(a?):(\w+):(\d+)&gt;/g, (m, an, n, id) => k('<img src="https://cdn.discordapp.com/emojis/' + id + '.' + (an ? "gif" : "webp") + '?size=48" width="' + em + '" height="' + em + '" alt=":' + n + ':">'))
    s = s.replace(/&lt;@!?(\d+)&gt;/g, (m, id) => k('<a href="cord:usr/' + id + '" style="color:' + ac + ';text-decoration:none;background:' + cb + '">@' + esc(nm[id] || id) + '</a>'))
    s = s.replace(/&lt;#(\d+)&gt;/g, (m, id) => k('<a href="cord:chan/' + id + '" style="color:' + ac + ';text-decoration:none">#' + esc(nm[id] || id) + '</a>'))
    s = s.replace(/&lt;@&amp;(\d+)&gt;/g, (m, id) => k('<span style="color:' + ac + ';text-decoration:none">@' + esc(nm[id] || "role") + '</span>'))
    s = s.replace(/&lt;t:(-?\d+)(?::([tTdDfFR]))?&gt;/g, (m, t, f) => k('<span style="background:' + cb + '">' + stamp(parseInt(t) * 1000, f || "f", o.now) + '</span>'))
    s = s.replace(/\[([^\]]+)\]\((https?:\/\/[^\s)]+)\)/g, (m, t, u) => k('<a href="' + u + '" style="color:' + ac + ';text-decoration:none">' + t + '</a>'))
    s = s.replace(/&lt;(https?:\/\/[^\s<]+?)&gt;/g, (m, u) => k('<a href="' + u + '" style="color:' + ac + ';text-decoration:none">' + u + '</a>'))
    s = s.replace(/(https?:\/\/(?:[^\s<&]|&(?!gt;|lt;))+[^\s<.,:;"')\]&])/g, (m, u) => k('<a href="' + u + '" style="color:' + ac + ';text-decoration:none">' + u + '</a>'))
    s = s.replace(/\*\*\*(.+?)\*\*\*/g, "<b><i>$1</i></b>")
    s = s.replace(/\*\*(.+?)\*\*/g, "<b>$1</b>")
    s = s.replace(/__(.+?)__/g, "<u>$1</u>")
    s = s.replace(/(^|[^\w*])\*([^*\n]+)\*/g, "$1<i>$2</i>")
    s = s.replace(/(^|[^\w])_([^_\n]+)_(?!\w)/g, "$1<i>$2</i>")
    s = s.replace(/~~(.+?)~~/g, "<s>$1</s>")
    s = s.replace(/\|\|(.+?)\|\|/g, (m, a) => o.spoil ? '<span style="background:' + cb + '">' + a + '</span>' : '<a href="cord:spoil" style="background:' + (o.sp || "#555") + ';color:' + (o.sp || "#555") + ';text-decoration:none">' + a + '</a>')
    for (let n = 0; n < 8 && s.indexOf("\u0000") >= 0; n++) s = s.replace(/\u0000(\d+)\u0000/g, (m, i) => keep[parseInt(i)])
    return s
}

function html(src, o) {
    o = o || {}
    if (!src) return ""
    let s = esc(src)
    const blocks = []
    s = s.replace(/```(\w*)\n?([\s\S]*?)```/g, (m, l, c) => {
        blocks.push('<table width="100%" style="background:' + (o.cb || "rgba(255,255,255,0.08)") + '" cellpadding="8"><tr><td><pre style="font-family:\'' + (o.mono || "monospace") + '\'">' + c.replace(/\n$/, "") + '</pre></td></tr></table>')
        return "\u0001" + (blocks.length - 1) + "\u0001"
    })
    const ls = s.split("\n")
    const out = []
    let all = false
    for (let i = 0; i < ls.length; i++) {
        let l = ls[i]
        let m
        if (!all && (m = l.match(/^&gt;&gt;&gt; ?(.*)$/))) { all = true; l = "&gt; " + m[1] }
        else if (all) l = "&gt; " + l
        if ((m = l.match(/^&gt; (.*)$/)) || l === "&gt;") {
            out.push('<table cellpadding="0" cellspacing="0"><tr><td width="3" style="background:' + (o.qb || "#555") + '"></td><td width="8"></td><td>' + inl(m ? m[1] : "", o) + '</td></tr></table>')
            continue
        }
        if ((m = l.match(/^(#{1,3}) (.*)$/))) {
            const sz = [0, 1.5, 1.3, 1.15][m[1].length]
            out.push('<span style="font-size:' + Math.round((o.fs || 14) * sz) + 'px"><b>' + inl(m[2], o) + '</b></span>')
            continue
        }
        if ((m = l.match(/^-# (.*)$/))) {
            out.push('<span style="font-size:' + Math.round((o.fs || 14) * 0.8) + 'px;color:' + (o.sub || "#888") + '">' + inl(m[1], o) + '</span>')
            continue
        }
        if ((m = l.match(/^\s*(\d{1,3})\. (.*)$/))) {
            out.push("&nbsp;" + m[1] + ".&nbsp;" + inl(m[2], o))
            continue
        }
        if ((m = l.match(/^\s*[-*] (.*)$/))) {
            out.push("&nbsp;•&nbsp;" + inl(m[1], o))
            continue
        }
        out.push(inl(l, o))
    }
    s = out.join("<br>")
    s = s.replace(/\u0001(\d+)\u0001/g, (m, i) => blocks[parseInt(i)])
    if (o.big && /^(\s*(<img[^>]*>|\p{Extended_Pictographic}|️|‍))+\s*$/u.test(s)) s = s.replace(/width="\d+" height="\d+"/g, 'width="48" height="48"')
    return s
}

function onlyEmoji(src) {
    return /^(\s*(<a?:\w+:\d+>|\p{Extended_Pictographic}|️|‍|\p{Emoji_Modifier}))+\s*$/u.test(src || "")
}

function time(ms) {
    const d = new Date(ms), n = new Date()
    const hm = d.toLocaleTimeString(Qt.locale(), "HH:mm")
    if (d.toDateString() === n.toDateString()) return hm
    const y = new Date(n.getTime() - 86400000)
    if (d.toDateString() === y.toDateString()) return "Yesterday " + hm
    return d.toLocaleDateString(Qt.locale(), "dd.MM.yyyy") + " " + hm
}

function day(ms) {
    return new Date(ms).toLocaleDateString(Qt.locale(), "d MMMM yyyy")
}

const EMO_TAG = "<(a?):(\\w+):(\\d+)>"
const EMO_LINK = "\\[(\\w+)\\]\\(https:\\/\\/cdn\\.discordapp\\.com\\/emojis\\/(\\d+)\\.(gif|webp|png)\\?[^)]*\\)"

function emoLink(an, n, id) {
    return "[" + n + "](https://cdn.discordapp.com/emojis/" + id + "." + (an ? "gif" : "webp") + "?size=48&quality=lossless&name=" + n + ")"
}

function emoTag(an, n, id) {
    return "<" + (an ? "a" : "") + ":" + n + ":" + id + ">"
}

function emoEncode(body, own) {
    own = own || {}
    return body.replace(new RegExp(EMO_TAG, "g"), (m, an, n, id) => (own[id] && !an) ? m : emoLink(an, n, id))
}

function emoDecode(body) {
    const whole = body.trim().match(new RegExp("^" + EMO_LINK + "$"))
    if (whole) return emoTag(whole[3] === "gif", whole[1], whole[2])
    return body.replace(new RegExp(EMO_LINK, "g"), (s, n, id, ex) => emoTag(ex === "gif", n, id))
}

function size(b) {
    if (b < 1024) return b + " B"
    if (b < 1048576) return (b / 1024).toFixed(1) + " KB"
    return (b / 1048576).toFixed(1) + " MB"
}
