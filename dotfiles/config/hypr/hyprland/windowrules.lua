local Rules = require("hyprland.lib.rules")

-- Ventanas Slint de MinimalistDots (app-id "md.<nombre>"): flotantes, centradas,
-- pinadas (visibles en todos los workspaces) y sin borde ni animación, para que
-- parezcan popups. `pin` solo afecta a ventanas flotantes, por eso va con `float`.
Rules.forClass("^md%..*$"):custom({
    float     = true,
    center    = true,
    pin       = true,
    no_border = true,
    no_anim   = true,
})

-- Los menús interactivos conservan el foco de teclado mientras están visibles.
-- El OSD (`md volume`) y las notificaciones (H6) NO deben robar foco: heredan la
-- regla `md.*` de arriba, que no lleva `stay_focused`. El OSD se muestra centrado
-- por ahora; anclarlo arriba-centrado se afina en el spike de H8 (posicionamiento
-- con `move` requiere verificación real en sesión Wayland).
Rules.forClass("^(md%.launcher|md%.menu%..*|md%.binds|md%.process)$"):custom({
    stay_focused = true,
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
