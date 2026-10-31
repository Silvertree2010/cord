import QtQuick
import QtQuick.Window
import cord.core

Window {
    width: 400; height: 200; visible: true; title: "cord"
    Hub { id: h }
    Text { anchors.centerIn: parent; text: h.st || "…"; color: "white" }
    color: "#111"
    Component.onCompleted: h.ping()
}
