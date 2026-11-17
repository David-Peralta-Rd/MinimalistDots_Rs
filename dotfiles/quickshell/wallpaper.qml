import QtQuick
import QtQuick.Controls
import Quickshell
import Quickshell.Io

ShellRoot {
    id: root

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

    // Paleta (theme/palette.toml) - estilo normal, sin espejo
    readonly property color cFondo: "#1e222a"
    readonly property color cSuperficie: "#282c34"
    readonly property color cSeleccion: "#3e4451"
    readonly property color cBorde: "#4b5263"
    readonly property color cTexto: "#abb2bf"
    readonly property color cTextoSuave: "#5c6370"
    readonly property color cAcento: "#7aa2f7"

    PanelWindow {
        anchors { top: true; left: true }
        margins { top: 80; left: 80 }
        implicitWidth: 560
        implicitHeight: 520
        color: root.cFondo

        Column {
            anchors.fill: parent
            anchors.margins: 12
            spacing: 8

            Text {
                text: "Fondos de pantalla"
                color: root.cTexto
                font.pixelSize: 16
                font.weight: Font.Bold
            }

            GridView {
                id: grid
                width: parent.width
                height: parent.height - 32
                cellWidth: 132
                cellHeight: 110
                clip: true
                model: root.wallpapers

                delegate: Rectangle {
                    width: 124
                    height: 102
                    radius: 6
                    color: root.cSuperficie
                    border.width: 1
                    border.color: (modelData.path === root.selectedPath) ? root.cAcento : root.cBorde

                    Column {
                        anchors.centerIn: parent
                        spacing: 4
                        Image {
                            source: "file://" + (modelData.path || "")
                            width: 110
                            height: 70
                            fillMode: Image.PreserveAspectCrop
                            smooth: true
                        }
                        Text {
                            width: 110
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
