# docs/adr/0005-bootstrap-y-distribucion.md
# ADR-0005: Arranque en una máquina limpia
Estado: propuesto · Fecha: 2026-11-09

## Contexto
En un Arch recién instalado no hay uv, Rust ni la extensión compilada.

## Decisión
`install.sh` (~10 líneas, único bash que queda): instala `uv`, `rustup`, `base-devel` con pacman; ejecuta `uv sync --frozen` y `uv run mdots install`. `rust-toolchain.toml` fija el canal exacto y rustup lo descarga en el primer `cargo`.
El paso de instalación compila `md` con `cargo install --path crates/md --locked --root ~/.local` y lo pone en `~/.local/bin/md`.
CI usa imagen `archlinux`; solo prueba lo que no necesita sesión Wayland.

## Alternativas descartadas
- Binarios precompilados en Releases: mejor UX, pero exigen firmado, Trusted Publishing y atestaciones; va a ROADMAP.
- Paquete `rust` de pacman: puede ir por detrás del canal fijado.

## Consecuencias
El primer install compila Slint: lento. Se documenta en README.

## Reconsiderar
Cuando la compilación inicial moleste → Releases con atestaciones.

Para investigar: rustup toolchain file, cargo install --locked, uv sync --frozen, GitHub artifact attestations, PKGBUILD
