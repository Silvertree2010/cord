import QtQuick
import Cord

Rectangle {
    id: gp
    default property alias content: grpC.data
    width: parent ? parent.width : 400
    height: grpC.implicitHeight
    radius: Th.r
    color: Th.bg
    border.color: Th.ln
    Column { id: grpC; width: parent.width }
}
