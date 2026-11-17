import QtQuick
import QtQuick.Controls
import Quickshell
import Quickshell.Io

ShellRoot {
    id: root

    Shortcut {
        sequence: "Escape"
        onActivated: Qt.quit()
    }

    property var wallpapers: []
    property string selectedPath: ""

    Process {
        id: proc
        command: ["md", "wallpaper", "--json"]
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

    readonly property color cFondo: "#1e222a"
    readonly property color cSuperficie: "#282c34"
    readonly property color cSeleccion: "#3e4451"
    readonly property color cBorde: "#4b5263"
    readonly property color cTexto: "#abb2bf"
    readonly property color cTextoSuave: "#5c6370"
    readonly property color cAcento: "#7aa2f7"

    PanelWindow {
        id: win
        anchors { top: true; left: true }
        margins.top: (screen ? (screen.height - implicitHeight) / 2 : 0)
        margins.left: (screen ? (screen.width - implicitWidth) / 2 : 0)
        implicitWidth: 600
        implicitHeight: 560
        color: root.cFondo

        // WlrLayershell not available here; default layer is fine

        // Cerrar con Escape (manejado por tecla global en ShellRoot)


        Column {
            anchors.fill: parent
            anchors.margins: 16
            spacing: 10

            // Cabecera
            Row {
                width: parent.width
                Text {
                    text: "Fondos de pantalla"
                    color: root.cTexto
                    font.pixelSize: 18
                    font.weight: Font.Bold
                }
                Item { width: parent.width - 180; height: 1 }
                Rectangle {
                    width: 28
                    height: 28
                    radius: 6
                    color: closeMa.containsMouse ? root.cSeleccion : "transparent"
                    Text {
                        anchors.centerIn: parent
                        text: "✕"
                        color: root.cTextoSuave
                        font.pixelSize: 14
                    }
                    MouseArea {
                        id: closeMa
                        anchors.fill: parent
                        hoverEnabled: true
                        onClicked: Qt.quit()
                    }
                }
            }

            GridView {
                id: grid
                width: parent.width
                height: parent.height - 40
                cellWidth: 140
                cellHeight: 120
                clip: true
                model: root.wallpapers

                delegate: Rectangle {
                    width: 132
                    height: 112
                    radius: 8
                    color: root.cSuperficie
                    border.width: (modelData.path === root.selectedPath) ? 2 : 1
                    border.color: (modelData.path === root.selectedPath) ? root.cAcento : root.cBorde

                    Column {
                        anchors.centerIn: parent
                        spacing: 6
                        Rectangle {
                            width: 116
                            height: 76
                            radius: 4
                            color: "#000000"
                            clip: true
                            Image {
                                anchors.fill: parent
                                source: "file://" + (modelData.path || "")
                                fillMode: Image.PreserveAspectCrop
                                smooth: true
                            }
                        }
                        Text {
                            width: 116
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
                        onClicked: {
                            root.selectedPath = modelData.path || ""
                            applyProc.command = ["md", "wallpaper", "--apply", modelData.path || ""]
                            applyProc.running = true
                        }
                    }
                }

                ScrollBar.vertical: ScrollBar { policy: ScrollBar.AsNeeded }
            }
        }
    }
}
