"""Paso: compilación e instalación del binario `md` (Rust)."""

import shutil
import subprocess
from pathlib import Path

from installer.context import Context

ORDER = 49


def run(ctx: Context) -> None:
    repo = ctx.paths.repo
    if not (repo / "Cargo.toml").is_file():
        print("  sin código Rust en el repo; se omite.")
        return

    if shutil.which("cargo") is None:
        print("  cargo no disponible; instala Rust (rustup) y compila manualmente.")
        return

    dest = ctx.paths.home / ".local" / "bin" / "md"
    src = repo / "target" / "release" / "md"

    # Recompila si el binario no existe o el código es más reciente.
    needs_build = not dest.is_file() or not src.is_file()
    if not needs_build:
        try:
            src_mtime = src.stat().st_mtime
            newest = max(
                (p.stat().st_mtime for p in repo.rglob("*.rs") if p.is_file()),
                default=0.0,
            )
            needs_build = newest > src_mtime
        except OSError:
            needs_build = True

    if ctx.dry_run:
        print(f"  + cargo build --release -p md -> {dest}")
        return

    if needs_build:
        print("  compilando `md` (cargo build --release -p md)...")
        subprocess.run(
            ["cargo", "build", "--release", "-p", "md"],
            cwd=repo,
            check=True,
        )

    if src.is_file():
        dest.parent.mkdir(parents=True, exist_ok=True)
        shutil.copy2(src, dest)
        print(f"  `md` instalado en {dest}")
    else:
        print("  aviso: no se generó el binario `md`.")
