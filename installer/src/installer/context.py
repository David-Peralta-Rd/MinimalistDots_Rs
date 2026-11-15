"""Entorno que reciben los pasos: rutas, idioma y modo dry-run."""

import subprocess
from dataclasses import dataclass
from functools import cached_property
from pathlib import Path

from installer.language import Translator, load_translator, resolve_language
from installer.paths import Paths

REPO_ROOT = Path(__file__).resolve().parents[3]


@dataclass(frozen=True)
class Context:
    paths: Paths
    dry_run: bool = False
    language: str = "es"

    @classmethod
    def create(cls, dry_run: bool = False, *, interactive: bool = True) -> "Context":
        paths = Paths(repo=REPO_ROOT, home=Path.home())
        chosen = resolve_language(paths.lang, paths.settings, interactive=interactive)
        return cls(paths=paths, dry_run=dry_run, language=chosen)

    @cached_property
    def translator(self) -> Translator:
        return load_translator(self.paths.lang, self.language)

    def t(self, key: str) -> str:
        """Texto traducido; si falta en toda la cadena, devuelve la clave."""
        return self.translator.t(key)

    def run(self, command: list[str], *, cwd: Path | None = None) -> None:
        """Ejecuta un comando (lista de argumentos, nunca shell=True)."""
        print("$", " ".join(command))
        if not self.dry_run:
            subprocess.run(command, check=True, cwd=cwd)  # noqa: S603

    def run_ok(self, command: list[str], *, cwd: Path | None = None) -> bool:
        """Ejecuta un comando devolviendo `True` si tuvo éxito (sin lanzar excepción)."""
        print("$", " ".join(command))
        if self.dry_run:
            return True
        return subprocess.run(command, cwd=cwd).returncode == 0  # noqa: S603

    def write(self, path: Path, content: str) -> None:
        """Escribe `content` en un archivo del usuario, respetando dry-run."""
        print(f"  + {path}")
        if not self.dry_run:
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_text(content, encoding="utf-8")

    def capture(self, command: list[str]) -> str:
        """Ejecuta un comando de solo lectura y devuelve su salida estándar."""
        print("$", " ".join(command))
        if self.dry_run:
            return ""
        return subprocess.run(  # noqa: S603
            command, capture_output=True, text=True, check=True
        ).stdout

    def write_sudo(self, path: str, content: str) -> None:
        """Escribe `content` en un archivo del sistema con sudo, respetando dry-run."""
        print(f"$ sudo tee {path}")
        if not self.dry_run:
            command = ["sudo", "tee", path]
            subprocess.run(  # noqa: S603
                command,
                input=content.encode(),
                check=True,
                stdout=subprocess.DEVNULL,
            )
