import QtQuick
import Quickshell
import Quickshell.Wayland

// Reloj de escritorio (reemplaza al widget desktop_clock de eww).
// Capa "bottom": queda detrás de las ventanas y no reserva espacio.
ShellRoot {
    SystemClock {
        id: clock
        precision: SystemClock.Minutes
    }

    PanelWindow {
        id: win
        // izquierda, centrado en vertical y 40px más abajo (igual que el eww anterior)
        anchors { left: true; top: true }
        margins.top: Math.round(((screen ? screen.height : 1080) - implicitHeight) / 2) + 40
        implicitWidth: 280
        implicitHeight: 120
        color: "transparent"
        exclusionMode: ExclusionMode.Ignore

        WlrLayershell.namespace: "md-clock"
        WlrLayershell.layer: WlrLayer.Bottom
        WlrLayershell.keyboardFocus: WlrKeyboardFocus.None

        Column {
            anchors { right: parent.right; rightMargin: 10; verticalCenter: parent.verticalCenter }
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
