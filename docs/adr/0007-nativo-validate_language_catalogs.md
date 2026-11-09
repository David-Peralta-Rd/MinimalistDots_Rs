# docs/adr/0007-nativo-validate_language_catalogs.md
# ADR-0007: Función nativa validate_language_catalogs
Estado: propuesto · Fecha: 2026-11-09

## Operación
`validate_language_catalogs(dir: str) -> CatalogReport`: lee todos los `*.toml`, verifica `[meta]`, claves faltantes frente a `es`, duplicados y marcadores `{nombre}` coherentes.

## Por qué Rust
No es rendimiento: es **fuente única de verdad**. Los menús (Rust) y `mdots language check` (Python) deben aceptar exactamente el mismo formato; implementarlo dos veces acabaría en desvío. Vive en md_core; Python lo llama vía md_native.

## Línea base
No aplica (la ganancia es corrección). Benchmark solo informativo.

## Riesgos
Acoplar el instalador a la compilación de la extensión (ya aceptado en ADR-0005).

## Reconsiderar
Si el formato se vuelve trivial o cambia a un estándar (Fluent) → eliminar la función.

Para investigar: serde(deny_unknown_fields), thiserror, PyO3 #[pyclass] frozen, interpolación de marcadores
