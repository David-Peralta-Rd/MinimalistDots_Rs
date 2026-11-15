require("load_direction")


local Service = require("hyprland.lib.services")

-- Pantalla de bienvenida de MinimalistDots. Se puede desactivar desde el propio
-- menú (queda guardado en ~/.config/minimalistdots/settings.toml).
local MD = os.getenv("HOME") .. "/.local/bin/md"

return Service.define("welcome", function()
    hl.on("hyprland.start", function()
        hl.exec_cmd(MD .. " welcome")
    end)
end)
