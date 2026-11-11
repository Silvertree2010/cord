import QtQuick
import QtQuick.Window
import QtWebView
import Cord

Window {
    id: w
    width: 420
    height: 640
    visible: true
    title: S.t("cord · captcha")
    color: "#1c1f23"
    property bool sent: false
    readonly property string html: "<!doctype html><html><head><meta name=viewport content='width=device-width'><style>html,body{margin:0;height:100%;background:#1c1f23;display:flex;align-items:center;justify-content:center}</style>"
        + "<script>window.__t='';function go(){var id=hcaptcha.render('c',{sitekey:'" + S.cap.sk + "',theme:'dark',callback:function(t){window.__t=t}});"
        + (S.cap.rq ? "hcaptcha.setData(id,{rqdata:'" + S.cap.rq + "'});" : "") + "hcaptcha.execute(id)}</script>"
        + "<script src='https://js.hcaptcha.com/1/api.js?render=explicit&onload=go' async defer></script></head><body><div id=c></div></body></html>"
    WebView {
        id: v
        anchors.fill: parent
        Component.onCompleted: loadHtml(w.html, "https://discord.com/login")
    }
    Timer {
        interval: 500
        repeat: true
        running: !w.sent
        onTriggered: v.runJavaScript("window.__t || ''", t => {
            if (t && !w.sent) {
                w.sent = true
                S.c.qr_cap(t)
                S.cap = null
            }
        })
    }
    onClosing: if (!sent) S.cap = null
}
