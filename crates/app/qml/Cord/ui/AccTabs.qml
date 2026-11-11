import QtQuick
import Cord

Item {
    id: r
    property int cur: 0
    readonly property var list: S.cfgAccs
    readonly property int n: list.length
    width: parent ? parent.width : 400
    height: 52

    Flickable {
        id: fl
        anchors.fill: parent
        contentWidth: row.width
        contentHeight: height
        clip: true
        boundsBehavior: Flickable.StopAtBounds
        interactive: contentWidth > width

        Rectangle {
            id: pill
            readonly property var it: row.children.length > r.cur ? row.children[Math.min(r.cur, row.children.length - 1)] : null
            x: it ? it.x : 0
            y: 4
            width: it ? it.width : 0
            height: 44
            radius: Th.rs + 2
            color: Th.bg4
            border.color: Th.a(Th.ac, 0.45)
            border.width: 1
            Behavior on x { NumberAnimation { duration: Th.mM; easing.type: Easing.OutCubic } }
            Behavior on width { NumberAnimation { duration: Th.mM; easing.type: Easing.OutCubic } }
        }

        Row {
            id: row
            y: 4
            spacing: 6
            Repeater {
                model: r.n
                Item {
                    id: tb
                    readonly property var a: r.list[index]
                    readonly property var live: S.accs[a.id]
                    readonly property bool on: r.cur === index
                    readonly property bool up: S.up.indexOf(a.id) >= 0
                    width: tc.implicitWidth + 28
                    height: 44
                    activeFocusOnTab: true
                    Accessible.role: Accessible.PageTab
                    Accessible.name: lb.text
                    Keys.onReturnPressed: r.cur = index
                    Keys.onSpacePressed: r.cur = index
                    Rectangle { anchors.fill: parent; radius: Th.rs + 2; color: hv.hovered && !tb.on ? Th.hov : "transparent" }
                    HoverHandler { id: hv; cursorShape: Qt.PointingHandCursor }
                    TapHandler { onTapped: r.cur = index }
                    Row {
                        id: tc
                        anchors.centerIn: parent
                        spacing: 8
                        Item {
                            width: 28
                            height: 28
                            anchors.verticalCenter: parent.verticalCenter
                            Av { sz: 28; src: tb.live ? (tb.live.me.av || "") : ""; name: tb.live ? tb.live.me.name : "?" }
                            Rectangle {
                                x: 20
                                y: 20
                                width: 10
                                height: 10
                                radius: 5
                                color: tb.up ? Th.ok : Th.tx3
                                border.color: tb.on ? Th.bg4 : Th.bg
                                border.width: 2
                            }
                        }
                        Column {
                            anchors.verticalCenter: parent.verticalCenter
                            spacing: 1
                            Lbl { id: lb; text: tb.live ? (tb.live.me.disp || tb.live.me.name) : tb.a.base === "demo" ? S.t("Demo") : S.t("Offline"); color: tb.on ? Th.tx : Th.tx2; font.weight: Font.DemiBold; font.pixelSize: Th.fsS + 1; Behavior on color { ColorAnimation { duration: Th.mF } } }
                            Lbl { text: Th.svcName(S.tag(tb.a.svc)); color: Th.tx3; font.pixelSize: Th.fsP }
                        }
                    }
                }
            }
            Item {
                id: ad
                readonly property bool on: r.cur === r.n
                width: ac.implicitWidth + 28
                height: 44
                activeFocusOnTab: true
                Accessible.role: Accessible.PageTab
                Accessible.name: "Add account"
                Keys.onReturnPressed: r.cur = r.n
                Keys.onSpacePressed: r.cur = r.n
                Rectangle { anchors.fill: parent; radius: Th.rs + 2; color: ahv.hovered && !ad.on ? Th.hov : "transparent" }
                HoverHandler { id: ahv; cursorShape: Qt.PointingHandCursor }
                TapHandler { onTapped: r.cur = r.n }
                Row {
                    id: ac
                    anchors.centerIn: parent
                    spacing: 8
                    Ico { n: "plus"; sz: 18; col: ad.on ? Th.tx : Th.tx2; anchors.verticalCenter: parent.verticalCenter }
                    Lbl { text: S.t("Add account"); color: ad.on ? Th.tx : Th.tx2; font.weight: Font.DemiBold; font.pixelSize: Th.fsS + 1; anchors.verticalCenter: parent.verticalCenter }
                }
            }
        }
    }
}
