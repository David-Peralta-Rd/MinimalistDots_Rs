# docs/adr/0006-nativo-compute_tree_digest.md
# ADR-0006: Función nativa compute_tree_digest
Estado: propuesto · Fecha: 2026-11-09

## Operación
`compute_tree_digest(root: str, ignore: list[str]) -> str`: hash determinista de un árbol de archivos (rutas relativas ordenadas + contenido). Pura, de grano grueso, libera el GIL.

## Por qué Rust y no una librería
`hashlib` + `os.walk` basta para árboles pequeños; la señal es que el instalador lo ejecuta sobre `~/.config` completo en cada `--dry-run` y compara contra el origen. Con blake3 y recorrido paralelo (rayon) el costo es predecible. Alternativa Python existente: `dirhash` (poco mantenida; verificar). Criptografía propia: no, se usa `blake3` (crate verificado en T01).

## Línea base
Estimación (no medida): `hashlib.sha256` + `os.walk` sobre 2 000 archivos pequeños ≈ 150-400 ms. Objetivo: ≥ 5x menos. El primer benchmark real (pytest-benchmark + criterion, release) sustituye esta estimación.

## Riesgos
Symlinks y permisos (se hashean como metadato); archivos que cambian durante el recorrido; rutas no UTF-8 (error tipado, sin panic).

## Reconsiderar
Si el benchmark real no mejora ≥ 2x → volver a Python puro y borrar la extensión.

Para investigar: blake3, rayon par_iter, PyO3 detach, Result a excepción Python, pytest-benchmark
