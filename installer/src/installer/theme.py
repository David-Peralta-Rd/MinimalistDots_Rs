"""Paleta de colores (`theme/palette.toml`) y generación de `colors.lua`."""

import tomllib
from pathlib import Path

# (clave en colors.lua, clave en palette.toml, alfa extra para Hyprland)
_COLORS: tuple[tuple[str, str, str | None], ...] = (
    ("background", "fondo", None),
    ("surface", "superficie", None),
    ("selection", "seleccion", None),
    ("border_inactive", "borde_inactivo", "aa"),
    ("border_active", "borde_activo", None),
    ("text", "texto", None),
    ("subtext", "texto_suave", None),
    ("accent_blue", "acento_azul", None),
    ("accent_green", "acento_verde", None),
    ("accent_red", "acento_rojo", None),
    ("accent_yellow", "acento_amarillo", None),
    ("accent_magenta", "acento_magenta", None),
    ("accent_cyan", "acento_cian", None),
    ("hover", "hover", None),
    ("card", "tarjeta", None),
    ("paused_text", "texto_pausado", None),
    ("contrast_light", "texto_contraste_claro", None),
    ("contrast_dark", "texto_contraste_oscuro", None),
    ("overlay", "overlay", None),
)


def load_palette(path: Path) -> dict[str, str]:
    """Carga la paleta (un diccionario `clave_toml -> "#RRGGBB"`)."""
    data = tomllib.loads(path.read_text(encoding="utf-8"))
    return {str(key): str(value) for key, value in data.items()}


def generate_colors_lua(palette: dict[str, str]) -> str:
    """Genera el contenido de `colors.lua` para Hyprland desde la paleta."""
    lines = [
        "-- ~/.config/hypr/hyprland/colors.lua",
        "-- Autogenerado desde theme/palette.toml. NO EDITES ESTE ARCHIVO A MANO.",
        "-- Para personalizar, usa el override en ~/.config/hypr/custom/colors.lua.",
        "",
        'local function rgba(raw_hex, alpha)',
        '    alpha = alpha or "ee"',
        '    return "rgba(" .. raw_hex .. alpha .. ")"',
        "end",
        "",
        "return {",
    ]
    for lua_key, toml_key, alpha in _COLORS:
        if toml_key not in palette:
            continue
        hex_color = palette[toml_key]
        raw = hex_color.lstrip("#")
        alpha_arg = f', "{alpha}"' if alpha else ""
        lines.append(
            f'    {lua_key:<16}= {{ hex = "{hex_color}", '
            f'hypr = rgba("{raw}"{alpha_arg}) }},'
        )
    lines.append("}")
    return "\n".join(lines) + "\n"


def _raw(palette: dict[str, str], key: str, default: str = "#000000") -> str:
    return palette.get(key, default).lstrip("#")


def generate_hyprlock_conf(palette: dict[str, str]) -> str:
    fondo = _raw(palette, "fondo", "#1e222a")
    texto = _raw(palette, "texto", "#abb2bf")
    superficie = _raw(palette, "superficie", "#282c34")
    borde = _raw(palette, "borde_inactivo", "#4b5263")
    acento_azul = _raw(palette, "acento_azul", "#7aa2f7")
    acento_rojo = _raw(palette, "acento_rojo", "#e06c75")
    return f'''general {{
    disable_loading_bar = true
    hide_cursor = true
    grace = 0
    no_fade_in = false
}}
background {{
    monitor =
    path = screenshot
    blur_passes = 4
    blur_size = 7
    noise = 0.015
    contrast = 0.8916
    brightness = 0.70
    color = rgba({fondo}cc)
}}
label {{
    monitor =
    text = cmd[update:1000] echo "$(date +"%H:%M")"
    color = rgba({texto}ff)
    font_size = 80
    font_family = JetBrains Mono Nerd Font Bold
    position = 0, 180
    halign = center
    valign = center
}}
label {{
    monitor =
    text = Hola, $USER
    color = rgba({texto}cc)
    font_size = 14
    font_family = JetBrains Mono Nerd Font Regular
    position = 0, -40
    halign = center
    valign = center
}}
input-field {{
    monitor =
    size = 250, 45
    outline_thickness = 2
    dots_size = 0.22
    dots_spacing = 0.35
    dots_center = true
    outer_color = rgba({borde}aa)
    check_color = rgba({acento_azul}ff)
    fail_color = rgba({acento_rojo}ff)
    inner_color = rgba({superficie}ff)
    font_color = rgba({texto}ff)
    fade_on_empty = true
    fade_timeout = 1500
    placeholder_text = <i>Ingresa contraseña / Enter password...</i>
    hide_input = false
    position = 0, -110
    halign = center
    valign = center
}}
'''


def generate_foot_ini(palette: dict[str, str]) -> str:
    fondo = palette.get("fondo", "#1e222a").lstrip("#")
    texto = palette.get("texto", "#abb2bf").lstrip("#")
    seleccion = palette.get("seleccion", "#3e4451").lstrip("#")
    subtext = palette.get("texto_suave", "#5c6370").lstrip("#")
    azul = palette.get("acento_azul", "#7aa2f7").lstrip("#")
    verde = palette.get("acento_verde", "#7ec7a2").lstrip("#")
    rojo = palette.get("acento_rojo", "#e06c75").lstrip("#")
    amarillo = palette.get("acento_amarillo", "#d19a66").lstrip("#")
    magenta = palette.get("acento_magenta", "#c678dd").lstrip("#")
    cian = palette.get("acento_cian", "#56b6c2").lstrip("#")
    return f'''[main]
shell=/bin/zsh
font=JetBrains Mono:size=6.8
pad=12x12

[csd]
preferred=none

[colors-dark]
background={fondo}
foreground={texto}
selection-background={seleccion}
selection-foreground={texto}

regular0={fondo}  # Negro (Fondo)
regular1={rojo}  # Rojo
regular2={verde}  # Verde
regular3={amarillo}  # Amarillo
regular4={azul}  # Azul
regular5={magenta}  # Magenta
regular6={cian}  # Cian
regular7={texto}  # Blanco

bright0={subtext}   # Gris
bright1={rojo}   # Rojo
bright2={verde}   # Verde
bright3={amarillo}   # Amarillo
bright4={azul}   # Azul
bright5={magenta}   # Magenta
bright6={cian}   # Cian
bright7={texto}   # Blanco brillante
'''


def generate_quickshell_palette(palette: dict[str, str]) -> str:
    def c(key: str) -> str:
        return palette.get(key, "#000000")

    return f'''// Generado desde theme/palette.toml. NO EDITES A MANO.
import QtQuick

QtObject {{
    readonly property color fondo: "{c("fondo")}"
    readonly property color superficie: "{c("superficie")}"
    readonly property color seleccion: "{c("seleccion")}"
    readonly property color bordeInactivo: "{c("borde_inactivo")}"
    readonly property color bordeActivo: "{c("borde_activo")}"
    readonly property color texto: "{c("texto")}"
    readonly property color textoSuave: "{c("texto_suave")}"
    readonly property color acentoAzul: "{c("acento_azul")}"
    readonly property color acentoVerde: "{c("acento_verde")}"
    readonly property color acentoRojo: "{c("acento_rojo")}"
    readonly property color acentoAmarillo: "{c("acento_amarillo")}"
    readonly property color acentoMagenta: "{c("acento_magenta")}"
    readonly property color acentoCian: "{c("acento_cian")}"
    readonly property color hover: "{c("hover")}"
    readonly property color tarjeta: "{c("tarjeta")}"
    readonly property color textoPausado: "{c("texto_pausado")}"
    readonly property color contrasteClaro: "{c("texto_contraste_claro")}"
    readonly property color contrasteOscuro: "{c("texto_contraste_oscuro")}"
    readonly property color overlay: "{c("overlay")}"
}}
'''
