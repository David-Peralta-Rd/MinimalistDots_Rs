"""Paso: IA para desarrollo (bk8_ai.sh). Plantillas, models.env y OpenCode."""

import shutil

from installer.context import Context
from installer.errors import StepError

ORDER = 56


def run(ctx: Context) -> None:
    templates_src = ctx.paths.ai_src / "templates"
    models_example = ctx.paths.ai_src / "config" / "models.env.example"
    api_example = ctx.paths.ai_src / "config" / "api_models.conf.example"
    if not templates_src.is_dir() or not models_example.is_file():
        raise StepError(f"{ctx.t('comun.configuracion_no_encontrada')} {ctx.paths.ai_src}")

    # Plantillas: siempre las del proyecto (se reemplazan en cada instalación).
    if ctx.dry_run:
        print(f"  + {ctx.paths.ai_dir / 'templates'}")
    else:
        templates_dest = ctx.paths.ai_dir / "templates"
        templates_dest.parent.mkdir(parents=True, exist_ok=True)
        shutil.rmtree(templates_dest, ignore_errors=True)
        shutil.copytree(templates_src, templates_dest)
        ctx.paths.ai_dir.mkdir(parents=True, exist_ok=True)
        shutil.copy2(models_example, ctx.paths.ai_dir / "models.env.example")
        shutil.copy2(api_example, ctx.paths.ai_dir / "api_models.conf.example")
    print(ctx.t("ai.instalado"), ctx.paths.ai_dir)

    # models.env: configuración del usuario, se crea una sola vez.
    models_env = ctx.paths.ai_conf / "models.env"
    if models_env.exists():
        print(ctx.t("ai.models_existe"))
    else:
        if not ctx.dry_run:
            ctx.paths.ai_conf.mkdir(parents=True, exist_ok=True)
            shutil.copy2(models_example, models_env)
        print(ctx.t("ai.models_creado"), models_env)

    # api_models.conf: APIs externas, también una sola vez.
    api_conf = ctx.paths.ai_conf / "api_models.conf"
    if not api_conf.exists() and not ctx.dry_run:
        shutil.copy2(api_example, api_conf)
        print(ctx.t("ai.api_creado"), api_conf)

    # OpenCode: repos oficiales y, si no está, el AUR. Un fallo no detiene nada.
    if not shutil.which("opencode") and not ctx.dry_run:
        if not ctx.run_ok(["paru", "-S", "--needed", "--noconfirm", "opencode"]):
            ctx.run_ok(["paru", "-S", "--needed", "--noconfirm", "opencode-bin"])

    # Config de OpenCode desde models.env (best-effort hasta H7, `md ai config`).
    if not ctx.run_ok(["md", "ai", "config"]):
        print(ctx.t("ai.opencode_config_fallo"))

    print(ctx.t("ai.siguiente"))
