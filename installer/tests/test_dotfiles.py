from pathlib import Path

import pytest
from installer.context import Context
from installer.errors import CancelledError
from installer.paths import Paths
from installer.steps import dotfiles

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


def _make_repo(tmp_path: Path) -> Paths:
    repo = tmp_path / "repo"
    home = tmp_path / "home"
    (repo / "dotfiles" / "config" / "hypr" / "hyprland").mkdir(parents=True)
    (repo / "dotfiles" / "config" / "hypr" / "hyprland" / "general.lua").write_text(
        "-- general\n"
    )
    (repo / "theme").mkdir()
    (repo / "theme" / "palette.toml").write_text(PALETTE)

    # Configuración previa del usuario (debe respaldarse y restaurarse custom/).
    (home / ".config" / "hypr" / "custom").mkdir(parents=True)
    (home / ".config" / "hypr" / "custom" / "keep.lua").write_text("-- del usuario\n")

    return Paths(repo=repo, home=home)


def test_discover_apps_lists_config_entries(tmp_path: Path) -> None:
    paths = _make_repo(tmp_path)
    assert dotfiles._discover_apps(paths.dots_config) == ["hypr"]


def test_run_deploys_backs_up_and_restores(tmp_path: Path, monkeypatch: pytest.MonkeyPatch) -> None:
    paths = _make_repo(tmp_path)
    monkeypatch.setattr("builtins.input", lambda _prompt="": "")

    dotfiles.run(Context(paths=paths, dry_run=False, language="es"))

    # Desplegado desde el proyecto.
    assert (paths.config / "hypr" / "hyprland" / "general.lua").exists()
    # colors.lua regenerado desde la paleta.
    colors = paths.config / "hypr" / "hyprland" / "colors.lua"
    assert colors.exists()
    assert 'hex = "#1e222a"' in colors.read_text()
    # custom/ del usuario restaurado tras el despliegue.
    assert (paths.config / "hypr" / "custom" / "keep.lua").exists()
    # Backup con timestamp de la config previa.
    backups = list((paths.backups / "hypr-backups").iterdir())
    assert len(backups) == 1
    assert (backups[0] / "custom" / "keep.lua").exists()


def test_run_cancels_when_user_rejects(tmp_path: Path, monkeypatch: pytest.MonkeyPatch) -> None:
    paths = _make_repo(tmp_path)
    monkeypatch.setattr("builtins.input", lambda _prompt="": "n")

    with pytest.raises(CancelledError):
        dotfiles.run(Context(paths=paths, dry_run=False, language="es"))

    # Nada desplegado al cancelar.
    assert not (paths.config / "hypr" / "hyprland").exists()


def test_run_dry_run_does_not_touch_disk(tmp_path: Path) -> None:
    paths = _make_repo(tmp_path)

    dotfiles.run(Context(paths=paths, dry_run=True, language="es"))

    # En dry-run no se despliega nada nuevo ni se toca la config previa.
    assert not (paths.config / "hypr" / "hyprland").exists()
