# docs/adr/0003-ui-slint-sin-layer-shell.md
# ADR-0003: Slint con ventanas normales en v1
Estado: propuesto · Fecha: 2026-11-09

## Contexto
Slint 1.18.1 no ofrece `wlr-layer-shell` oficial (Discusiones #1765, #6910). Terceros: spell-framework 1.0.6 (GPL-3.0-or-later, activo) y layer-shika (estancado desde ene-2026). Quickshell sí era layer-shell.

## Decisión
v1: cada menú/OSD/widget es una ventana xdg (toplevel) de Slint, posicionada con windowrules de Hyprland (float, center, pin, noborder, noanim, stayfocused) por app-id `md.<nombre>`. Los widgets viven tras un puerto `ShellHost` en md_ui para cambiar el anfitrión sin tocar los .slint.
Spike en H0: lanzador con foco de teclado, OSD sin robar foco, popup de notificación arriba a la derecha. Si falla, plan B = crate propio con smithay-client-toolkit 0.21.x, sin `unsafe` salvo ADR.

## Alternativas descartadas
- spell-framework: GPL-3.0-or-later contagia el binario; el repo es MIT.
- iced/egui/gtk4-layer-shell: cambian la herramienta que pediste.

## Consecuencias
OSD y notificaciones dependen de windowrules (frágil entre versiones de Hyprland). El filtro "<5 MB" del BACKLOG deja de aplicar al binario: se reinterpreta como "sin dependencias residentes".

## Reconsiderar
Cuando Slint publique layer-shell oficial, o si el spike falla.

Para investigar: wlr-layer-shell, xdg-toplevel, Hyprland windowrule (float, pin, stayfocused), raw-window-handle, slint platform backend
