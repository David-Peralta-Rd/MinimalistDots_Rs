# docs/adr/0001-stack.md
# ADR-0001: Arquetipo y stack
Estado: propuesto · Fecha: 2026-11-09

## Contexto
Dotfiles + instalador + herramientas de escritorio. Sin servidor ni BD. El autor domina Python y aprende Rust.

## Decisión
- Arquetipo 4 (CLI), con idempotencia y registro del 5.
- Instalador: Python 3.14 + uv (backend maturin por el puente PyO3).
- Herramientas y UI: Rust (workspace) + Slint.
- Datos: TOML tipado (palette, lang, packages).
- Raíz: se añaden `install.sh` (entrada pública, ~10 líneas) y `LICENSE` (MIT); `BACKLOG.md` pasa a docs/ROADMAP.md.

## No aplica (motivo)
- Docker prod/dev: no hay servicio; CI usa imagen `archlinux` solo para probar.
- BD, API/OpenAPI, frontend web: no existen.
- AGENTS.md, CLAUDE.md, .opencode/, AGENT_WORKFLOW.md: el autor construye sin IA.
- Dependabot ecosistema docker: sin Dockerfiles.

## Alternativas descartadas
- Todo Python (+PyQt/GTK): contradice el objetivo de aprender Rust y el núcleo nativo.
- Todo Rust (instalador incluido): bash→Rust del instalador (subprocess, tomllib, pruebas rápidas) rinde menos que Python y pierde la mitad del stack deseado.

## Consecuencias
Dos toolchains (uv y cargo) en una máquina de usuario; ver ADR-0005.

## Reconsiderar
Si el instalador crece en cómputo (hash, parsing) → más funciones nativas; si Slint falla (ADR-0003) → plan B de UI.

Para investigar: maturin mixed project, uv workspace, tomllib, clap derive, cargo workspace lints
