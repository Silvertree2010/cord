import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import Cord

ColumnLayout {
    id: r
    spacing: 12
    property string sv: "dc"
    property string mode: "qr"
    property bool busy: false
    property bool tos: false
    property bool qrStarted: false
    property string qerr: ""
    property string ferr: ""
    property int age: 0
    property var vals: ({})
    readonly property var fb: ({
        dc: [{ k: "token", label: "Token", secret: true, ph: "Account token" }],
        mx: [{ k: "hs", label: "Homeserver", secret: false, ph: "Homeserver (optional, matrix.org)" }, { k: "user", label: "Username", secret: false, ph: "Username or @user:server" }, { k: "pass", label: "Password", secret: true, ph: "Password" }, { k: "recovery", label: "Recovery key", secret: true, ph: "Recovery key (optional, for encrypted history)" }],
        st: [{ k: "base", label: "Server", secret: false, ph: "Instance API (https://api.stoat.chat)" }, { k: "email", label: "Email", secret: false, ph: "Email" }, { k: "pass", label: "Password", secret: true, ph: "Password" }, { k: "token", label: "Session token", secret: true, ph: "Session token (instead of email and password)" }]
    })
    readonly property var svcs: S.services.length ? S.services.map(x => [x.id, x.name]) : [["dc", "Discord"], ["mx", "Matrix"], ["st", "Stoat"]]
    readonly property var def: S.services.find(x => x.id === sv) || null
    readonly property var fields: def && def.fields && def.fields.length ? def.fields : (fb[sv] || [])
    readonly property bool tokOnly: sv !== "dc" || (def && def.modes ? def.modes.indexOf("qr") < 0 : false)
    readonly property bool qrOn: visible && r.sv === "dc" && r.mode === "qr" && r.tos
    readonly property bool ready: {
        const v = vals
        const h = k => !!v[k]
        if (sv === "dc") return h("token")
        if (sv === "mx") return h("user") && h("pass")
        if (sv === "st") return (h("email") && h("pass")) || h("token")
        return fields.every(f => h(f.k) || /optional|instead/i.test(f.ph || ""))
    }
    readonly property string keep: S.secretBackend === "file" ? "Your token is stored in a file on this computer without encryption." : S.secretBackend === "keyring" ? "Your token stays in the system keyring on this computer." : ""
    signal done
    function setV(k, t) {
        const o = Object.assign({}, vals)
        o[k] = t
        vals = o
        ferr = ""
    }
    function getQr() {
        S.qr = ""
        S.qrUsr = ""
        qerr = ""
        age = 0
        qrStarted = true
        S.c.dc_qr()
    }
    function stopQr() {
        if (!qrStarted) return
        qrStarted = false
        if (typeof S.c.qr_cancel === "function") S.c.qr_cancel()
    }
    function submit() {
        if (!ready || busy || (sv === "dc" && mode === "qr")) return
        U.guardDemo(() => {
            r.busy = true
            r.ferr = ""
            const o = {}
            for (const f of r.fields) o[f.k] = r.vals[f.k] || ""
            S.c.add(r.sv, JSON.stringify(o))
        })
    }
    Connections {
        target: S
        function onEv(d) {
            if (d.t === "Accs") {
                if (r.busy || r.qrOn) {
                    r.busy = false
                    r.qrStarted = false
                    r.done()
                }
            }
            if (d.t === "Err" && (d.d.ctx === "add" || d.d.ctx === "login")) {
                const was = r.busy
                r.busy = false
                if (d.d.ctx === "login") { if (r.qrOn && S.qr === "") r.qerr = d.d.msg }
                else if (was && r.visible) r.ferr = d.d.msg
            }
        }
    }
    onQrOnChanged: {
        if (qrOn && !S.qr) getQr()
        else if (!qrOn) stopQr()
    }
    Component.onCompleted: if (qrOn && !S.qr) getQr()
    Component.onDestruction: stopQr()
    Timer {
        interval: 1000
        repeat: true
        running: r.qrOn
        onTriggered: {
            r.age++
            if (S.qr === "" && r.qerr === "" && r.age > 20) r.qerr = "No response from Discord"
            else if (S.qr !== "" && r.age >= 100 && S.qrUsr === "") r.getQr()
        }
    }
    Row {
        spacing: 6
        Repeater {
            model: r.svcs
            Rectangle {
                width: 130
                height: 40
                radius: Th.rs
                color: r.sv === modelData[0] ? Th.bg4 : sm.containsMouse ? Th.hov : Th.bg3
                border.color: r.sv === modelData[0] ? Th.svcCol(modelData[0]) : Th.ln
                border.width: r.sv === modelData[0] ? 2 : 1
                activeFocusOnTab: true
                Accessible.role: Accessible.Button
                Accessible.name: modelData[1]
                Keys.onReturnPressed: sm.clicked(null)
                Keys.onSpacePressed: sm.clicked(null)
                Row {
                    anchors.centerIn: parent
                    spacing: 8
                    Badge { sv: modelData[0]; sz: 18; ring: "transparent"; anchors.verticalCenter: parent.verticalCenter }
                    Lbl { text: modelData[1]; font.weight: Font.DemiBold; anchors.verticalCenter: parent.verticalCenter }
                }
                MouseArea {
                    id: sm
                    anchors.fill: parent
                    hoverEnabled: true
                    cursorShape: Qt.PointingHandCursor
                    onClicked: {
                        if (r.sv === "dc" && modelData[0] !== "dc") r.stopQr()
                        r.sv = modelData[0]
                        r.vals = ({})
                        r.mode = modelData[0] === "dc" && !r.tokOnly ? "qr" : "tok"
                        r.ferr = ""
                    }
                }
            }
        }
    }
    Rectangle {
        visible: r.sv === "dc"
        Layout.preferredWidth: 400
        Layout.preferredHeight: tw.implicitHeight + 24
        radius: Th.rs
        color: Th.bg3
        border.color: Th.ln
        Lbl {
            id: tw
            x: 12
            y: 12
            width: parent.width - 24
            wrapMode: Text.Wrap
            elide: Text.ElideNone
            text: "Third party clients are against Discord's Terms of Service. Your account could be suspended."
            color: Th.tx2
        }
    }
    Btn {
        visible: r.sv === "dc" && !r.tos
        text: "Continue"
        height: 36
        onClicked: r.tos = true
    }
    Seg {
        visible: r.sv === "dc" && r.tos && !r.tokOnly
        opts: [["qr", "QR code"], ["tok", "Token"]]
        cur: r.mode
        onPick: v => r.mode = v
    }
    ColumnLayout {
        visible: r.sv === "dc" && r.mode === "qr" && r.tos
        spacing: 10
        Rectangle {
            Layout.preferredWidth: 220
            Layout.preferredHeight: 220
            radius: Th.rs
            color: "#e6e8ec"
            Image {
                anchors.centerIn: parent
                width: 196
                height: 196
                visible: S.qr !== "" && r.qerr === ""
                smooth: false
                source: S.qr ? S.c.qr(S.qr) : ""
            }
            Column {
                anchors.centerIn: parent
                spacing: 8
                visible: S.qr === "" && r.qerr === ""
                BusyIndicator { anchors.horizontalCenter: parent.horizontalCenter; running: parent.visible; width: 28; height: 28 }
                Lbl { text: "Waiting for code"; color: "#1c1f23"; anchors.horizontalCenter: parent.horizontalCenter }
            }
            Column {
                anchors.centerIn: parent
                spacing: 8
                visible: r.qerr !== ""
                width: 190
                Lbl { width: parent.width; horizontalAlignment: Text.AlignHCenter; text: "Could not get a code"; color: "#b3342f"; font.weight: Font.DemiBold }
                Lbl { width: parent.width; horizontalAlignment: Text.AlignHCenter; text: r.qerr; color: "#1c1f23"; font.pixelSize: Th.fsS; wrapMode: Text.Wrap; elide: Text.ElideNone; maximumLineCount: 3 }
                Btn { anchors.horizontalCenter: parent.horizontalCenter; text: "Retry"; onClicked: r.getQr() }
            }
        }
        Lbl { visible: S.qr !== "" && S.qrUsr === "" && r.qerr === ""; text: "Scan with the Discord app: Settings, Scan QR code"; color: Th.tx2 }
        Lbl { visible: S.qrUsr !== ""; text: "Confirm the login on your phone as " + S.qrUsr; color: Th.ok }
        Btn { visible: S.qr !== "" && r.qerr === ""; text: "New code"; onClicked: r.getQr() }
    }
    ColumnLayout {
        visible: !(r.sv === "dc" && (r.mode === "qr" || !r.tos))
        spacing: 8
        Repeater {
            model: r.fields
            Fld {
                pw: !!modelData.secret
                placeholderText: modelData.ph || modelData.label
                Layout.preferredWidth: 400
                onTextChanged: r.setV(modelData.k, text)
                onAccepted: r.submit()
            }
        }
        Lbl { visible: r.keep !== "" && r.sv !== "st"; text: r.sv === "mx" ? r.keep.replace("Your token", "Your session") : r.keep; color: Th.tx3; font.pixelSize: Th.fsS }
    }
    Lbl {
        visible: r.ferr !== "" && !(r.sv === "dc" && (r.mode === "qr" || !r.tos))
        Layout.preferredWidth: 400
        wrapMode: Text.Wrap
        elide: Text.ElideNone
        text: r.ferr
        color: Th.err
        font.pixelSize: Th.fsS
    }
    Row {
        spacing: 8
        visible: !(r.sv === "dc" && (r.mode === "qr" || !r.tos))
        Btn {
            text: r.busy ? "Signing in" : "Sign in"
            pri: true
            on: !r.busy && r.ready
            height: 36
            onClicked: r.submit()
        }
        BusyIndicator { visible: r.busy; running: r.busy; width: 30; height: 30 }
    }
}
