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
        hex_color = palette[toml_key]
        raw = hex_color.lstrip("#")
        alpha_arg = f', "{alpha}"' if alpha else ""
        lines.append(
            f'    {lua_key:<16}= {{ hex = "{hex_color}", '
            f'hypr = rgba("{raw}"{alpha_arg}) }},'
        )
    lines.append("}")
    return "\n".join(lines) + "\n"
