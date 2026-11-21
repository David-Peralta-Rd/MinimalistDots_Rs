import QtQuick
import QtQuick.Controls
import Quickshell
import Quickshell.Io
import Quickshell.Wayland
import "."

ShellRoot {
    id: root

    property string title: Quickshell.env("MD_WELCOME_TITLE") || "MinimalistDots_Rs"
    property string subtitle: Quickshell.env("MD_WELCOME_SUBTITLE") || ""
    property string labelChangelog: Quickshell.env("MD_WELCOME_CHANGELOG") || ""
    property string labelLanguage: Quickshell.env("MD_WELCOME_LANGUAGE") || "Idioma"
    property string labelBinds: Quickshell.env("MD_WELCOME_BINDS") || "Atajos"
    property string labelGithub: Quickshell.env("MD_WELCOME_GITHUB") || "GitHub"
    property string labelToggle: Quickshell.env("MD_WELCOME_TOGGLE") || "Bienvenida"
    property string labelExit: Quickshell.env("MD_WELCOME_EXIT") || "Salir"
    property string githubUrl: Quickshell.env("MD_WELCOME_GITHUB_URL") || ""
    property bool bienvenidaOn: (Quickshell.env("MD_WELCOME_BIENVENIDA") || "true") === "true"
    property var changelog: (labelChangelog || "").split("\n")

    Shortcut {
        sequence: "Escape"
        onActivated: Qt.quit()
    }

    PanelWindow {
        implicitWidth: 760
        implicitHeight: 600
        color: "transparent"

        anchors.top: true
        anchors.left: true
        margins.top: (screen ? Math.round((screen.height - implicitHeight) / 2) : 0)
        margins.left: (screen ? Math.round((screen.width - implicitWidth) / 2) : 0)

        WlrLayershell.namespace: "md.welcome"
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
                anchors.margins: 24
                spacing: 16

                Text {
                    width: parent.width
                    text: root.title
                    color: Theme.acentoVerde
                    font.pixelSize: 32
                    font.weight: Font.ExtraBold
                    horizontalAlignment: Text.AlignHCenter
                }

                Text {
                    width: parent.width
                    text: root.subtitle
                    color: Theme.texto
                    font.pixelSize: 14
                    horizontalAlignment: Text.AlignHCenter
                }

                Text {
                    text: "📦 Novedades"
                    color: Theme.acentoAzul
                    font.pixelSize: 16
                    font.weight: Font.Bold
                }

                ScrollView {
                    width: parent.width
                    height: 260
                    clip: true

                    Column {
                        width: parent.width
                        spacing: 6
                        Repeater {
                            model: root.changelog
                            Text {
                                width: parent.width
                                text: modelData
                                color: Theme.texto
                                font.pixelSize: 13
                                wrapMode: Text.Wrap
                            }
                        }
                    }
                }

                Row {
                    spacing: 12
                    anchors.horizontalCenter: parent.horizontalCenter

                    Button {
                        text: "🌐 " + root.labelLanguage
                        onClicked: langProc.running = true
                    }
                    Button {
                        text: "⌨️ " + root.labelBinds
                        onClicked: bindsProc.running = true
                    }
                    Button {
                        text: "🐙 " + root.labelGithub
                        onClicked: githubProc.running = true
                    }
                    Button {
                        text: root.bienvenidaOn ? root.labelToggle : root.labelToggle
                        onClicked: toggleProc.running = true
                    }
                    Button {
                        text: "🚪 " + root.labelExit
                        onClicked: Qt.quit()
                    }
                }
            }
        }
    }

    Process {
        id: langProc
        command: ["md", "language"]
    }
    Process {
        id: bindsProc
        command: ["md", "binds"]
    }
    Process {
        id: githubProc
        command: ["xdg-open", root.githubUrl]
    }
    Process {
        id: toggleProc
        command: ["md", "welcome", "--toggle"]
        onExited: Qt.quit()
    }
}