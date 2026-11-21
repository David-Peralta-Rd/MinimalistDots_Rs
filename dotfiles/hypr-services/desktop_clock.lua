require("load_direction")


local Service = require("hyprland.lib.services")

-- Reloj de escritorio (Quickshell): capa "bottom", queda detrás de todo y no
-- reserva espacio.
local QML = os.getenv("HOME") .. "/.local/bin/MinimalistDots_Rs/quickshell/clock.qml"

return Service.define("desktop_clock", function()
    hl.on("hyprland.start", function()
        hl.exec_cmd("qs -p " .. QML)
    end)
end)
