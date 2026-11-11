import QtQuick
import QtQuick.Shapes
import Cord

Item {
    id: r
    property var usr: null
    property string text: ""
    property color col: Th.tx
    property real fs: Th.fs
    property int weight: Font.DemiBold
    readonly property var nc: r.usr && r.usr.ncol && r.usr.ncol.length && S.ui.nstyle !== false ? r.usr.ncol : []
    readonly property bool grad: r.nc.length >= 2
    function hex(n) { return "#" + ("000000" + (n >>> 0).toString(16)).slice(-6) }
    implicitWidth: t.implicitWidth
    implicitHeight: t.implicitHeight
    baselineOffset: t.baselineOffset
    clip: r.grad && r.width < t.implicitWidth
    Lbl {
        id: t
        width: r.width
        text: r.text
        color: r.nc.length === 1 ? Th.lift(r.hex(r.nc[0]), Th.bgc, 3) : r.col
        opacity: r.grad ? 0 : 1
        font.pixelSize: r.fs
        font.weight: r.weight
    }
    Loader {
        active: r.grad
        sourceComponent: Component {
            Shape {
            width: t.implicitWidth
            height: t.implicitHeight
            preferredRendererType: Shape.CurveRenderer
            ShapePath {
                strokeColor: "transparent"
                fillGradient: LinearGradient {
                    x1: 0
                    y1: 0
                    x2: t.implicitWidth
                    y2: 0
                    GradientStop { position: 0; color: r.grad ? Th.lift(r.hex(r.nc[0]), Th.bgc, 3) : "white" }
                    GradientStop { position: 1; color: r.grad ? Th.lift(r.hex(r.nc[r.nc.length - 1]), Th.bgc, 3) : "white" }
                }
                PathText {
                    x: 0
                    y: 0
                    text: r.grad ? r.text : ""
                    font.family: Th.font
                    font.pixelSize: r.fs
                    font.weight: r.weight
                }
            }
        }
        }
    }
}
