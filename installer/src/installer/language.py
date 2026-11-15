"""Descubrimiento, selección y traducción de idiomas (catálogos TOML).

Cada idioma es `lang/<BCP47>.toml` con `[meta]` (`codigo`, `nombre`, `base`
opcional) y el resto de secciones con claves en español. La cadena de respaldo
es: código → base → `es` → la propia clave (ADR-0004).
"""

import tomllib
from collections.abc import Mapping
from dataclasses import dataclass
from pathlib import Path

FALLBACK = "es"


@dataclass(frozen=True)
class Catalog:
    """Un catálogo de idioma cargado desde TOML."""

    codigo: str
    nombre: str
    base: str | None
    sections: Mapping[str, Mapping[str, str]]

    def get(self, section: str, key: str) -> str | None:
        table = self.sections.get(section)
        return table.get(key) if table is not None else None


@dataclass(frozen=True)
class Translator:
    """Traduce claves aplicando la cadena de respaldo código → base → es → clave."""

    chain: tuple[Catalog, ...]

    def t(self, key: str) -> str:
        section, _, name = key.partition(".")
        for catalog in self.chain:
            value = catalog.get(section, name)
            if value is not None:
                return value
        return key


def _load(path: Path) -> Catalog:
    data = tomllib.loads(path.read_text(encoding="utf-8"))
    meta = data.get("meta", {})
    base_value = meta.get("base")
    sections: dict[str, dict[str, str]] = {}
    for key, value in data.items():
        if key == "meta" or not isinstance(value, dict):
            continue
        sections[key] = {str(k): str(v) for k, v in value.items()}
    return Catalog(
        codigo=str(meta.get("codigo", "")),
        nombre=str(meta.get("nombre", "")),
        base=base_value if isinstance(base_value, str) else None,
        sections=sections,
    )


def available_languages(lang_dir: Path) -> list[tuple[str, str]]:
    """Lista los idiomas disponibles como pares `(codigo, nombre)` ordenados."""
    languages: list[tuple[str, str]] = []
    for path in lang_dir.glob("*.toml"):
        catalog = _load(path)
        languages.append((catalog.codigo, catalog.nombre))
    languages.sort()
    return languages


def base_of(codigo: str) -> str:
    """Sufijo base de un código BCP 47 (`es-CO` → `es`)."""
    head, sep, _ = codigo.partition("-")
    return head if sep else codigo


def load_translator(lang_dir: Path, codigo: str) -> Translator:
    """Carga la cadena de catálogos para `codigo`: específico → base → es."""
    chain: list[Catalog] = []
    primary = lang_dir / f"{codigo}.toml"
    if primary.exists():
        catalog = _load(primary)
        base = catalog.base or base_of(codigo)
        chain.append(catalog)
        if base != codigo:
            base_path = lang_dir / f"{base}.toml"
            if base_path.exists():
                chain.append(_load(base_path))
    if codigo != FALLBACK and all(catalog.codigo != FALLBACK for catalog in chain):
        fallback_path = lang_dir / f"{FALLBACK}.toml"
        if fallback_path.exists():
            chain.append(_load(fallback_path))
    return Translator(tuple(chain))


def score(query: str, target: str) -> int | None:
    """Puntúa la coincidencia de `query` en `target`. Menor = mejor; `None` = no hay."""
    q = query.lower()
    s = target.lower()
    if not q:
        return 0
    if s.startswith(q):
        return 0
    if f" {q}" in s:
        return 1
    if q in s:
        return 2
    return 3 if _fuzzy(q, s) else None


def _fuzzy(query: str, target: str) -> bool:
    """¿Todos los caracteres de `query` aparecen en `target`, en orden?"""
    remaining = iter(target)
    return all(char in remaining for char in query)


def filter_languages(
    languages: list[tuple[str, str]], query: str
) -> list[tuple[str, str]]:
    """Filtra por código o nombre con búsqueda difusa, ordenadas por relevancia."""
    query = query.strip()
    if not query:
        return list(languages)
    scored: list[tuple[int, str, str]] = []
    for codigo, nombre in languages:
        best = min(
            (value for value in (score(query, codigo), score(query, nombre)) if value is not None),
            default=None,
        )
        if best is not None:
            scored.append((best, codigo, nombre))
    scored.sort(key=lambda item: (item[0], item[2].lower()))
    return [(codigo, nombre) for _, codigo, nombre in scored]


def save_language(settings: Path, codigo: str) -> None:
    """Guarda la elección de idioma en `settings.toml`."""
    settings.parent.mkdir(parents=True, exist_ok=True)
    settings.write_text(f'idioma = "{codigo}"\n', encoding="utf-8")


def read_language(settings: Path) -> str | None:
    """Lee el idioma guardado, o `None` si no hay."""
    if not settings.exists():
        return None
    data = tomllib.loads(settings.read_text(encoding="utf-8"))
    value = data.get("idioma")
    return value if isinstance(value, str) else None


def choose_language(lang_dir: Path, saved: str | None = None) -> str:
    """Pregunta el idioma (número o búsqueda difusa) y devuelve el código elegido."""
    languages = available_languages(lang_dir)
    codes = {codigo for codigo, _ in languages}
    if saved in codes:
        return saved
    if not languages:
        return FALLBACK
    if len(languages) == 1:
        return languages[0][0]

    print("Selecciona tu idioma / Select your language:")
    for index, (codigo, nombre) in enumerate(languages, start=1):
        print(f"  {index}) {nombre} ({codigo})")

    while True:
        try:
            answer = input(f"Opción / Option (1-{len(languages)} o búsqueda): ").strip()
        except EOFError:
            return FALLBACK
        if answer.isdigit():
            index = int(answer)
            if 1 <= index <= len(languages):
                return languages[index - 1][0]
        matches = filter_languages(languages, answer)
        if len(matches) == 1:
            return matches[0][0]
        if matches:
            print("Coincidencias:")
            for codigo, nombre in matches:
                print(f"  {nombre} ({codigo})")


def resolve_language(
    lang_dir: Path, settings: Path, *, interactive: bool = True
) -> str:
    """Resuelve el idioma: guardado → interactivo → respaldo. Guarda la elección."""
    saved = read_language(settings)
    if saved is not None and (lang_dir / f"{saved}.toml").exists():
        return saved
    if not interactive:
        return FALLBACK
    chosen = choose_language(lang_dir, saved)
    save_language(settings, chosen)
    return chosen
