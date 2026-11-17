import QtQuick
import Quickshell
import Quickshell.Wayland

ShellRoot {
    SystemClock {
        id: clock
        precision: SystemClock.Minutes
    }

    PanelWindow {
        anchors { left: true; top: true }
        margins.top: Math.round(((screen ? screen.height : 1080) - implicitHeight) / 2) + 40
        margins.left: 20
        implicitWidth: 280
        implicitHeight: 120
        color: "#1e222a"

        WlrLayershell.namespace: "md-clock"
        WlrLayershell.layer: WlrLayer.Bottom
        WlrLayershell.keyboardFocus: WlrKeyboardFocus.None

        Column {
            anchors { right: parent.right; rightMargin: 12; verticalCenter: parent.verticalCenter }
            spacing: 4

            Text {
                anchors.right: parent.right
                text: Qt.formatDateTime(clock.date, "HH:mm")
                color: "#abb2bf"
                font.family: "monospace"
                font.pixelSize: 56
                font.weight: Font.ExtraBold
            }
            Text {
                anchors.right: parent.right
                text: Qt.locale().toString(clock.date, "ddd, dd MMM").toLowerCase()
                color: "#5c6370"
                font.family: "monospace"
                font.pixelSize: 16
                font.weight: Font.Medium
            }
        }
    }
}
