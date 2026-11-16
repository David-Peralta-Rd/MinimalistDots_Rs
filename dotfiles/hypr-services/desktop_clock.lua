require("load_direction")


local Service = require("hyprland.lib.services")

-- Reloj de escritorio (ventana Slint de `md`). La posición y el anclaje se
-- resuelven en las windowrules; bajar al fondo lo hace el propio binario.
local MD = os.getenv("HOME") .. "/.local/bin/md"

return Service.define("desktop_clock", function()
    hl.on("hyprland.start", function()
        hl.exec_cmd(MD .. " clock")
    end)
end)
