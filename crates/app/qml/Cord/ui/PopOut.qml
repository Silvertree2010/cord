import QtQuick
import Cord

Transition {
    ParallelAnimation {
        NumberAnimation { property: "opacity"; to: 0; duration: Th.mF; easing.type: Easing.InCubic }
        NumberAnimation { property: "scale"; to: 0.97; duration: Th.mF; easing.type: Easing.InCubic }
    }
}
