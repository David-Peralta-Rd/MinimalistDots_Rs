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
        onFileChanged: {
            try { stateFile.reload() } catch (e) {}
            try { root.notifications = JSON.parse(stateFile.text()) } catch (e) { root.notifications = [] }
        }
    }

    PanelWindow {
        implicitWidth: 340
        implicitHeight: notifColumn.implicitHeight
        visible: root.notifications.length > 0
        color: "transparent"

        anchors.top: true
        anchors.right: true
        margins.top: 10
        margins.right: 10

        WlrLayershell.namespace: "md.notifications"
        WlrLayershell.layer: WlrLayer.Overlay
        WlrLayershell.keyboardFocus: WlrKeyboardFocus.None
        WlrLayershell.exclusiveZone: 0

        mask: Region { item: notifColumn }

        Column {
            id: notifColumn
            anchors.top: parent.top
            anchors.left: parent.left
            anchors.right: parent.right
            spacing: 8

            Repeater {
                model: root.notifications
                SoftCard {
                    width: 340
                    implicitHeight: contentCol.implicitHeight + 24
                    softRadius: Theme.radiusCard
                    softColor: Theme.fondo
                    softBorder: modelData.urgency === "critical" ? Theme.acentoRojo : Theme.bordeInactivo

                    Column {
                        id: contentCol
                        anchors.left: parent.left
                        anchors.right: parent.right
                        anchors.top: parent.top
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
                            wrapMode: Text.WordWrap
                        }
                        Text {
                            text: modelData.body || ""
                            color: Theme.textoSuave
                            font.pixelSize: 12
                            width: parent.width
                            wrapMode: Text.WordWrap
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
