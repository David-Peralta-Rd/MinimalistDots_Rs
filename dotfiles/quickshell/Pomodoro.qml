import QtQuick
import QtQuick.Controls
import Quickshell
import Quickshell.Io
import Quickshell.Wayland
import "."

ShellRoot {
    id: root

    property int focusSec: parseInt(Quickshell.env("MD_POMODORO_FOCUS") || "1500")
    property int breakSec: parseInt(Quickshell.env("MD_POMODORO_BREAK") || "300")
    property int sessions: parseInt(Quickshell.env("MD_POMODORO_SESSIONS") || "4")

    property int currentSession: 1
    property int remaining: focusSec
    property string phase: "focus" // focus | break
    property bool paused: false
    property bool finished: false
    property bool collapsed: false

    property int pct: phase === "focus"
        ? Math.round((remaining / focusSec) * 100)
        : Math.round((remaining / breakSec) * 100)

    onPhaseChanged: collapsed = false

    Shortcut {
        sequence: "Escape"
        onActivated: Qt.quit()
    }

    function mmss(s) {
        var m = Math.floor(s / 60)
        var sec = s % 60
        return (m < 10 ? "0" : "") + m + ":" + (sec < 10 ? "0" : "") + sec
    }

    function tick() {
        if (paused || finished) return
        if (remaining > 0) {
            remaining--
            if (remaining === 0) phaseComplete()
        }
    }

    function phaseComplete() {
        soundProc.running = true
        if (phase === "focus") {
            phase = "break"
            remaining = breakSec
            notifyProc.command = ["notify-send", "Pomodoro", "Descanso"]
            notifyProc.running = true
        } else {
            currentSession++
            if (currentSession > sessions) {
                finished = true
                notifyProc.command = ["notify-send", "Pomodoro", "Completado"]
                notifyProc.running = true
                Qt.quit()
                return
            }
            phase = "focus"
            remaining = focusSec
            notifyProc.command = ["notify-send", "Pomodoro", "Concentración"]
            notifyProc.running = true
        }
    }

    function restart() {
        remaining = (phase === "focus" ? focusSec : breakSec)
    }

    Timer {
        interval: 1000
        running: true
        repeat: true
        onTriggered: root.tick()
    }

    // ---------- tarjeta pequeña (concentración o descanso minimizado) ----------
    PanelWindow {
        visible: root.phase === "focus" || root.collapsed
        implicitWidth: 90
        implicitHeight: 50
        color: "transparent"

        anchors.bottom: true
        anchors.left: true
        margins.bottom: 10
        margins.left: (screen ? Math.round((screen.width - implicitWidth) / 2) : 0)

        WlrLayershell.namespace: "md.pomodoro.small"
        WlrLayershell.layer: WlrLayer.Overlay
        WlrLayershell.keyboardFocus: WlrKeyboardFocus.None
        WlrLayershell.exclusiveZone: 0

        Rectangle {
            anchors.fill: parent
            color: Theme.fondo
            radius: Theme.radiusCard
            border.width: Theme.borderWidth
            border.color: Theme.bordeInactivo

            Column {
                anchors.fill: parent
                anchors.margins: 4
                spacing: 2

                Text {
                    width: parent.width
                    text: root.mmss(root.remaining)
                    color: root.paused ? Theme.textoSuave : Theme.texto
                    font.pixelSize: 12
                    font.weight: Font.Bold
                    horizontalAlignment: Text.AlignHCenter
                }

                Text {
                    width: parent.width
                    text: root.currentSession + "/" + root.sessions
                    color: Theme.textoSuave
                    font.pixelSize: 8
                    horizontalAlignment: Text.AlignHCenter
                }

                Rectangle {
                    width: parent.width
                    height: 2
                    radius: 1
                    color: Theme.seleccion

                    Rectangle {
                        width: parent.width * root.pct / 100
                        height: parent.height
                        radius: 1
                        color: Theme.acentoRojo
                    }
                }

                Row {
                    anchors.horizontalCenter: parent.horizontalCenter
                    spacing: 2

                    Rectangle {
                        width: 20
                        height: 16
                        radius: 3
                        color: pa.hasHover ? Theme.hover : "transparent"

                        Text {
                            anchors.centerIn: parent
                            text: root.paused ? "▶" : "❚❚"
                            color: Theme.texto
                            font.pixelSize: 9
                        }

                        MouseArea {
                            id: pa
                            anchors.fill: parent
                            hoverEnabled: true
                            onClicked: root.paused = !root.paused
                        }
                    }

                    Rectangle {
                        width: 20
                        height: 16
                        radius: 3
                        color: ra.hasHover ? Theme.hover : "transparent"

                        Text {
                            anchors.centerIn: parent
                            text: "↻"
                            color: Theme.texto
                            font.pixelSize: 9
                        }

                        MouseArea {
                            id: ra
                            anchors.fill: parent
                            hoverEnabled: true
                            onClicked: root.restart()
                        }
                    }

                    Rectangle {
                        width: 20
                        height: 16
                        radius: 3
                        color: sa.hasHover ? Theme.hover : "transparent"

                        Text {
                            anchors.centerIn: parent
                            text: "✕"
                            color: Theme.texto
                            font.pixelSize: 9
                        }

                        MouseArea {
                            id: sa
                            anchors.fill: parent
                            hoverEnabled: true
                            onClicked: Qt.quit()
                        }
                    }
                }
            }
        }
    }

    // ---------- tarjeta grande (descanso) ----------
    PanelWindow {
        visible: root.phase === "break" && !root.collapsed
        implicitWidth: 500
        implicitHeight: 320
        color: "transparent"

        anchors.top: true
        anchors.left: true
        margins.top: (screen ? Math.round((screen.height - implicitHeight) / 2) : 0)
        margins.left: (screen ? Math.round((screen.width - implicitWidth) / 2) : 0)

        WlrLayershell.namespace: "md.pomodoro.break"
        WlrLayershell.layer: WlrLayer.Overlay
        WlrLayershell.keyboardFocus: WlrKeyboardFocus.None
        WlrLayershell.exclusiveZone: 0

        Rectangle {
            anchors.fill: parent
            color: Theme.fondo
            radius: 22
            border.width: 2
            border.color: root.paused ? Theme.textoSuave : Theme.acentoRojo

            Column {
                anchors.fill: parent
                anchors.margins: 24
                spacing: 10

                Text {
                    width: parent.width
                    text: root.paused ? "⏸  En pausa" : "☕  Hora de descansar"
                    color: root.paused ? Theme.textoSuave : Theme.acentoRojo
                    font.pixelSize: 24
                    font.weight: Font.Bold
                    horizontalAlignment: Text.AlignHCenter
                }

                Text {
                    width: parent.width
                    text: root.mmss(root.remaining)
                    color: Theme.texto
                    font.pixelSize: 72
                    font.weight: Font.Bold
                    horizontalAlignment: Text.AlignHCenter
                }

                Text {
                    width: parent.width
                    text: root.currentSession + "/" + root.sessions
                    color: Theme.textoSuave
                    font.pixelSize: 14
                    horizontalAlignment: Text.AlignHCenter
                }

                Text {
                    width: parent.width
                    text: "Levántate, estírate y mira a lo lejos."
                    color: Theme.textoSuave
                    font.pixelSize: 13
                    horizontalAlignment: Text.AlignHCenter
                }

                Rectangle {
                    width: parent.width
                    height: 6
                    radius: 3
                    color: Theme.seleccion

                    Rectangle {
                        width: parent.width * root.pct / 100
                        height: parent.height
                        radius: 3
                        color: Theme.acentoRojo
                    }
                }

                Row {
                    anchors.horizontalCenter: parent.horizontalCenter
                    spacing: 8

                    Rectangle {
                        width: 104
                        height: 30
                        radius: 8
                        color: bpa.hasHover ? Theme.hover : Theme.superficie

                        Text {
                            anchors.centerIn: parent
                            text: root.paused ? "▶ Reanudar" : "❚❚ Pausar"
                            color: Theme.texto
                            font.pixelSize: 12
                        }

                        MouseArea {
                            id: bpa
                            anchors.fill: parent
                            hoverEnabled: true
                            onClicked: root.paused = !root.paused
                        }
                    }

                    Rectangle {
                        width: 104
                        height: 30
                        radius: 8
                        color: bra.hasHover ? Theme.hover : Theme.superficie

                        Text {
                            anchors.centerIn: parent
                            text: "↻ Reiniciar"
                            color: Theme.texto
                            font.pixelSize: 12
                        }

                        MouseArea {
                            id: bra
                            anchors.fill: parent
                            hoverEnabled: true
                            onClicked: root.restart()
                        }
                    }

                    Rectangle {
                        width: 104
                        height: 30
                        radius: 8
                        color: bca.hasHover ? Theme.hover : Theme.superficie

                        Text {
                            anchors.centerIn: parent
                            text: "▾ Minimizar"
                            color: Theme.texto
                            font.pixelSize: 12
                        }

                        MouseArea {
                            id: bca
                            anchors.fill: parent
                            hoverEnabled: true
                            onClicked: root.collapsed = true
                        }
                    }

                    Rectangle {
                        width: 104
                        height: 30
                        radius: 8
                        color: bsa.hasHover ? Theme.hover : Theme.superficie

                        Text {
                            anchors.centerIn: parent
                            text: "✕ Detener"
                            color: Theme.texto
                            font.pixelSize: 12
                        }

                        MouseArea {
                            id: bsa
                            anchors.fill: parent
                            hoverEnabled: true
                            onClicked: Qt.quit()
                        }
                    }
                }
            }
        }
    }

    Process { id: soundProc; command: ["md", "pomodoro", "sound"] }
    Process { id: notifyProc }
}