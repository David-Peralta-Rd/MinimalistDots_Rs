"""Todas las rutas del proyecto y del sistema del usuario, con nombre."""

from dataclasses import dataclass
from pathlib import Path


@dataclass(frozen=True)
class Paths:
    repo: Path  # raíz del repositorio
    home: Path  # HOME del usuario (en pruebas: una carpeta temporal)

    # --- Dentro del repositorio ---
    @property
    def dots_config(self) -> Path:
        return self.repo / "dotfiles" / "config"

    @property
    def lang(self) -> Path:
        return self.repo / "lang"

    @property
    def palette(self) -> Path:
        return self.repo / "theme" / "palette.toml"

    @property
    def packages(self) -> Path:
        return self.repo / "packages.toml"

    # --- En el sistema del usuario ---
    @property
    def config(self) -> Path:
        return self.home / ".config"

    @property
    def hypr(self) -> Path:
        return self.config / "hypr"

    @property
    def backups(self) -> Path:
        return self.config / "backups_dots"

    @property
    def settings(self) -> Path:
        return self.config / "minimalistdots" / "settings.toml"
