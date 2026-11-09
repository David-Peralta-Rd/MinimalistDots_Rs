"""Paso de ejemplo: lista lo que se desplegaría desde dotfiles/config."""
from installer.context import Context

ORDER = 30


def run(ctx: Context) -> None:
    for item in sorted((ctx.repo / "dotfiles" / "config").iterdir()):
        if item.name.startswith("."):
            continue
        print(f"  {item.name} -> {ctx.home / '.config' / item.name}")
