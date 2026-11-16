local Rules = require("hyprland.lib.rules")

-- Ventanas Slint de MinimalistDots (app-id "md.<nombre>"): flotantes, centradas,
-- pinadas (visibles en todos los workspaces) y sin borde ni animación, para que
-- parezcan popups. `pin` solo afecta a ventanas flotantes, por eso va con `float`.
Rules.forClass("^md%..*$"):custom({
    float       = true,
    pin         = true,
    border_size = 0,
    rounding    = 12,
    blur        = true,
    no_anim     = true,
})

-- Los menús interactivos conservan el foco de teclado mientras están visibles.
-- El OSD (`md volume`) y el popup de notificaciones (`md.notifications`) NO deben
-- robar foco: heredan la regla `md.*` de arriba, que no lleva `stay_focused`.
-- Ambos se muestran centrados por ahora; anclarlos (arriba-centro el OSD,
-- arriba-derecha las notificaciones) se afina en el spike de H8 con `move`
-- (requiere verificación real en sesión Wayland).
Rules.forClass("^(md%.launcher|md%.menu%..*|md%.binds|md%.clip|md%.wallpaper|md%.process|md%.notify|md%.pomodoro%.ask|md%.ai%.ask|md%.language)$"):custom({
    stay_focused = true,
    center       = true,
})

-- El reloj (`md clock`) va a la izquierda, centrado en vertical y 40px más abajo
-- (como el eww/QML original). El `move` es una aproximación: su anclado exacto y
-- el comportamiento "detrás de las ventanas" (que exigiría layer-shell) se
-- verifican en H8.




-- Anclaje explícito (la lib `Rules` no aplica `move` de forma fiable a
-- ventanas Wayland/xfg-toplevel; se usa la API directa como hyprland-run).
hl.window_rule({
    name  = "move-md-clock",
    match = { class = "md.clock" },
    float = true,
    pin   = true,
    border_size = 0,
    move  = "20 monitor_h/2+20",
    no_focus = true,
})
hl.window_rule({
    name  = "move-md-notifications",
    match = { class = "md.notifications" },
    float = true,
    pin   = true,
    border_size = 0,
    move  = "monitor_w-380 20",
    no_focus = true,
})

Rules.forClass(".*"):noBlur()
Rules.forClass("^(pavucontrol)$"):float()
Rules.forClass("^(blueman%-manager)$"):float()

Rules.forClass("hyprland-run"):custom({
    name  = "move-hyprland-run",
    move  = "20 monitor_h-120",
    float = true,
})

-- Reglas para imv y mpv (imágenes y videos)
Rules.forClass("imv"):custom({ float = true, size = "70% 70%", center = true })
Rules.forClass("mpv"):custom({ float = true, size = "60% 60%", center = true })

-- Reglas para terminal flotante
Rules.forClass("programing-terminal"):custom({
    name = "move-programing-terminal",
    size = "monitor_w/4.1 monitor_h/7.9",
    move = "monitor_w/1.33 monitor_h/1.16",
    float = true,
})

-- Reglas para terminal flotante de open code
Rules.forClass("programing-terminal-opencode"):custom({
    name = "move-programing-terminal-opencode",
    size = "monitor_w/3.45 monitor_h/2.52",
    move = "monitor_w*0.004 monitor_h/1.69",
    float = true,
})
