import QtQuick
import QtQuick.Controls
import Quickshell
import Quickshell.Io
import Quickshell.Wayland
import "."

ShellRoot {
    id: root

    property string dataPath: Quickshell.env("MD_EMOJI_DATA") || ""
    property var emojis: []
    property var categories: []
    property string selectedCategory: "all"
    property string query: ""

    FileView {
        id: dataFile
        path: root.dataPath
        preload: true
        watchChanges: false
        onLoaded: {
            try {
                const d = JSON.parse(dataFile.text())
                root.emojis = d.emojis || []
                root.categories = d.categories || []
            } catch (e) {
                root.emojis = []
                root.categories = []
            }
        }
    }

    function filteredEmojis() {
        const q = root.query.trim().toLowerCase()
        return root.emojis.filter(function(e) {
            if (root.selectedCategory !== "all" && e.category !== root.selectedCategory) return false
            if (q === "") return true
            return (e.emoji + " " + (e.name || "")).toLowerCase().indexOf(q) !== -1
        })
    }

    Shortcut {
        sequence: "Escape"
        onActivated: Qt.quit()
    }

    PanelWindow {
        implicitWidth: 460
        implicitHeight: 560
        color: "transparent"

        anchors.top: true
        anchors.left: true
        margins.top: (screen ? Math.round((screen.height - implicitHeight) / 2) : 0)
        margins.left: (screen ? Math.round((screen.width - implicitWidth) / 2) : 0)

        WlrLayershell.namespace: "md.emoji"
        WlrLayershell.layer: WlrLayer.Overlay
        WlrLayershell.keyboardFocus: WlrKeyboardFocus.OnDemand

        // Clic fuera cierra
        Rectangle {
            anchors.fill: parent
            color: "#00000066"
            MouseArea {
                anchors.fill: parent
                onClicked: Qt.quit()
            }
        }

        Rectangle {
            id: card
            z: 1
            anchors.centerIn: parent
            width: 420
            height: 520
            radius: 14
            color: Theme.fondo
            border.width: 1
            border.color: Theme.bordeInactivo
            clip: true

            Column {
                anchors.fill: parent
                anchors.margins: 14
                spacing: 10

                Text {
                    text: "😀  Emojis"
                    color: Theme.texto
                    font.pixelSize: 16
                    font.weight: Font.Bold
                }

                // Búsqueda
                Rectangle {
                    width: parent.width
                    height: 40
                    radius: 8
                    color: Theme.superficie
                    border.width: 1
                    border.color: searchField.activeFocus ? Theme.bordeActivo : Theme.bordeInactivo

                    TextField {
                        id: searchField
                        anchors.fill: parent
                        anchors.margins: 8
                        placeholderText: "Buscar…"
                        color: Theme.texto
                        placeholderTextColor: Theme.textoSuave
                        font.pixelSize: 13
                        background: null
                        onTextChanged: root.query = text
                        Keys.onEscapePressed: Qt.quit()
                    }
                }

                // Categorías en dos filas horizontales (sin barra desplazable)
                Flow {
                    width: parent.width
                    spacing: 6
                    Repeater {
                        model: root.categories
                        Rectangle {
                            height: 28
                            width: catLabel.implicitWidth + 14
                            radius: 14
                            color: root.selectedCategory === modelData.key ? Theme.seleccion : Theme.superficie
                            border.width: 1
                            border.color: root.selectedCategory === modelData.key ? Theme.bordeActivo : Theme.bordeInactivo

                            Text {
                                id: catLabel
                                anchors.centerIn: parent
                                text: modelData.label || modelData.key
                                color: root.selectedCategory === modelData.key ? Theme.contrasteClaro : Theme.texto
                                font.pixelSize: 11
                                font.weight: Font.DemiBold
                            }

                            MouseArea {
                                anchors.fill: parent
                                cursorShape: Qt.PointingHandCursor
                                onClicked: root.selectedCategory = modelData.key
                            }
                        }
                    }
                }

                // Grid de emojis
                GridView {
                    id: grid
                    width: parent.width
                    height: Math.max(120, parent.height - 170)
                    cellWidth: 64
                    cellHeight: 64
                    clip: true
                    boundsBehavior: Flickable.StopAtBounds
                    model: root.filteredEmojis()
                    ScrollBar.vertical: ScrollBar { policy: ScrollBar.AsNeeded }

                    delegate: Rectangle {
                        width: 60
                        height: 60
                        radius: 8
                        color: emojiMa.containsMouse ? Theme.hover : "transparent"

                        Column {
                            anchors.centerIn: parent
                            spacing: 2

                            Text {
                                anchors.horizontalCenter: parent.horizontalCenter
                                text: modelData.emoji
                                font.pixelSize: 26
                            }
                            Text {
                                width: 56
                                anchors.horizontalCenter: parent.horizontalCenter
                                text: modelData.name || ""
                                color: Theme.textoSuave
                                font.pixelSize: 8
                                elide: Text.ElideRight
                                horizontalAlignment: Text.AlignHCenter
                            }
                        }

                        MouseArea {
                            id: emojiMa
                            anchors.fill: parent
                            hoverEnabled: true
                            cursorShape: Qt.PointingHandCursor
                            onClicked: root.copyEmoji(modelData.emoji)
                        }
                    }
                }
            }
        }
    }

    Process {
        id: copyProc
        onExited: Qt.quit()
    }

    function copyEmoji(e) {
        copyProc.command = ["wl-copy", e]
        copyProc.running = true
    }
}
