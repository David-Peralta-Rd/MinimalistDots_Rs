# docs/adr/0003-ui-quickshell-layer-shell.md
# ADR-0003: UI con QuickShell (layer-shell)

La UI ahora es QuickShell (QML) usando layer-shell nativo. Slint se eliminó por completo.

- El posicionamiento y anclaje se definen en los QML (`anchors`, `margins`, `WlrLayershell`).
- No se usan windowrules de Hyprland para ventanas `md.*`.
- El tema visual vive en `dotfiles/quickshell/Theme.qml` y `Palette.qml`.