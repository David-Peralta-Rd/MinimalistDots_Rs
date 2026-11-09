# docs/adr/0004-idioma.md
# ADR-0004: Catálogos TOML compartidos y cambio por reinicio
Estado: propuesto · Fecha: 2026-11-09

## Contexto
BACKLOG: idioma escalable, con buscador, solo aplica al reiniciar, sin proceso residente, claves en español para el autor, textos en el idioma elegido.

## Decisión
- Un archivo por idioma: `src/dotfiles/lang/<BCP47>.toml` (ej. `es-CO.toml`), con `[meta] codigo, nombre, base`. Se descubren por nombre de archivo; no hay tabla fija en código.
- Claves de catálogo en español (excepción documentada a "nombres en inglés": son datos, no código). Identificadores de Python/Rust, en inglés.
- Cadena de respaldo: código → base → `es` → la clave.
- Elección guardada en `~/.config/minimalistdots/settings.toml`. Cada herramienta la lee al arrancar: el cambio vale para el siguiente arranque. Sin caché ni servicio.
- Menú de idioma: 7 filas visibles + búsqueda difusa por código o nombre.
- Parser único: `validate_language_catalogs` (md_core) expuesto a Python vía PyO3.

## Alternativas descartadas
- Slint `@tr`/gettext (.po): estándar, pero Python no lo comparte y las claves no serían en español.
- Fluent: potente, innecesario; v1 no necesita plurales ni género.

## Reconsiderar
Si hacen falta plurales o interpolación compleja → Fluent.

Para investigar: BCP 47, locale fallback chain, tomllib, serde derive Deserialize, XDG base directory
