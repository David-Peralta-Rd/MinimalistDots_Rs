"""Paso 2: instalación de paquetes (c2_install_packages.sh)."""

import shutil
import tempfile
from pathlib import Path

from installer import packages
from installer.context import Context

ORDER = 30

AUR_PARU_BIN = "https://aur.archlinux.org/paru-bin.git"


def _ensure_paru(ctx: Context) -> None:
    if shutil.which("paru"):
        return
    # CachyOS (y otras derivadas) traen paru en sus repositorios.
    if ctx.run_ok(["sudo", "pacman", "-S", "--needed", "--noconfirm", "paru"]):
        return

    # Arch puro: paru solo existe en el AUR, se compila paru-bin.
    print(ctx.t("paquetes.instalando_paru"))
    ctx.run(["sudo", "pacman", "-S", "--needed", "--noconfirm", "base-devel", "git"])

    build_dir = Path(tempfile.mkdtemp(prefix="md-paru-"))
    try:
        ctx.run(["git", "clone", "--depth", "1", AUR_PARU_BIN, str(build_dir / "paru-bin")])
        ctx.run(["makepkg", "-si", "--noconfirm"], cwd=build_dir / "paru-bin")
    finally:
        shutil.rmtree(build_dir, ignore_errors=True)


def run(ctx: Context) -> None:
    print(ctx.t("paquetes.instalando_paquetes"))

    groups = packages.load_groups(ctx.paths.packages)
    pkgs = packages.all_packages(groups)

    ctx.run(["sudo", "pacman", "-Syu", "--noconfirm"])
    _ensure_paru(ctx)
    ctx.run(["paru", "-S", "--needed", "--noconfirm", *pkgs])
