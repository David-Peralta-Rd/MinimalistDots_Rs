"""Paso 1b: despliegue de los catálogos de idioma (`lang/*.toml`).

Los lee `md` en runtime (ADR-0004): sin este paso, el binario no encontraría
`es.toml`/`en.toml` y mostraría las claves sin traducir.
"""

import shutil

from installer.context import Context
from installer.errors import StepError

ORDER = 25


def run(ctx: Context) -> None:
    src = ctx.paths.lang
    dest = ctx.paths.lang_dest
    if not src.is_dir():
        raise StepError(f"{ctx.t('comun.configuracion_no_encontrada')} {src}")

    catalogs = sorted(src.glob("*.toml"))
    if not catalogs:
        raise StepError(f"{ctx.t('comun.configuracion_no_encontrada')} {src}")

    print(ctx.t("idioma.desplegando"))
    for path in catalogs:
        print(f"  {path.name} -> {dest}")
        if not ctx.dry_run:
            dest.mkdir(parents=True, exist_ok=True)
            shutil.copy2(path, dest / path.name)
