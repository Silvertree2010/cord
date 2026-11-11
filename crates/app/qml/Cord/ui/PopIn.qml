import QtQuick
import Cord

Transition {
    ParallelAnimation {
        NumberAnimation { property: "opacity"; from: 0; to: 1; duration: Th.mF + 20; easing.type: Easing.OutCubic }
        NumberAnimation { property: "scale"; from: 0.95; to: 1; duration: Th.mM; easing.type: Easing.OutCubic }
    }
}
