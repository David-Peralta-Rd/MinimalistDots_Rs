require("load_direction")


local Service = require("hyprland.lib.services")

-- Reloj de escritorio (ventana Slint de `md`). No hace falta daemon: es un
-- proceso ligero que se cierra con la sesión.
local MD = os.getenv("HOME") .. "/.local/bin/md"

return Service.define("desktop_clock", function()
    hl.on("hyprland.start", function()
        hl.exec_cmd(MD .. " clock")
        -- Baja el reloj por debajo de las demás ventanas (best-effort).
        hl.exec_cmd("sh -c 'sleep 2; addr=$(hyprctl clients -j | (command -v jq >/dev/null && jq -r \".[] | select(.class==\\\"md.clock\\\") | .address\" | head -n1 || true)); [ -n \"$addr\" ] && hyprctl dispatch alterzorder bottom,address:$addr >/dev/null 2>&1 || true'")
    end)
end)
