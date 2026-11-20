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

    PanelWindow {
        implicitWidth: 190
        implicitHeight: 96
        color: "transparent"

        anchors.bottom: true
        anchors.right: true
        margins.bottom: 20
        margins.right: 20

        WlrLayershell.namespace: "md.pomodoro"
        WlrLayershell.layer: WlrLayer.Overlay
        WlrLayershell.keyboardFocus: WlrKeyboardFocus.None

        Rectangle {
            anchors.fill: parent
            radius: Theme.radiusCard
            color: Theme.fondo
            border.width: Theme.borderWidth
            border.color: root.phase === "break" ? Theme.acentoVerde : Theme.acentoRojo

            Column {
                anchors.fill: parent
                anchors.margins: 4
                spacing: 2

                Text {
                    width: parent.width
                    text: root.phase === "break" ? "☕ Descanso" : (root.paused ? "⏸ En pausa" : "Concentración")
                    color: root.phase === "break" ? Theme.acentoVerde : Theme.acentoRojo
                    font.pixelSize: 12
                    font.weight: Font.Bold
                    horizontalAlignment: Text.AlignHCenter
                }

                Text {
                    width: parent.width
                    text: root.mmss(root.remaining)
                    color: Theme.texto
                    font.pixelSize: 18
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
                    spacing: 4

                    Button {
                        text: root.paused ? "▶" : "❚❚"
                        onClicked: root.paused = !root.paused
                    }
                    Button {
                        text: "↻"
                        onClicked: root.restart()
                    }
                    Button {
                        text: "✕"
                        onClicked: Qt.quit()
                    }
                }
            }
        }
    }

    Process { id: soundProc; command: ["md", "pomodoro", "sound"] }
    Process { id: notifyProc }
}