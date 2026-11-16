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

    @property
    def hypr_services(self) -> Path:
        return self.repo / "dotfiles" / "hypr-services"

    @property
    def ai_src(self) -> Path:
        return self.repo / "dotfiles" / "ai"

    @property
    def quickshell(self) -> Path:
        return self.repo / "dotfiles" / "quickshell"

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

    @property
    def lang_dest(self) -> Path:
        return self.config / "minimalistdots" / "lang"

    @property
    def services(self) -> Path:
        return self.home / ".local" / "bin" / "MinimalistDots_Rs" / "services"

    @property
    def quickshell_dest(self) -> Path:
        return self.home / ".local" / "bin" / "MinimalistDots_Rs" / "quickshell"

    @property
    def cache_hypr(self) -> Path:
        return self.home / ".cache" / "hypr"

    @property
    def models(self) -> Path:
        return self.home / "models"

    @property
    def ai_dir(self) -> Path:
        return self.home / ".local" / "bin" / "MinimalistDots_Rs" / "ai"

    @property
    def ai_conf(self) -> Path:
        return self.home / ".config" / "ai"
