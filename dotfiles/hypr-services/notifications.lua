require("load_direction")


local Service = require("hyprland.lib.services")

-- Servidor de notificaciones (`md notify serve`), reemplaza a SwayNC/Quickshell.
-- Debe haber un solo servidor en el bus: si aún tienes swaync instalado, desinstálalo.
local MD = os.getenv("HOME") .. "/.local/bin/md"

return Service.define("notifications", function()
    hl.on("hyprland.start", function()
        hl.exec_cmd(MD .. " notify serve")
    end)
end)
