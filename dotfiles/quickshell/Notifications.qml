import QtQuick
import QtQuick.Controls
import Quickshell
import Quickshell.Io
import Quickshell.Wayland
import "."

ShellRoot {
    id: root

    property string statePath: Quickshell.env("MD_NOTIFICATIONS_STATE") || "/tmp/md-notifications.json"
    property var notifications: []

    Shortcut {
        sequence: "Escape"
        onActivated: closeAllProc.running = true
    }

    FileView {
        id: stateFile
        path: root.statePath
        preload: true
        watchChanges: true
        onLoaded: {
            try { root.notifications = JSON.parse(stateFile.text()) } catch (e) { root.notifications = [] }
        }
    }

    PanelWindow {
        implicitWidth: 420
        implicitHeight: 400
        color: "transparent"

        anchors.top: true
        anchors.right: true
        margins.top: 20
        margins.right: 20

        WlrLayershell.namespace: "md.notifications"
        WlrLayershell.layer: WlrLayer.Overlay
        WlrLayershell.keyboardFocus: WlrKeyboardFocus.None

        Column {
            anchors.fill: parent
            spacing: 8

            Repeater {
                model: root.notifications
                Rectangle {
                    width: 420
                    height: 72
                    radius: Theme.radiusCard
                    color: Theme.fondo
                    border.width: 1
                    border.color: modelData.urgency === "critical" ? Theme.acentoRojo : Theme.bordeInactivo

                    Column {
                        anchors.fill: parent
                        anchors.margins: 12
                        spacing: 2

                        Text {
                            text: modelData.app || ""
                            color: modelData.urgency === "critical" ? Theme.acentoRojo : Theme.textoSuave
                            font.pixelSize: 10
                            font.weight: Font.DemiBold
                        }
                        Text {
                            text: modelData.summary || ""
                            color: Theme.texto
                            font.pixelSize: 13
                            font.weight: Font.Bold
                            width: parent.width
                            elide: Text.ElideRight
                        }
                        Text {
                            text: modelData.body || ""
                            color: Theme.textoSuave
                            font.pixelSize: 12
                            width: parent.width
                            elide: Text.ElideRight
                            visible: (modelData.body || "") !== ""
                        }
                        Rectangle {
                            visible: (modelData.progress || -1) >= 0
                            width: parent.width
                            height: 3
                            radius: 1
                            color: Theme.seleccion

                            Rectangle {
                                width: parent.width * (modelData.progress || 0) / 100
                                height: parent.height
                                radius: 1
                                color: modelData.urgency === "critical" ? Theme.acentoRojo : Theme.acentoVerde
                            }
                        }
                    }

                    // Botón cerrar
                    Rectangle {
                        width: 22
                        height: 22
                        radius: 4
                        anchors.top: parent.top
                        anchors.right: parent.right
                        anchors.margins: 6
                        color: closeArea.pressed ? Theme.seleccion : "transparent"

                        Text {
                            anchors.centerIn: parent
                            text: "✕"
                            color: Theme.texto
                            font.pixelSize: 12
                        }

                        MouseArea {
                            id: closeArea
                            anchors.fill: parent
                            onClicked: {
                                closeProc.command = ["md", "notify", "close", String(modelData.id)]
                                closeProc.running = true
                            }
                        }
                    }
                }
            }
        }
    }

    Process {
        id: closeProc
    }
    Process {
        id: closeAllProc
        command: ["md", "notify", "dismiss"]
    }
}