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
    signal done
    Connections {
        target: S
        function onEv(d) {
            if (d.t === "Accs") { r.busy = false; r.done() }
            if (d.t === "Err") r.busy = false
        }
    }
    Row {
        spacing: 6
        Repeater {
            model: [["dc", "Discord"], ["mx", "Matrix"], ["st", "Stoat"]]
            Rectangle {
                width: 130
                height: 44
                radius: Th.rs
                color: r.sv === modelData[0] ? Th.sel : sm.containsMouse ? Th.hov : Th.bg3
                border.color: r.sv === modelData[0] ? Th.svcCol(modelData[0]) : "transparent"
                border.width: 2
                Row {
                    anchors.centerIn: parent
                    spacing: 8
                    Badge { sv: modelData[0]; sz: 18; ring: "transparent"; anchors.verticalCenter: parent.verticalCenter }
                    Lbl { text: modelData[1]; font.weight: Font.DemiBold; anchors.verticalCenter: parent.verticalCenter }
                }
                MouseArea { id: sm; anchors.fill: parent; hoverEnabled: true; cursorShape: Qt.PointingHandCursor; onClicked: { r.sv = modelData[0]; r.mode = modelData[0] === "dc" ? "qr" : "pw" } }
            }
        }
    }
    Row {
        visible: r.sv === "dc"
        spacing: 6
        Btn { text: "QR code"; flat: r.mode !== "qr"; onClicked: r.mode = "qr" }
        Btn { text: "Token"; flat: r.mode !== "tok"; onClicked: r.mode = "tok" }
    }
    ColumnLayout {
        visible: r.sv === "dc" && r.mode === "qr"
        spacing: 10
        Lbl { text: "Scan with the Discord mobile app: Settings, Scan QR code"; color: Th.tx2; wrapMode: Text.Wrap; Layout.preferredWidth: 400 }
        Rectangle {
            width: 220
            height: 220
            radius: Th.r
            color: "#fff"
            Image {
                anchors.centerIn: parent
                width: 196
                height: 196
                visible: S.qr !== ""
                smooth: false
                source: S.qr ? S.c.qr(S.qr) : ""
            }
            BusyIndicator { anchors.centerIn: parent; running: S.qr === ""; visible: running }
        }
        Lbl { visible: S.qrUsr !== ""; text: "Confirm the login on your phone as " + S.qrUsr; color: Th.ok }
        Btn { text: S.qr ? "New code" : "Get code"; onClicked: { S.qr = ""; S.qrUsr = ""; S.c.dc_qr() } }
        onVisibleChanged: if (visible && !S.qr) S.c.dc_qr()
        Component.onCompleted: if (visible && !S.qr) S.c.dc_qr()
    }
    ColumnLayout {
        visible: r.sv === "dc" && r.mode === "tok"
        spacing: 8
        Fld { id: dtok; pw: true; placeholderText: "Account token"; Layout.preferredWidth: 400 }
        Lbl { text: "Your token stays in the system keyring on this computer."; color: Th.tx3; font.pixelSize: Th.fsS }
    }
    ColumnLayout {
        visible: r.sv === "mx"
        spacing: 8
        Fld { id: mhs; placeholderText: "Homeserver (matrix.org)"; Layout.preferredWidth: 400 }
        Fld { id: mus; placeholderText: "Username or @user:server"; Layout.preferredWidth: 400 }
        Fld { id: mpw; pw: true; placeholderText: "Password"; Layout.preferredWidth: 400 }
        Fld { id: mrk; pw: true; placeholderText: "Recovery key (optional, for encrypted history)"; Layout.preferredWidth: 400 }
    }
    ColumnLayout {
        visible: r.sv === "st"
        spacing: 8
        Fld { id: sba; placeholderText: "Instance API (https://api.stoat.chat)"; Layout.preferredWidth: 400 }
        Fld { id: sem; placeholderText: "Email"; Layout.preferredWidth: 400 }
        Fld { id: spw; pw: true; placeholderText: "Password"; Layout.preferredWidth: 400 }
        Lbl { text: "Or paste a session token instead of email and password"; color: Th.tx3; font.pixelSize: Th.fsS }
        Fld { id: stk; pw: true; placeholderText: "Session token"; Layout.preferredWidth: 400 }
    }
    Row {
        spacing: 8
        visible: !(r.sv === "dc" && r.mode === "qr")
        Btn {
            text: r.busy ? "Signing in" : "Sign in"
            pri: true
            on: !r.busy
            onClicked: {
                r.busy = true
                if (r.sv === "dc") S.c.add("dc", JSON.stringify({ token: dtok.text }))
                if (r.sv === "mx") S.c.add("mx", JSON.stringify({ hs: mhs.text, user: mus.text, pass: mpw.text, recovery: mrk.text }))
                if (r.sv === "st") S.c.add("st", JSON.stringify({ base: sba.text, email: sem.text, pass: spw.text, token: stk.text }))
            }
        }
        BusyIndicator { visible: r.busy; running: r.busy; width: 30; height: 30 }
    }
    Lbl {
        visible: r.sv === "dc"
        Layout.preferredWidth: 420
        wrapMode: Text.Wrap
        text: "Third party clients are against Discord's terms. Your account could be suspended."
        color: Th.tx3
        font.pixelSize: 11
    }
}
