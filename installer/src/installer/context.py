"""Entorno que reciben los pasos: rutas, idioma y modo dry-run."""

import subprocess
import tomllib
from dataclasses import dataclass
from pathlib import Path

from installer.paths import Paths

REPO_ROOT = Path(__file__).resolve().parents[3]


@dataclass(frozen=True)
class Context:
    paths: Paths
    dry_run: bool = False
    language: str = "es"  # temporal: pasará a resolverse en language.py

    @classmethod
    def create(cls, dry_run: bool = False) -> "Context":
        return cls(paths=Paths(repo=REPO_ROOT, home=Path.home()), dry_run=dry_run)

    def t(self, key: str) -> str:
        """Texto traducido; si falta, devuelve la clave."""
        catalog = tomllib.loads((self.paths.lang / f"{self.language}.toml").read_text())
        section, _, name = key.partition(".")
        return str(catalog.get(section, {}).get(name, key))

    def run(self, command: list[str]) -> None:
        """Ejecuta un comando (lista de argumentos, nunca shell=True)."""
        print("$", " ".join(command))
        if not self.dry_run:
            subprocess.run(command, check=True)  # noqa: S603
