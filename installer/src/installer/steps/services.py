"""Paso 4a: despliegue de los servicios Lua (c4_install_services_and_scripts.sh)."""

import shutil

from installer.context import Context
from installer.errors import StepError

ORDER = 50


def run(ctx: Context) -> None:
    src = ctx.paths.hypr_services
    dest = ctx.paths.services
    if not src.is_dir():
        raise StepError(f"{ctx.t('comun.configuracion_no_encontrada')} {src}")

    new_files = sorted(src.glob("*.lua"))
    if not new_files:
        raise StepError(f"{ctx.t('comun.configuracion_no_encontrada')} {src}")

    old_files = sorted(dest.glob("*.lua")) if dest.is_dir() else []
    if old_files:
        print(ctx.t("servicios.listando_servicios_antiguos"))
        for file in old_files:
            print(f"    {file.name}")

    print(ctx.t("servicios.borrando_servicios_antiguos"))
    for file in old_files:
        if not ctx.dry_run:
            file.unlink()

    print(ctx.t("servicios.copiando_servicios_nuevos"))
    for file in new_files:
        if not ctx.dry_run:
            dest.mkdir(parents=True, exist_ok=True)
            shutil.copy2(file, dest / file.name)

    # Despliega los archivos Quickshell (p. ej. clock.qml) que usan layer-shell.
    qs_src = ctx.paths.quickshell
    qs_dest = ctx.paths.quickshell_dest
    if qs_src.is_dir():
        if not ctx.dry_run:
            qs_dest.mkdir(parents=True, exist_ok=True)
            for qfile in qs_src.iterdir():
                if qfile.is_file():
                    shutil.copy2(qfile, qs_dest / qfile.name)
