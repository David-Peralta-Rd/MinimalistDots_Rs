import QtQuick
import QtQuick.Controls
import Quickshell
import Quickshell.Io
import Quickshell.Wayland

ShellRoot {
    id: root

    property var wallpapers: []
    property string selectedPath: ""
    property string mdBin: Quickshell.env("HOME") + "/.local/bin/md"

    Process {
        id: proc
        command: [root.mdBin, "wallpaper", "--json"]
        running: true
        stdout: StdioCollector {
            onStreamFinished: {
                try { root.wallpapers = JSON.parse(text) }
                catch (e) { console.log("parse error", e) }
            }
        }
    }

    Process {
        id: applyProc
        running: false
    }

    // Paleta normal (sin espejo)
    readonly property color cFondo: "#1e222a"
    readonly property color cTarjeta: "#23272f"
    readonly property color cSuperficie: "#282c34"
    readonly property color cHover: "#323742"
    readonly property color cSeleccion: "#3e4451"
    readonly property color cBorde: "#3e4451"
    readonly property color cBordeFuerte: "#4b5263"
    readonly property color cTexto: "#abb2bf"
    readonly property color cTextoSuave: "#5c6370"
    readonly property color cAcento: "#7aa2f7"

    PanelWindow {
        id: win
        anchors { top: true; left: true }
        margins.top: Math.max(0, (screen ? (screen.height - implicitHeight) / 2 : 0))
        margins.left: Math.max(0, (screen ? (screen.width - implicitWidth) / 2 : 0))
        implicitWidth: 720
        implicitHeight: 560
        color: "transparent"

        WlrLayershell.namespace: "md-wallpaper"
        WlrLayershell.layer: WlrLayer.Top
        WlrLayershell.keyboardFocus: WlrKeyboardFocus.Exclusive

        Timer {
            interval: 100
            running: true
            repeat: false
            onTriggered: card.forceActiveFocus()
        }

        // Fondo oscuro suave detrás del panel (clic fuera cierra)
        Rectangle {
            anchors.fill: parent
            color: "#00000066"
            MouseArea {
                anchors.fill: parent
                onClicked: Qt.quit()
            }
        }

        // Tarjeta principal
        Rectangle {
            id: card
            z: 1
            anchors.centerIn: parent
            width: 680
            height: 520
            radius: 14
            color: root.cFondo
            border.width: 1
            border.color: root.cBordeFuerte
            focus: true
            Keys.onEscapePressed: Qt.quit()
            Component.onCompleted: forceActiveFocus()

            Column {
                anchors.fill: parent
                anchors.margins: 16
                spacing: 12

                // Cabecera
                Row {
                    width: parent.width
                    height: 32
                    spacing: 8

                    Text {
                        anchors.verticalCenter: parent.verticalCenter
                        text: "Fondos de pantalla"
                        color: root.cTexto
                        font.pixelSize: 18
                        font.weight: Font.Bold
                    }

                    Item { width: parent.width - 220; height: 1 }

                    Rectangle {
                        width: 32
                        height: 32
                        radius: 8
                        color: closeMa.containsMouse ? root.cHover : "transparent"
                        border.width: 1
                        border.color: closeMa.containsMouse ? root.cBordeFuerte : "transparent"
                        Text {
                            anchors.centerIn: parent
                            text: "✕"
                            color: root.cTextoSuave
                            font.pixelSize: 14
                            font.weight: Font.Medium
                        }
                        MouseArea {
                            id: closeMa
                            anchors.fill: parent
                            hoverEnabled: true
                            cursorShape: Qt.PointingHandCursor
                            onClicked: Qt.quit()
                        }
                    }
                }

                // Grid de miniaturas
                GridView {
                    id: grid
                    width: parent.width
                    height: parent.height - 50
                    cellWidth: grid.width > 0 ? Math.floor(grid.width / 4) : 150
                    cellHeight: 128
                    clip: true
                    model: root.wallpapers

                    delegate: Rectangle {
                        width: grid.cellWidth - 6
                        height: 120
                        x: 3
                        radius: 10
                        color: root.cSuperficie
                        border.width: (modelData.path === root.selectedPath) ? 2 : 1
                        border.color: (modelData.path === root.selectedPath) ? root.cAcento : root.cBorde

                        Column {
                            anchors.centerIn: parent
                            spacing: 6

                            Rectangle {
                                width: 124
                                height: 80
                                radius: 6
                                color: "#0a0a0a"
                                clip: true
                                border.width: 1
                                border.color: root.cBorde

                                Image {
                                    anchors.fill: parent
                                    source: "file://" + (modelData.path || "")
                                    fillMode: Image.PreserveAspectCrop
                                    smooth: true
                                    asynchronous: true
                                }
                            }

                            Text {
                                width: 124
                                text: modelData.name || ""
                                color: root.cTextoSuave
                                font.pixelSize: 9
                                elide: Text.ElideMiddle
                                horizontalAlignment: Text.AlignHCenter
                            }
                        }

                        MouseArea {
                            anchors.fill: parent
                            hoverEnabled: true
                            cursorShape: Qt.PointingHandCursor
                            onEntered: parent.color = root.cHover
                            onExited: parent.color = root.cSuperficie
                            onClicked: {
                                root.selectedPath = modelData.path || ""
                                applyProc.command = [root.mdBin, "wallpaper", "--apply", modelData.path || ""]
                                applyProc.running = true
                            }
                        }
                    }

                    ScrollBar.vertical: ScrollBar {
                        policy: ScrollBar.AsNeeded
                        width: 8
                    }
                }
            }
        }
    }
}
