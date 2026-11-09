"""Descubre los pasos: cada archivo de esta carpeta con ORDER y run()."""
import importlib
import pkgutil
from collections.abc import Callable
from dataclasses import dataclass

from installer.context import Context


@dataclass(frozen=True)
class Step:
    name: str
    order: int
    run: Callable[[Context], None]


def discover_steps() -> list[Step]:
    steps = []
    for info in pkgutil.iter_modules(__path__):
        if info.name.startswith("_"):
            continue
        module = importlib.import_module(f"{__name__}.{info.name}")
        steps.append(Step(info.name, module.ORDER, module.run))
    return sorted(steps, key=lambda step: step.order)
