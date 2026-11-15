from pathlib import Path

from installer.theme import generate_colors_lua, load_palette

PALETTE = """\
fondo = "#1e222a"
superficie = "#282c34"
seleccion = "#3e4451"
borde_inactivo = "#4b5263"
borde_activo = "#7ec7a2"
texto = "#abb2bf"
texto_suave = "#5c6370"
acento_azul = "#7aa2f7"
acento_verde = "#7ec7a2"
acento_rojo = "#e06c75"
acento_amarillo = "#d19a66"
acento_magenta = "#c678dd"
acento_cian = "#56b6c2"
"""


def test_load_palette_reads_all_colors(tmp_path: Path) -> None:
    path = tmp_path / "palette.toml"
    path.write_text(PALETTE)
    palette = load_palette(path)
    assert palette["fondo"] == "#1e222a"
    assert palette["texto_suave"] == "#5c6370"
    assert palette["acento_cian"] == "#56b6c2"


def test_generate_colors_lua_uses_palette_and_alpha(tmp_path: Path) -> None:
    path = tmp_path / "palette.toml"
    path.write_text(PALETTE)
    lua = generate_colors_lua(load_palette(path))
    assert 'hex = "#1e222a"' in lua
    assert 'hypr = rgba("1e222a")' in lua
    # border_inactive lleva alfa "aa"
    assert 'hypr = rgba("4b5263", "aa")' in lua
    # el resto usa el alfa por defecto "ee" (sin segundo argumento)
    assert 'hypr = rgba("7ec7a2")' in lua
    assert "accent_cyan" in lua
