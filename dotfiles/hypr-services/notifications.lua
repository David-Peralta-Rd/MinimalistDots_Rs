require("load_direction")


local Service = require("hyprland.lib.services")

-- Servidor de notificaciones (`md notify serve`), reemplaza a SwayNC/Quickshell.
-- Debe haber un solo servidor en el bus: si aún tienes swaync instalado, desinstálalo.
local MD = os.getenv("HOME") .. "/.local/bin/md"

return Service.define("notifications", function()
    hl.on("hyprland.start", function()
        hl.exec_cmd(MD .. " notify serve")
        -- Ancla arriba-derecha (best-effort).
        hl.exec_cmd("sh -c 'sleep 2; monw=$(hyprctl monitors -j 2>/dev/null | (command -v jq >/dev/null && jq -r \".[0].width\" || echo 1920)); addr=$(hyprctl clients -j 2>/dev/null | (command -v jq >/dev/null && jq -r \".[] | select(.class==\\\"md.notifications\\\") | .address\" | head -n1 || true)); [ -n \"$addr\" ] && hyprctl dispatch movewindowpixel exact $((monw-380)) 20,address:$addr >/dev/null 2>&1'")
    end)
end)
