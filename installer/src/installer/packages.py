"""Grupos de paquetes leídos desde `packages.toml`."""

import tomllib
from dataclasses import dataclass
from pathlib import Path


@dataclass(frozen=True)
class PackageGroup:
    nombre: str
    origen: str
    obligatorio: bool
    paquetes: tuple[str, ...]


def load_groups(path: Path) -> list[PackageGroup]:
    """Carga los grupos de paquetes de `packages.toml`."""
    data = tomllib.loads(path.read_text(encoding="utf-8"))
    groups: list[PackageGroup] = []
    for raw in data.get("grupo", []):
        groups.append(
            PackageGroup(
                nombre=str(raw["nombre"]),
                origen=str(raw["origen"]),
                obligatorio=bool(raw.get("obligatorio", False)),
                paquetes=tuple(str(package) for package in raw["paquetes"]),
            )
        )
    return groups


def all_packages(groups: list[PackageGroup]) -> list[str]:
    """Aplana y deduplica los paquetes de todos los grupos, conservando el orden."""
    seen: set[str] = set()
    result: list[str] = []
    for group in groups:
        for package in group.paquetes:
            if package not in seen:
                seen.add(package)
                result.append(package)
    return result
