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

    property string focusLabel: Quickshell.env("MD_POMODORO_FOCUS_LABEL") || "Concentración"
    property string breakLabel: Quickshell.env("MD_POMODORO_BREAK_LABEL") || "Descanso"
    property string pausedLabel: Quickshell.env("MD_POMODORO_PAUSED_LABEL") || "En pausa"
    property string pauseLabel: Quickshell.env("MD_POMODORO_PAUSE_LABEL") || "Pausar"
    property string resumeLabel: Quickshell.env("MD_POMODORO_RESUME_LABEL") || "Reanudar"
    property string restartLabel: Quickshell.env("MD_POMODORO_RESTART_LABEL") || "Reiniciar"
    property string closeLabel: Quickshell.env("MD_POMODORO_CLOSE_LABEL") || "Cerrar"

    property int currentSession: 1
    property int remaining: focusSec
    property string phase: "focus" // focus | break
    property bool paused: false
    property bool finished: false

    Shortcut {
        sequence: "Escape"
        onActivated: Qt.quit()
    }

    function mmss(s) {
        var m = Math.floor(s / 60)
        var sec = s % 60
        return (m < 10 ? "0" : "") + m + ":" + (sec < 10 ? "0" : "") + sec
    }

    function minutes(s) {
        return Math.max(1, Math.ceil(s / 60))
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
            notifyProc.command = ["notify-send", "Pomodoro", breakLabel]
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
            notifyProc.command = ["notify-send", "Pomodoro", focusLabel]
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

    // Ventana compacta en la mitad inferior.
    PanelWindow {
        visible: root.phase === "focus"
        implicitWidth: 200
        implicitHeight: 92
        color: "transparent"

        anchors.bottom: true
        anchors.left: true
        margins.bottom: 24
        margins.left: (screen ? Math.round((screen.width - implicitWidth) / 2) : 0)

        WlrLayershell.namespace: "md.pomodoro.focus"
        WlrLayershell.layer: WlrLayer.Overlay
        WlrLayershell.keyboardFocus: WlrKeyboardFocus.None
        WlrLayershell.exclusiveZone: 0

        SoftCard {
            anchors.fill: parent
            softRadius: Theme.radiusCard
            softColor: Qt.rgba(Theme.fondo.r, Theme.fondo.g, Theme.fondo.b, 0.55)
            softBorder: Theme.acentoRojo

            Column {
                anchors.fill: parent
                anchors.margins: 6
                spacing: 2

                Text {
                    width: parent.width
                    text: root.paused ? "⏸" : "⏱"
                    color: Theme.acentoRojo
                    font.pixelSize: 11
                    font.weight: Font.Bold
                    horizontalAlignment: Text.AlignHCenter
                }

                Text {
                    width: parent.width
                    text: root.minutes(root.remaining) + " min"
                    color: Theme.texto
                    font.pixelSize: 16
                    font.weight: Font.ExtraBold
                    horizontalAlignment: Text.AlignHCenter
                }

                Text {
                    width: parent.width
                    text: root.currentSession + "/" + root.sessions
                    color: Theme.textoSuave
                    font.pixelSize: 9
                    horizontalAlignment: Text.AlignHCenter
                }

                Row {
                    anchors.horizontalCenter: parent.horizontalCenter
                    spacing: 4

                    Button {
                        implicitWidth: 32
                        implicitHeight: 22
                        font.pixelSize: 11
                        background: Rectangle { radius: Theme.radiusPill; color: Theme.seleccion }
                        text: root.paused ? "▶" : "❚❚"
                        onClicked: root.paused = !root.paused
                    }
                    Button {
                        implicitWidth: 32
                        implicitHeight: 22
                        font.pixelSize: 11
                        background: Rectangle { radius: Theme.radiusPill; color: Theme.seleccion }
                        text: "↻"
                        onClicked: root.restart()
                    }
                    Button {
                        implicitWidth: 32
                        implicitHeight: 22
                        font.pixelSize: 11
                        background: Rectangle { radius: Theme.radiusPill; color: Theme.seleccion }
                        text: "✕"
                        onClicked: Qt.quit()
                    }
                }
            }
        }
    }

    // Ventana central en modo descanso.
    PanelWindow {
        visible: root.phase === "break"
        implicitWidth: 300
        implicitHeight: 160
        color: "transparent"

        anchors.top: true
        anchors.left: true
        margins.top: (screen ? Math.round((screen.height - implicitHeight) / 2) : 0)
        margins.left: (screen ? Math.round((screen.width - implicitWidth) / 2) : 0)

        WlrLayershell.namespace: "md.pomodoro.break"
        WlrLayershell.layer: WlrLayer.Overlay
        WlrLayershell.keyboardFocus: WlrKeyboardFocus.None
        WlrLayershell.exclusiveZone: 0

        SoftCard {
            anchors.fill: parent
            softRadius: Theme.radiusCard
            softColor: Theme.fondo
            softBorder: Theme.acentoVerde

            Column {
                anchors.fill: parent
                anchors.margins: 8
                spacing: 4

                Text {
                    width: parent.width
                    text: root.breakLabel
                    color: Theme.acentoVerde
                    font.pixelSize: 14
                    font.weight: Font.Bold
                    horizontalAlignment: Text.AlignHCenter
                }

                Text {
                    width: parent.width
                    text: root.mmss(root.remaining)
                    color: Theme.texto
                    font.pixelSize: 24
                    font.weight: Font.ExtraBold
                    horizontalAlignment: Text.AlignHCenter
                }

                Text {
                    width: parent.width
                    text: root.currentSession + "/" + root.sessions
                    color: Theme.textoSuave
                    font.pixelSize: 10
                    horizontalAlignment: Text.AlignHCenter
                }

                Row {
                    anchors.horizontalCenter: parent.horizontalCenter
                    spacing: 6

                    Button {
                        implicitWidth: 32
                        implicitHeight: 22
                        font.pixelSize: 11
                        background: Rectangle { radius: Theme.radiusPill; color: Theme.seleccion }
                        text: root.paused ? root.resumeLabel : root.pauseLabel
                        onClicked: root.paused = !root.paused
                    }
                    Button {
                        implicitWidth: 32
                        implicitHeight: 22
                        font.pixelSize: 11
                        background: Rectangle { radius: Theme.radiusPill; color: Theme.seleccion }
                        text: root.restartLabel
                        onClicked: root.restart()
                    }
                    Button {
                        implicitWidth: 32
                        implicitHeight: 22
                        font.pixelSize: 11
                        background: Rectangle { radius: Theme.radiusPill; color: Theme.seleccion }
                        text: root.closeLabel
                        onClicked: Qt.quit()
                    }
                }
            }
        }
    }

    Process { id: soundProc; command: ["md", "pomodoro", "sound"] }
    Process { id: notifyProc }
}