---------------------
---- MY PROGRAMS ----
---------------------

local HOME = os.getenv("HOME")
local MD = HOME .. "/.local/bin/md"

return {
-- ======================== --
-- ==== Fast Execution ==== --
-- ======================== --
    terminal        = "footclient",
    terminal_float  = "footclient --app-id=programing-terminal",
    terminal_float_opencode  = "footclient --app-id=programing-terminal-opencode",
    fileManager     = "dolphin",
    menu            = MD .. " launcher",
    browser         = "brave",
    mainMod         = "SUPER",
    editor          = "code",
    hyprlock        = "loginctl lock-session",

-- =========================== --
-- ==== Complex Execution ==== --
-- =========================== --
    cliphist = MD .. " clip",

-- ========================== --
-- ==== Keybinds Scripts ==== --
-- ========================== --
    volume_up           = MD .. " volume up",
    volume_down         = MD .. " volume down",
    volume_mute         = MD .. " volume mute",
    screenshot          = MD .. " shot",
    select_wallpaper    = MD .. " wallpaper",
    show_binds          = MD .. " binds",
    screenrecord        = MD .. " record",
    process_manager     = MD .. " process",
    pomodoro            = MD .. " pomodoro toggle",
    llm_menu            = MD .. " ai toggle",
    language            = MD .. " language",
    notifications       = MD .. " notify history",
    notifications_dnd   = MD .. " notify dnd",
    notifications_close = MD .. " notify dismiss",
}
