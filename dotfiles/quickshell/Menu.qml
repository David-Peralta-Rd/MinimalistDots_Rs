import QtQuick
import QtQuick.Controls
import Quickshell
import Quickshell.Io
import Quickshell.Wayland
import "."

ShellRoot {
    id: root

    property string entriesPath: Quickshell.env("MD_MENU_ENTRIES") || ""
    property string resultPath: Quickshell.env("MD_MENU_RESULT") || ""
    property string prompt: Quickshell.env("MD_MENU_PROMPT") || ""
    property bool canDelete: (Quickshell.env("MD_MENU_CAN_DELETE") || "false") === "true"
    property bool canWipe: (Quickshell.env("MD_MENU_CAN_WIPE") || "false") === "true"
    property bool submitEmpty: (Quickshell.env("MD_MENU_SUBMIT_EMPTY") || "false") === "true"
    property string wipeLabel: Quickshell.env("MD_MENU_WIPE_LABEL") || "🧹 Borrar todo"
    property string confirmWipeLabel: Quickshell.env("MD_MENU_CONFIRM_WIPE_LABEL") || "¿Seguro? Pulsa otra vez"

    property var entries: []
    property string query: ""
    property var filtered: []
    property int currentIndex: 0
    property bool confirmWipe: false

    Component.onCompleted: loadEntries()

    function loadEntries() {
        if (entriesPath === "") { entries = []; filtered = []; return }
        var xhr = new XMLHttpRequest()
        xhr.open("GET", "file://" + entriesPath)
        xhr.onreadystatechange = function() {
            if (xhr.readyState === XMLHttpRequest.DONE && xhr.status === 200) {
                try { entries = JSON.parse(xhr.responseText) } catch (e) { entries = [] }
                updateFilter()
            }
        }
        xhr.send()
    }

    function updateFilter() {
        var q = query.toLowerCase()
        filtered = entries.filter(function(e) {
            return (e.label || "").toLowerCase().indexOf(q) !== -1
        })
        if (currentIndex >= filtered.length) currentIndex = Math.max(0, filtered.length - 1)
    }

    function writeResult(payload) {
        writeProc.command = ["sh", "-c", "printf '%s' '" + payload + "' > " + resultPath]
        writeProc.running = true
    }

    function selectCurrent() {
        if (filtered.length === 0) {
            if (submitEmpty) writeResult('{"type":"typed","query":"' + query.replace(/"/g, '\\"') + '"}')
            return
        }
        var originalIndex = entries.indexOf(filtered[currentIndex])
        writeResult('{"type":"selected","index":' + originalIndex + '}')
    }

    function deleteCurrent() {
        if (filtered.length === 0) return
        var originalIndex = entries.indexOf(filtered[currentIndex])
        writeResult('{"type":"delete","index":' + originalIndex + '}')
    }

    function requestWipe() {
        if (!confirmWipe) {
            confirmWipe = true
            wipeTimer.restart()
            return
        }
        confirmWipe = false
        wipeTimer.stop()
        writeResult('{"type":"wipe"}')
    }

    PanelWindow {
        implicitWidth: 600
        implicitHeight: 400
        color: "transparent"

        anchors.top: true
        anchors.left: true
        margins.top: (screen ? Math.round((screen.height - implicitHeight) / 2) : 0)
        margins.left: (screen ? Math.round((screen.width - implicitWidth) / 2) : 0)

        WlrLayershell.namespace: "md.menu"
        WlrLayershell.layer: WlrLayer.Overlay
        WlrLayershell.keyboardFocus: WlrKeyboardFocus.OnDemand

        Rectangle {
            anchors.fill: parent
            radius: Theme.radiusCard
            color: Theme.fondo
            border.width: Theme.borderWidth
            border.color: Theme.bordeActivo

            Column {
                anchors.fill: parent
                anchors.margins: 10
                spacing: 8

                // Barra de búsqueda
                Rectangle {
                    width: parent.width
                    height: 40
                    radius: 6
                    color: Theme.superficie

                    Row {
                        anchors.fill: parent
                        anchors.margins: 8
                        spacing: 8

                        TextField {
                            id: search
                            width: parent.width - (wipeBtn.visible ? wipeBtn.width + 8 : 0)
                            height: parent.height
                            text: root.query
                            onTextChanged: { root.query = text; root.updateFilter() }
                            onAccepted: root.selectCurrent()
                            Keys.onEscapePressed: root.writeResult('{"type":"cancelled"}')
                            Keys.onUpPressed: { if (root.currentIndex > 0) root.currentIndex--; }
                            Keys.onDownPressed: { if (root.currentIndex < root.filtered.length - 1) root.currentIndex++ }
                        }

                        Rectangle {
                            id: wipeBtn
                            visible: root.canWipe
                            width: 106
                            height: 28
                            radius: 4
                            color: root.confirmWipe ? Theme.acentoRojo : Theme.seleccion

                            Text {
                                anchors.centerIn: parent
                                text: root.confirmWipe ? root.confirmWipeLabel : root.wipeLabel
                                color: root.confirmWipe ? Theme.fondo : Theme.texto
                                font.pixelSize: 12
                            }

                            MouseArea {
                                anchors.fill: parent
                                onClicked: root.requestWipe()
                            }
                        }
                    }
                }

                // Lista
                ListView {
                    id: listView
                    width: parent.width
                    height: parent.height - 48
                    clip: true
                    model: root.filtered
                    currentIndex: root.currentIndex
                    onCurrentIndexChanged: root.currentIndex = currentIndex

                    delegate: Rectangle {
                        width: ListView.view.width
                        height: 40
                        radius: 4
                        color: index === root.currentIndex ? Theme.seleccion : "transparent"

                        Row {
                            anchors.verticalCenter: parent.verticalCenter
                            anchors.left: parent.left
                            anchors.leftMargin: 8
                            spacing: 8

                            Image {
                                width: 24
                                height: 24
                                source: modelData.icon ? ("file://" + modelData.icon) : ""
                                visible: !!modelData.icon
                            }
                            Text {
                                text: modelData.label || ""
                                color: index === root.currentIndex ? Theme.acentoVerde : Theme.texto
                                font.pixelSize: 14
                                font.weight: index === root.currentIndex ? Font.DemiBold : Font.Normal
                                verticalAlignment: Text.AlignVCenter
                            }
                        }

                        MouseArea {
                            anchors.fill: parent
                            hoverEnabled: true
                            onEntered: root.currentIndex = index
                            onClicked: root.selectCurrent()
                        }

                        // Botón borrar fila
                        Rectangle {
                            visible: root.canDelete && index === root.currentIndex
                            width: 26
                            height: 22
                            radius: 4
                            anchors.right: parent.right
                            anchors.rightMargin: 8
                            anchors.verticalCenter: parent.verticalCenter
                            color: "transparent"

                            Text {
                                anchors.centerIn: parent
                                text: "🗑"
                                color: Theme.texto
                            }

                            MouseArea {
                                anchors.fill: parent
                                onClicked: root.deleteCurrent()
                            }
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

    Timer {
        id: wipeTimer
        interval: 3000
        running: false
        repeat: false
        onTriggered: root.confirmWipe = false
    }
}