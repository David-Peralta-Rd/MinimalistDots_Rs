import QtQuick
import "."

// Botón con borde suave (glow sutil), similar a SoftCard.
Rectangle {
    id: root

    property int softRadius: 4
    property color softColor: "transparent"
    property color softBorder: Theme.bordeInactivo
    property color softGlow: Qt.rgba(softBorder.r, softBorder.g, softBorder.b, 0.35)

    radius: softRadius
    color: softColor
    border.width: 1
    border.color: softBorder
    antialiasing: true
    smooth: true

    Rectangle {
        anchors.fill: parent
        anchors.margins: -2
        radius: root.softRadius + 2
        color: "transparent"
        border.width: 1
        border.color: root.softGlow
        antialiasing: true
        smooth: true
        z: -1
    }
}