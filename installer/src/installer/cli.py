"""Entrada de `mdots`. Sin lógica de negocio."""

import argparse
import subprocess
import sys

from installer.context import Context
from installer.errors import CancelledError, StepError
from installer.steps import discover_steps


def main() -> None:
    parser = argparse.ArgumentParser(prog="mdots")
    parser.add_argument("action", choices=["install"])
    parser.add_argument("--dry-run", action="store_true")
    parser.add_argument("--list", action="store_true")
    parser.add_argument("--only")
    args = parser.parse_args()

    steps = [step for step in discover_steps() if args.only in (None, step.name)]
    if args.list:
        print("\n".join(f"{step.order:>3}  {step.name}" for step in steps))
        return

    ctx = Context.create(dry_run=args.dry_run)
    try:
        for step in steps:
            print(f"== {step.name}")
            step.run(ctx)
    except CancelledError:
        return
    except StepError as error:
        print(f"❌ {error}", file=sys.stderr)
        raise SystemExit(1) from error
    except subprocess.CalledProcessError as error:
        print(f"❌ comando falló: {' '.join(error.cmd)}", file=sys.stderr)
        raise SystemExit(error.returncode) from error
