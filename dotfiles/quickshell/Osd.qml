import QtQuick
import Quickshell
import Quickshell.Wayland
import "."

ShellRoot {
    id: root

    property string label: Quickshell.env("MD_OSD_LABEL") || "70%"
    property bool muted: Quickshell.env("MD_OSD_MUTED") === "true"
    property color accentColor: Theme.acentoVerde
    property int lifetimeMs: 800

    PanelWindow {
        id: panel
        implicitWidth: 80
        implicitHeight: 40
        color: "transparent"

        anchors.top: true
        margins.top: 20
        margins.left: (screen ? Math.round((screen.width - implicitWidth) / 2) : 0)

        WlrLayershell.namespace: "md.osd"
        WlrLayershell.layer: WlrLayer.Overlay
        WlrLayershell.keyboardFocus: WlrKeyboardFocus.None
        WlrLayershell.exclusiveZone: 0

        Rectangle {
            anchors.fill: parent
            radius: Theme.radiusPill
            border.width: Theme.borderWidth
            border.color: root.muted ? Theme.acentoRojo : Theme.bordeActivo

            gradient: Gradient {
                orientation: Gradient.Horizontal
                GradientStop { position: 0.0; color: Theme.tarjeta }
                GradientStop { position: 1.0; color: Theme.seleccion }
            }

            Text {
                anchors.centerIn: parent
                text: root.label
                color: root.muted ? Theme.acentoRojo : Theme.contrasteClaro
                font.family: Theme.fontFamily
                font.pixelSize: Theme.fontSizeLarge
                font.weight: Font.Bold
                horizontalAlignment: Text.AlignHCenter
                verticalAlignment: Text.AlignVCenter
            }
        }
    }

    Timer {
        interval: root.lifetimeMs
        running: true
        repeat: false
        onTriggered: Qt.quit()
    }
}