import QtQuick
import Quickshell
import Quickshell.Wayland
import "."

PanelWindow {
    id: root

    property int panelWidth: 80
    property int panelHeight: 40
    property color fillColor: Theme.tarjeta
    property color fillColorEnd: Theme.seleccion
    property color borderColor: Theme.bordeActivo
    property int cornerRadius: Theme.radiusPill

    implicitWidth: panelWidth
    implicitHeight: panelHeight
    color: "transparent"

    WlrLayershell.namespace: "md-panel"
    WlrLayershell.layer: WlrLayer.Overlay
    WlrLayershell.keyboardFocus: WlrKeyboardFocus.None

    Rectangle {
        anchors.fill: parent
        radius: root.cornerRadius
        border.width: Theme.borderWidth
        border.color: root.borderColor

        gradient: Gradient {
            orientation: Gradient.Horizontal
            GradientStop { position: 0.0; color: root.fillColor }
            GradientStop { position: 1.0; color: root.fillColorEnd }
        }
    }
}
