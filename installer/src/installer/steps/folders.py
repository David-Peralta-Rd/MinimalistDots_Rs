"""Paso 1: creación de carpetas (c1_new_folders.sh)."""

from installer.context import Context

ORDER = 20


def run(ctx: Context) -> None:
    print(ctx.t("carpetas.nuevas_carpetas"))

    directories = [
        ctx.paths.hypr / "hyprland",
        ctx.paths.hypr / "custom",
        ctx.paths.services,
        ctx.paths.cache_hypr,
        ctx.paths.models,
    ]

    for directory in directories:
        print(f"  {ctx.t('carpetas.nombre_de_carpeta')} {directory}")
        if not ctx.dry_run:
            directory.mkdir(parents=True, exist_ok=True)
