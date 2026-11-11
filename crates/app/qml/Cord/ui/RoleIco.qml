import QtQuick
import Cord

Item {
    id: r
    property var role: null
    property real sz: 16
    visible: !!r.role && !!r.role.icon
    width: visible ? r.sz : 0
    height: r.sz
    readonly property bool emo: !!r.role && !!r.role.icon && r.role.icon.indexOf("emoji:") === 0
    Image {
        anchors.fill: parent
        visible: !r.emo
        source: r.role && r.role.icon && !r.emo ? r.role.icon : ""
        sourceSize: Qt.size(r.sz * 2, r.sz * 2)
        asynchronous: true
        cache: true
    }
    Text { visible: r.emo; anchors.centerIn: parent; text: r.emo ? r.role.icon.slice(6) : ""; font.pixelSize: r.sz - 2 }
    HoverHandler { id: hh }
    Tip { text: r.role ? r.role.name : ""; hov: hh.hovered; side: "b" }
}
