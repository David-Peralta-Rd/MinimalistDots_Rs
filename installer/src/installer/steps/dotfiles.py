"""Paso de ejemplo: lista lo que se desplegaría desde dotfiles/config."""

from installer.context import Context

ORDER = 30


def run(ctx: Context) -> None:
    for item in sorted((ctx.paths.dots_config).iterdir()):
        if item.name.startswith("."):
            continue
        print(f"  {item.name} -> {ctx.paths.config / item.name}")
