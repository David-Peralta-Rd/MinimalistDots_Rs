import QtQuick
import "."

// Tarjeta con borde suave (glow sutil) para Launcher/Menu/Notifications.
Rectangle {
    id: root

    property int softRadius: Theme.radiusCard
    property color softColor: Theme.fondo
    property color softBorder: Theme.bordeInactivo
    property color softGlow: Qt.rgba(softBorder.r, softBorder.g, softBorder.b, 0.35)

    radius: softRadius
    color: softColor
    border.width: 1
    border.color: softBorder
    antialiasing: true
    smooth: true

    // Borde exterior difuso.
    Rectangle {
        anchors.fill: parent
        anchors.margins: -3
        radius: root.softRadius + 3
        color: "transparent"
        border.width: 1
        border.color: root.softGlow
        antialiasing: true
        smooth: true
        z: -1
    }
}