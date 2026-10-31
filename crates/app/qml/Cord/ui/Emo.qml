import QtQuick
import QtQuick.Controls
import Cord

Popup {
    id: r
    property var cb: null
    width: 380
    height: 420
    padding: 10
    closePolicy: Popup.CloseOnEscape | Popup.CloseOnPressOutside
    onAboutToShow: U.reg(r)
    onClosed: U.unreg(r)
    background: Rectangle { color: Th.bg3; radius: Th.r; border.color: Th.ln }
    readonly property var uni: "😀 😃 😄 😁 😆 😅 😂 🤣 🥲 😊 😇 🙂 🙃 😉 😌 😍 🥰 😘 😗 😋 😛 😜 🤪 😝 🤑 🤗 🤭 🤫 🤔 🤐 🤨 😐 😑 😶 😏 😒 🙄 😬 😮‍💨 🤥 😔 😪 🤤 😴 😷 🤒 🤕 🤢 🤮 🥵 🥶 🥴 😵 🤯 🤠 🥳 😎 🤓 🧐 😕 😟 🙁 😮 😯 😲 😳 🥺 😦 😧 😨 😰 😥 😢 😭 😱 😖 😣 😞 😓 😩 😫 🥱 😤 😡 😠 🤬 😈 👿 💀 ☠️ 💩 🤡 👹 👻 👽 🤖 😺 👋 🤚 ✋ 🖖 👌 🤌 🤏 ✌️ 🤞 🤟 🤘 🤙 👈 👉 👆 👇 ☝️ 👍 👎 ✊ 👊 🤛 🤜 👏 🙌 👐 🤲 🤝 🙏 💪 🧠 👀 👁️ 👅 👄 ❤️ 🧡 💛 💚 💙 💜 🖤 🤍 🤎 💔 ❣️ 💕 💞 💓 💗 💖 💘 💝 💯 💢 💥 💫 💦 💨 🕳️ 💬 💤 🔥 ✨ ⭐ 🌟 ⚡ 🎉 🎊 🎈 🎁 🏆 🥇 ⚽ 🎮 🕹️ 🎲 🎧 🎵 🎶 💻 🖥️ ⌨️ 🖱️ 📱 📷 🔒 🔑 🛠️ ⚙️ 🧪 📌 📎 ✅ ❌ ⚠️ ❓ ❗ 🔴 🟢 🔵 ⚫ ⚪ 🍕 🍔 🍟 🌭 🍿 🍩 🍪 🎂 🍰 ☕ 🍺 🍻 🥂 🐶 🐱 🐭 🐹 🐰 🦊 🐻 🐼 🐨 🐯 🦁 🐮 🐷 🐸 🐵 🐔 🐧 🐦 🦆 🦅 🦉 🐺 🐗 🐴 🦄 🐝 🦋 🐌 🐞 🐢 🐍 🐙 🦀 🐬 🐳 🦈 🐊 🦖".split(" ")
    property string q: ""
    property var cust: {
        const out = []
        for (const g of S.glds) for (const e of g.emos || []) out.push({ e: e, g: g.rid })
        return out
    }
    Column {
        anchors.fill: parent
        spacing: 8
        Fld { id: ef; width: parent.width; placeholderText: "Search emoji"; onTextChanged: r.q = text.toLowerCase() }
        GridView {
            width: parent.width
            height: parent.height - ef.height - 8
            clip: true
            cellWidth: 40
            cellHeight: 40
            model: r.cust.filter(x => !r.q || x.e.name.toLowerCase().indexOf(r.q) >= 0).map(x => ({ c: x.e, g: x.g })).concat(r.q ? [] : r.uni.map(u => ({ u: u })))
            ScrollBar.vertical: Scr {}
            delegate: Rectangle {
                width: 38
                height: 38
                radius: Th.rs
                color: em.containsMouse ? Th.hov : "transparent"
                Image { visible: !!modelData.c; anchors.centerIn: parent; width: 28; height: 28; source: modelData.c ? (modelData.c.url || "") : ""; sourceSize: Qt.size(56, 56); asynchronous: true }
                Text { visible: !!modelData.u; anchors.centerIn: parent; text: modelData.u || ""; font.pixelSize: 24 }
                MouseArea {
                    id: em
                    anchors.fill: parent
                    hoverEnabled: true
                    cursorShape: Qt.PointingHandCursor
                    onClicked: {
                        if (r.cb) r.cb(modelData.c ? modelData.c : { id: null, name: modelData.u, url: null, anim: false })
                        r.close()
                    }
                }
            }
        }
    }
}
