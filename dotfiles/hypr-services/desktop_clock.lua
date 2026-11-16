require("load_direction")


local Service = require("hyprland.lib.services")

-- Reloj de escritorio (ventana Slint de `md`). Se baja al fondo y se ancla a la
-- izquierda, centrado en vertical (+40px), como el Quickshell original.
local MD = os.getenv("HOME") .. "/.local/bin/md"

return Service.define("desktop_clock", function()
    hl.on("hyprland.start", function()
        hl.exec_cmd(MD .. " clock")
        -- Ancla y baja (best-effort): las windowrules `move` no se aplican en
        -- este Hyprland, así que lo hacemos por hyprctl tras dar tiempo a crear.
        hl.exec_cmd("sh -c 'sleep 2; monh=$(hyprctl monitors -j 2>/dev/null | (command -v jq >/dev/null && jq -r \".[0].height\" || echo 1080)); addr=$(hyprctl clients -j 2>/dev/null | (command -v jq >/dev/null && jq -r \".[] | select(.class==\\\"md.clock\\\") | .address\" | head -n1 || true)); [ -n \"$addr\" ] && { hyprctl dispatch movewindowpixel exact 20 $(((monh-120)/2+40)),address:$addr >/dev/null 2>&1; hyprctl dispatch alterzorder bottom,address:$addr >/dev/null 2>&1; }'")
    end)
end)
