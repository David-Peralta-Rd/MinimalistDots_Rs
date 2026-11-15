"""Paso 3: backup e instalación de configuraciones (c3_backup_and_install.sh)."""

import shutil
from datetime import datetime
from pathlib import Path

from installer import theme
from installer.context import Context
from installer.errors import CancelledError, StepError

ORDER = 40


def _discover_apps(dots_config: Path) -> list[str]:
    """Nombres de carpetas y archivos dentro de `dotfiles/config` (sin ocultos)."""
    return sorted(
        item.name for item in dots_config.iterdir() if not item.name.startswith(".")
    )


def _confirm(ctx: Context, apps: list[str]) -> None:
    print(f"📦 {ctx.t('dotfiles.encabezado')} ({len(apps)}: {', '.join(apps)})")
    if ctx.dry_run:
        return
    yes = ctx.t("dotfiles.letra_confirmacion").lower()
    answer = input(
        f"{ctx.t('dotfiles.confirmacion')} ({ctx.t('dotfiles.letra_confirmacion')}/n): "
    )
    if answer.strip().lower() not in ("", yes):
        print(ctx.t("dotfiles.cancelado"))
        raise CancelledError()


def _deploy(ctx: Context, app: str, src: Path, dest: Path, backup_root: Path) -> None:
    if dest.exists() or dest.is_symlink():
        print(f"{ctx.t('dotfiles.backup_si_existe')} {app} -> {backup_root}")
        if not ctx.dry_run:
            backup_root.mkdir(parents=True, exist_ok=True)
            if dest.is_dir():
                shutil.copytree(dest, backup_root, dirs_exist_ok=True)
            else:
                shutil.copy2(dest, backup_root)

        print(f"{ctx.t('dotfiles.limpiando_destino')} {dest}")
        if not ctx.dry_run:
            if dest.is_dir():
                shutil.rmtree(dest)
            else:
                dest.unlink()
    else:
        print(f"{ctx.t('dotfiles.paquete_detectado')} {app}")

    print(f"{ctx.t('dotfiles.copiando_paquete')} {src} -> {dest}")
    if not ctx.dry_run:
        if src.is_dir():
            shutil.copytree(src, dest)
        else:
            dest.parent.mkdir(parents=True, exist_ok=True)
            shutil.copy2(src, dest)


def _restore_custom(ctx: Context, hypr_dir: Path, backup_root: Path) -> None:
    custom_backup = backup_root / "custom"
    if not custom_backup.is_dir():
        return
    print(f"{ctx.t('dotfiles.custom_restaurado')} {hypr_dir / 'custom'}")
    if not ctx.dry_run:
        shutil.copytree(custom_backup, hypr_dir / "custom")


def _regen_colors(ctx: Context) -> None:
    """Regenera `colors.lua` desde `theme/palette.toml` (sustituye a bk5_colors_lua.sh)."""
    print(ctx.t("dotfiles.configurando_especificos"))
    palette = theme.load_palette(ctx.paths.palette)
    target = ctx.paths.hypr / "hyprland" / "colors.lua"
    if not ctx.dry_run:
        target.parent.mkdir(parents=True, exist_ok=True)
        target.write_text(theme.generate_colors_lua(palette), encoding="utf-8")


def run(ctx: Context) -> None:
    dots_config = ctx.paths.dots_config
    if not dots_config.is_dir():
        raise StepError(f"{ctx.t('comun.configuracion_no_encontrada')} {dots_config}")

    apps = _discover_apps(dots_config)
    if not apps:
        print(f"{ctx.t('comun.configuracion_no_encontrada')} {dots_config}")
        return

    _confirm(ctx, apps)

    timestamp = datetime.now().strftime("%Y%m%d_%H%M%S")
    for app in apps:
        src = dots_config / app
        dest = ctx.paths.config / app
        backup_root = ctx.paths.backups / f"{app}-backups" / timestamp
        _deploy(ctx, app, src, dest, backup_root)
        if app == "hypr":
            _restore_custom(ctx, dest, backup_root)
            _regen_colors(ctx)
