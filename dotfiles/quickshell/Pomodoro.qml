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
    property string breakLabel: Quickshell.env("MD_POMODORO_BREAK_LABEL") || "☕  Hora de descansar"
    property string pausedLabel: Quickshell.env("MD_POMODORO_PAUSED_LABEL") || "⏸  En pausa"
    property string pauseLabel: Quickshell.env("MD_POMODORO_PAUSE_LABEL") || "❚❚ Pausar"
    property string resumeLabel: Quickshell.env("MD_POMODORO_RESUME_LABEL") || "▶ Reanudar"
    property string restartLabel: Quickshell.env("MD_POMODORO_RESTART_LABEL") || "↻ Reiniciar"
    property string closeLabel: Quickshell.env("MD_POMODORO_CLOSE_LABEL") || "✕ Detener"
    property string minimizeLabel: Quickshell.env("MD_POMODORO_MINIMIZE_LABEL") || "▾ Minimizar"
    property string tipLabel: Quickshell.env("MD_POMODORO_TIP_LABEL") || "Levántate, estírate y mira a lo lejos."
    property string completedLabel: Quickshell.env("MD_POMODORO_COMPLETED_LABEL") || "Pomodoro completado"

    property int currentSession: 1
    property int remaining: focusSec
    property string phase: "focus" // focus | break
    property bool paused: false
    property bool finished: false
    property bool collapsed: false
    property bool workHovered: false

    property int pct: phase === "focus"
        ? Math.round((remaining / focusSec) * 100)
        : Math.round((remaining / breakSec) * 100)

    onPhaseChanged: collapsed = false

    Shortcut {
        sequence: "Escape"
        onActivated: { stopProc.running = true }
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
            notifyProc.command = ["notify-send", "Pomodoro", root.breakLabel]
            notifyProc.running = true
        } else {
            currentSession++
            if (currentSession > sessions) {
                finished = true
                notifyProc.command = ["notify-send", "Pomodoro", root.completedLabel]
                notifyProc.running = true
                Qt.quit()
                return
            }
            phase = "focus"
            remaining = focusSec
            notifyProc.command = ["notify-send", "Pomodoro", root.focusLabel]
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
            opacity: root.workHovered ? 1.0 : 0.3
            radius: Theme.radiusCard
            border.width: Theme.borderWidth
            border.color: Theme.bordeInactivo

            Row {
                anchors.fill: parent
                anchors.margins: 4
                spacing: 4

                Column {
                    width: parent.width - 22
                    spacing: 4

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
                        color: Theme.texto
                        font.pixelSize: 10
                        font.weight: Font.Bold
                        horizontalAlignment: Text.AlignHCenter
                    }

                    Rectangle {
                        width: parent.width * 0.6
                        height: 1
                        radius: 1
                        color: Theme.seleccion
                        anchors.horizontalCenter: parent.horizontalCenter

                        Rectangle {
                            width: parent.width * root.pct / 100
                            height: parent.height
                            radius: 1
                            color: Theme.acentoRojo
                        }
                    }
                }

                Column {
                    width: 18
                    spacing: 2
                    anchors.verticalCenter: parent.verticalCenter

                    SoftButton {
                        width: 18
                        height: 14
                        softRadius: 3
                        softColor: pa.containsMouse ? Theme.seleccion : "transparent"

                        Text {
                            anchors.centerIn: parent
                            text: root.paused ? "▶" : "❚❚"
                            color: Theme.texto
                            font.pixelSize: 5
                        }

                        MouseArea {
                            id: pa
                            anchors.fill: parent
                            hoverEnabled: true
                            onClicked: root.paused = !root.paused
                        }
                    }

                    SoftButton {
                        width: 18
                        height: 14
                        softRadius: 3
                        softColor: ra.containsMouse ? Theme.seleccion : "transparent"

                        Text {
                            anchors.centerIn: parent
                            text: "↻"
                            color: Theme.texto
                            font.pixelSize: 5
                        }

                        MouseArea {
                            id: ra
                            anchors.fill: parent
                            hoverEnabled: true
                            onClicked: root.restart()
                        }
                    }

                    SoftButton {
                        width: 18
                        height: 14
                        softRadius: 3
                        softColor: sa.containsMouse ? Theme.seleccion : "transparent"

                        Text {
                            anchors.centerIn: parent
                            text: "✕"
                            color: Theme.texto
                            font.pixelSize: 5
                        }

                        MouseArea {
                            id: sa
                            anchors.fill: parent
                            hoverEnabled: true
                            onClicked: { stopProc.running = true }
                        }
                    }
                }
            }

            HoverHandler {
                onHoveredChanged: root.workHovered = hovered
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
                    text: root.paused ? root.pausedLabel : root.breakLabel
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
                    text: root.tipLabel
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
                        color: bpa.containsMouse ? Theme.hover : Theme.superficie

                        Text {
                            anchors.centerIn: parent
                            text: root.paused ? root.resumeLabel : root.pauseLabel
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
                        color: bra.containsMouse ? Theme.hover : Theme.superficie

                        Text {
                            anchors.centerIn: parent
                            text: root.restartLabel
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
                        color: bca.containsMouse ? Theme.hover : Theme.superficie

                        Text {
                            anchors.centerIn: parent
                            text: root.minimizeLabel
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
                        color: bsa.containsMouse ? Theme.hover : Theme.superficie

                        Text {
                            anchors.centerIn: parent
                            text: root.closeLabel
                            color: Theme.texto
                            font.pixelSize: 12
                        }

                        MouseArea {
                            id: bsa
                            anchors.fill: parent
                            hoverEnabled: true
                            onClicked: { stopProc.running = true }
                        }
                    }
                }
            }
        }
    }

    Process { id: soundProc; command: ["md", "pomodoro", "sound"] }
    Process { id: notifyProc }
    Process {
        id: stopProc
        command: ["md", "pomodoro", "stop"]
        onExited: Qt.quit()
    }
}