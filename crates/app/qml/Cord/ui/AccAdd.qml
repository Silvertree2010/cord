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
    property string qerr: ""
    property string ferr: ""
    property int age: 0
    readonly property bool qrOn: visible && r.sv === "dc" && r.mode === "qr" && r.tos
    readonly property bool ready: sv === "dc" ? dtok.text.length > 0
        : sv === "mx" ? mhs.text.length > 0 && mus.text.length > 0 && mpw.text.length > 0
        : (sem.text.length > 0 && spw.text.length > 0) || stk.text.length > 0
    readonly property string keep: S.secretBackend === "file" ? "Your token is stored in a file on this computer without encryption." : S.secretBackend === "keyring" ? "Your token stays in the system keyring on this computer." : ""
    signal done
    function getQr() {
        S.qr = ""
        S.qrUsr = ""
        qerr = ""
        age = 0
        S.c.dc_qr()
    }
    function submit() {
        if (!ready || busy || (sv === "dc" && mode === "qr")) return
        U.guardDemo(() => {
            r.busy = true
            r.ferr = ""
            if (r.sv === "dc") S.c.add("dc", JSON.stringify({ token: dtok.text }))
            if (r.sv === "mx") S.c.add("mx", JSON.stringify({ hs: mhs.text, user: mus.text, pass: mpw.text, recovery: mrk.text }))
            if (r.sv === "st") S.c.add("st", JSON.stringify({ base: sba.text, email: sem.text, pass: spw.text, token: stk.text }))
        })
    }
    Connections {
        target: S
        function onEv(d) {
            if (d.t === "Accs") { r.busy = false; r.done() }
            if (d.t === "Err") {
                const was = r.busy
                r.busy = false
                if (r.qrOn && S.qr === "") r.qerr = d.d.msg
                else if (was && r.visible) r.ferr = d.d.msg
            }
        }
    }
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
    onQrOnChanged: if (qrOn && !S.qr) getQr()
    Component.onCompleted: if (qrOn && !S.qr) getQr()
    Row {
        spacing: 6
        Repeater {
            model: [["dc", "Discord"], ["mx", "Matrix"], ["st", "Stoat"]]
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
                MouseArea { id: sm; anchors.fill: parent; hoverEnabled: true; cursorShape: Qt.PointingHandCursor; onClicked: { r.sv = modelData[0]; r.mode = modelData[0] === "dc" ? "qr" : "pw"; r.ferr = "" } }
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
        onClicked: U.guardDemo(() => { r.tos = true })
    }
    Seg {
        visible: r.sv === "dc" && r.tos
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
        visible: r.sv === "dc" && r.mode === "tok" && r.tos
        spacing: 8
        Fld { id: dtok; pw: true; placeholderText: "Account token"; Layout.preferredWidth: 400; onTextChanged: r.ferr = ""; onAccepted: r.submit() }
        Lbl { visible: r.keep !== ""; text: r.keep; color: Th.tx3; font.pixelSize: Th.fsS }
    }
    ColumnLayout {
        visible: r.sv === "mx"
        spacing: 8
        Fld { id: mhs; placeholderText: "Homeserver (matrix.org)"; Layout.preferredWidth: 400; onTextChanged: r.ferr = ""; onAccepted: mus.forceActiveFocus() }
        Fld { id: mus; placeholderText: "Username or @user:server"; Layout.preferredWidth: 400; onTextChanged: r.ferr = ""; onAccepted: mpw.forceActiveFocus() }
        Fld { id: mpw; pw: true; placeholderText: "Password"; Layout.preferredWidth: 400; onTextChanged: r.ferr = ""; onAccepted: r.submit() }
        Fld { id: mrk; pw: true; placeholderText: "Recovery key (optional, for encrypted history)"; Layout.preferredWidth: 400; onTextChanged: r.ferr = ""; onAccepted: r.submit() }
        Lbl { visible: r.keep !== ""; text: r.keep.replace("Your token", "Your session"); color: Th.tx3; font.pixelSize: Th.fsS }
    }
    ColumnLayout {
        visible: r.sv === "st"
        spacing: 8
        Fld { id: sba; placeholderText: "Instance API (https://api.stoat.chat)"; Layout.preferredWidth: 400; onTextChanged: r.ferr = ""; onAccepted: sem.forceActiveFocus() }
        Fld { id: sem; placeholderText: "Email"; Layout.preferredWidth: 400; onTextChanged: r.ferr = ""; onAccepted: spw.forceActiveFocus() }
        Fld { id: spw; pw: true; placeholderText: "Password"; Layout.preferredWidth: 400; onTextChanged: r.ferr = ""; onAccepted: r.submit() }
        Lbl { text: "Or paste a session token instead of email and password"; color: Th.tx3; font.pixelSize: Th.fsS }
        Fld { id: stk; pw: true; placeholderText: "Session token"; Layout.preferredWidth: 400; onTextChanged: r.ferr = ""; onAccepted: r.submit() }
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
