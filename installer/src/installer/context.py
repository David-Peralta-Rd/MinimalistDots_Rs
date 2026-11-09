"""Entorno que reciben los pasos: rutas, idioma y modo dry-run."""
import subprocess
import tomllib
from dataclasses import dataclass, field
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parents[3]


@dataclass(frozen=True)
class Context:
    dry_run: bool = False
    language: str = "es"
    repo: Path = REPO_ROOT
    home: Path = field(default_factory=Path.home)

    def t(self, key: str) -> str:
        """Texto traducido; si falta, devuelve la clave."""
        catalog = tomllib.loads((self.repo / "lang" / f"{self.language}.toml").read_text())
        section, _, name = key.partition(".")
        return str(catalog.get(section, {}).get(name, key))

    def run(self, command: list[str]) -> None:
        """Ejecuta un comando (lista de argumentos, nunca shell=True)."""
        print("$", " ".join(command))
        if not self.dry_run:
            subprocess.run(command, check=True)  # noqa: S603
