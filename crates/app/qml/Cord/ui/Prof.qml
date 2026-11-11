import QtQuick
import QtQuick.Controls
import Cord

Popup {
    id: r
    enter: PopIn {}
    exit: PopOut {}
    transformOrigin: Popup.TopLeft
    property var u: null
    objectName: "prof"
    property string uid: u && u.id ? S.ridOf(u.id) : ""
    property var p: S.pres[uid] || null
    width: 340
    padding: 0
    closePolicy: Popup.CloseOnEscape | Popup.CloseOnPressOutside
    onAboutToShow: U.reg(r)
    onClosed: U.unreg(r)
    background: Pane {}
    contentItem: ProfCard {
        id: pc
        implicitWidth: 340
        u: r.u
        gld: S.sel !== "home" && S.sel.indexOf(":") > 0 ? S.sel : ""
        popup: true
        playing: r.opened
        onClosed: r.close()
    }
}
