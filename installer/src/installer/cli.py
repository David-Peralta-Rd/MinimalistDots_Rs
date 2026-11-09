"""Entrada de `mdots`. Sin lógica de negocio."""

import argparse

from installer.context import Context
from installer.steps import discover_steps


def main() -> None:
    parser = argparse.ArgumentParser(prog="mdots")
    parser.add_argument("action", choices=["install"])
    parser.add_argument("--dry-run", action="store_true")
    parser.add_argument("--list", action="store_true")
    parser.add_argument("--only")
    args = parser.parse_args()

    steps = [s for s in discover_steps() if args.only in (None, s.name)]
    if args.list:
        print("\n".join(f"{s.order:>3}  {s.name}" for s in steps))
        return
    ctx = Context.create(dry_run=args.dry_run)
    for step in steps:
        print(f"== {step.name}")
        step.run(ctx)
