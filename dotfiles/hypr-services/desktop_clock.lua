require("load_direction")


local Service = require("hyprland.lib.services")

-- Reloj de escritorio (ventana Slint de `md`). No hace falta daemon: es un
-- proceso ligero que se cierra con la sesión.
local MD = os.getenv("HOME") .. "/.local/bin/md"

return Service.define("desktop_clock", function()
    hl.on("hyprland.start", function()
        hl.exec_cmd(MD .. " clock")
    end)
end)
