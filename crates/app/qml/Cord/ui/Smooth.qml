import QtQuick
import Cord

WheelHandler {
    id: w
    property var f: parent && parent.contentY !== undefined ? parent : parent ? parent.parent : null
    readonly property bool busy: an.running
    property real dest: 0
    property real lastT: 0
    property real boost: 1
    acceptedDevices: PointerDevice.Mouse
    orientation: Qt.Vertical
    target: null
    onWheel: e => {
        const d = e.angleDelta.y
        if (!d || !w.f) return
        const lo = w.f.originY - w.f.topMargin
        const hi = Math.max(lo, w.f.originY + w.f.contentHeight + w.f.bottomMargin - w.f.height)
        const now = Date.now()
        w.boost = now - w.lastT < 90 ? Math.min(3, w.boost + 0.35) : 1
        w.lastT = now
        const base = an.running ? w.dest : w.f.contentY
        w.dest = Math.max(lo, Math.min(hi, base - d / 120 * 180 * w.boost))
        an.to = w.dest
        an.restart()
        e.accepted = true
    }
    property NumberAnimation an: NumberAnimation { target: w.f; property: "contentY"; duration: Th.motion ? 170 : 0; easing.type: Easing.OutQuad }
}
