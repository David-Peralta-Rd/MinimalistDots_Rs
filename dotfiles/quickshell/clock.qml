import QtQuick
import Quickshell
import Quickshell.Io
import Quickshell.Wayland

ShellRoot {
    id: root

    property string textColor: "#abb2bf"
    property string shadowColor: "#00000080"

    Process {
        id: colorProc
        command: [Quickshell.env("HOME") + "/.local/bin/md", "clock", "--color"]
        running: true
        stdout: StdioCollector {
            onStreamFinished: {
                try {
                    const d = JSON.parse(text)
                    if (d.text) root.textColor = d.text
                    // sombra opuesta para legibilidad
                    root.shadowColor = (root.textColor === "#e8e8e8") ? "#00000099" : "#ffffff99"
                } catch (e) { }
            }
        }
    }

    Timer {
        interval: 60000
        running: true
        repeat: true
        onTriggered: { colorProc.running = false; colorProc.running = true }
    }

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
        color: "transparent"

        WlrLayershell.namespace: "md-clock"
        WlrLayershell.layer: WlrLayer.Bottom
        WlrLayershell.keyboardFocus: WlrKeyboardFocus.None

        Column {
            anchors { right: parent.right; rightMargin: 12; verticalCenter: parent.verticalCenter }
            spacing: 4

            Text {
                anchors.right: parent.right
                text: Qt.formatDateTime(clock.date, "HH:mm")
                color: root.textColor
                font.family: "monospace"
                font.pixelSize: 56
                font.weight: Font.ExtraBold
                style: Text.Outline
                styleColor: root.shadowColor
            }
            Text {
                anchors.right: parent.right
                text: Qt.locale().toString(clock.date, "ddd, dd MMM").toLowerCase()
                color: root.textColor
                font.family: "monospace"
                font.pixelSize: 16
                font.weight: Font.Medium
                style: Text.Outline
                styleColor: root.shadowColor
            }
        }
    }
}
