import QtQuick
import "."
import QtQuick.Controls
import Quickshell
import Quickshell.Io
import Quickshell.Wayland

ShellRoot {
    id: root

    Palette { id: palette }

    property var wallpapers: []
    property var filteredWallpapers: []
    property string selectedPath: ""
    property string titleText: "Fondos de pantalla"
    property string searchPlaceholder: "Buscar fondo…"
    property string searchText: ""

    function updateFilter() {
        if (searchText === "") { filteredWallpapers = wallpapers; return }
        var q = searchText.toLowerCase()
        var out = []
        for (var i = 0; i < wallpapers.length; i++) {
            var w = wallpapers[i]
            if ((w.name || "").toLowerCase().indexOf(q) >= 0) out.push(w)
        }
        filteredWallpapers = out
    }
    property string mdBin: Quickshell.env("HOME") + "/.local/bin/md"

    Process {
        id: proc
        command: [root.mdBin, "wallpaper", "--json"]
        running: true
        stdout: StdioCollector {
            onStreamFinished: {
                try {
                    const d = JSON.parse(text)
                    root.wallpapers = d.items || []
                    if (d.title) root.titleText = d.title
                    if (d.search) root.searchPlaceholder = d.search
                    root.updateFilter()
                }
                catch (e) { console.log("parse error", e) }
            }
        }
    }

    Process {
        id: applyProc
        running: false
    }

    // Paleta normal (sin espejo)
    readonly property color cFondo: palette.fondo
    readonly property color cTarjeta: palette.tarjeta
    readonly property color cSuperficie: palette.superficie
    readonly property color cHover: palette.hover
    readonly property color cSeleccion: palette.seleccion
    readonly property color cBorde: palette.seleccion
    readonly property color cBordeFuerte: palette.bordeInactivo
    readonly property color cTexto: palette.texto
    readonly property color cTextoSuave: palette.textoSuave
    readonly property color cAcento: palette.acentoAzul

    onWallpapersChanged: updateFilter()
    onSearchTextChanged: updateFilter()

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
            onTriggered: searchField.forceActiveFocus()
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
            Component.onCompleted: searchField.forceActiveFocus()

            Column {
                anchors.fill: parent
                anchors.margins: 16
                spacing: 12

                // Cabecera
                Item {
                    width: parent.width
                    height: 36

                    Text {
                        anchors.verticalCenter: parent.verticalCenter
                        anchors.left: parent.left
                        text: root.titleText
                        color: root.cTexto
                        font.pixelSize: 18
                        font.weight: Font.Bold
                        elide: Text.ElideRight
                        width: parent.width - 48
                    }

                    Rectangle {
                        anchors.verticalCenter: parent.verticalCenter
                        anchors.right: parent.right
                        width: 26
                        height: 26
                        radius: 8
                        color: closeMa.containsMouse ? root.cHover : "transparent"
                        border.width: 1
                        border.color: closeMa.containsMouse ? root.cBordeFuerte : "transparent"
                        Text {
                            anchors.centerIn: parent
                            text: "✕"
                            color: root.cTextoSuave
                            font.pixelSize: 16
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

                // Buscador
                Rectangle {
                    width: parent.width
                    height: 34
                    radius: 8
                    color: root.cSuperficie
                    border.width: 1
                    border.color: searchField.activeFocus ? root.cAcento : root.cBorde

                    Row {
                        anchors.fill: parent
                        anchors.leftMargin: 10
                        anchors.rightMargin: 10
                        spacing: 8

                        Text {
                            anchors.verticalCenter: parent.verticalCenter
                            text: "🔍"
                            color: root.cTextoSuave
                            font.pixelSize: 13
                        }
                        TextField {
                            id: searchField
                            anchors.verticalCenter: parent.verticalCenter
                            width: parent.width - 28
                            placeholderText: root.searchPlaceholder
                            color: "#ffffff"
                            placeholderTextColor: "#ffffff"
                            font.pixelSize: 13
                            background: null
                            palette { text: "#ffffff"; placeholderText: "#ffffff" }
                            onTextChanged: root.searchText = text
                            Keys.onEscapePressed: Qt.quit()
                        }
                    }
                }

                // Grid de miniaturas
                GridView {
                    id: grid
                    width: parent.width
                    height: parent.height - 120
                    cellWidth: grid.width > 0 ? Math.floor(grid.width / 4) : 150
                    cellHeight: 128
                    clip: true
                    model: root.filteredWallpapers

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

                Text {
                    anchors.horizontalCenter: parent.horizontalCenter
                    visible: root.filteredWallpapers.length === 0
                    text: "Sin resultados"
                    color: root.cTextoSuave
                    font.pixelSize: 12
                }
            }
        }
    }
}
