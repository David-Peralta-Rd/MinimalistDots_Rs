import QtQuick
import QtQuick.Controls
import Quickshell
import Quickshell.Io
import Quickshell.Wayland
import "."

ShellRoot {
    id: root

    property string entriesPath: Quickshell.env("MD_LAUNCHER_ENTRIES") || ""
    property string resultPath: Quickshell.env("MD_LAUNCHER_RESULT") || ""
    property string prompt: Quickshell.env("MD_LAUNCHER_PROMPT") || "🚀"
    property var entries: []
    property string query: ""
    property var filtered: []

    Shortcut {
        sequence: "Escape"
        onActivated: root.cancel()
    }

    FileView {
        id: entriesFile
        path: root.entriesPath
        preload: true
        watchChanges: true
        onLoaded: {
            try { root.entries = JSON.parse(entriesFile.text()) } catch (e) { root.entries = [] }
            root.updateFilter()
        }
    }

    Component.onCompleted: search.forceActiveFocus()

    function updateFilter() {
        var q = query.toLowerCase()
        filtered = entries.filter(function(e) {
            return (e.label || "").toLowerCase().indexOf(q) !== -1
        })
    }

    function select(index) {
        writeResult('{"selected":' + index + '}')
    }

    function cancel() {
        writeResult('{"selected":-1}')
    }

    function writeResult(payload) {
        writeProc.command = ["sh", "-c", "printf '%s' '" + payload + "' > " + resultPath]
        writeProc.running = true
    }

    PanelWindow {
        implicitWidth: 500
        implicitHeight: 460
        color: "transparent"

        anchors.top: true
        anchors.left: true
        margins.top: (screen ? Math.round((screen.height - implicitHeight) / 2) : 0)
        margins.left: (screen ? Math.round((screen.width - implicitWidth) / 2) : 0)

        WlrLayershell.namespace: "md.launcher"
        WlrLayershell.layer: WlrLayer.Overlay
        WlrLayershell.keyboardFocus: WlrKeyboardFocus.OnDemand

        MouseArea {
            anchors.fill: parent
            onClicked: root.cancel()
        }

        SoftCard {
            anchors.fill: parent
            softRadius: Theme.radiusCard
            softColor: Theme.fondo
            softBorder: Theme.bordeActivo

            Column {
                anchors.fill: parent
                anchors.margins: 12
                spacing: 8

                Rectangle {
                    width: parent.width
                    height: 40
                    radius: Theme.radiusCard
                    color: Theme.superficie
                    border.width: Theme.borderWidth
                    border.color: search.activeFocus ? Theme.bordeActivo : Theme.bordeInactivo

                    TextField {
                        id: search
                        anchors.fill: parent
                        anchors.margins: 8
                        placeholderText: root.prompt
                        color: Theme.texto
                        placeholderTextColor: Theme.textoSuave
                        font.pixelSize: Theme.fontSizeNormal
                        background: null
                        focus: true
                        onTextChanged: { root.query = text; root.updateFilter() }
                        onAccepted: {
                            if (root.filtered.length > 0) root.select(root.entries.indexOf(root.filtered[0]))
                        }
                        Keys.onEscapePressed: root.cancel()
                    }
                }

                ListView {
                    width: parent.width
                    height: parent.height - search.height - 16
                    clip: true
                    model: root.filtered
                    delegate: Rectangle {
                        width: ListView.view.width
                        height: 40
                        color: mouseArea.containsMouse ? Theme.hover : "transparent"
                        radius: Theme.radiusCard
                        border.width: mouseArea.containsMouse ? Theme.borderWidth : 0
                        border.color: mouseArea.containsMouse ? Theme.bordeActivo : "transparent"

                        Row {
                            anchors.verticalCenter: parent.verticalCenter
                            anchors.left: parent.left
                            anchors.leftMargin: 12
                            spacing: 10

                            Image {
                                width: 26
                                height: 26
                                source: modelData.icon ? ("file://" + modelData.icon) : ""
                                visible: !!modelData.icon
                                fillMode: Image.PreserveAspectFit
                            }
                            Text {
                                text: modelData.label || ""
                                color: Theme.texto
                                font.pixelSize: Theme.fontSizeNormal
                                verticalAlignment: Text.AlignVCenter
                            }
                        }

                        MouseArea {
                            id: mouseArea
                            anchors.fill: parent
                            hoverEnabled: true
                            onClicked: root.select(root.entries.indexOf(modelData))
                        }
                    }
                }
            }

        }
    }

    Process {
        id: writeProc
        onExited: Qt.quit()
    }

    Keys.onEscapePressed: root.cancel()
}