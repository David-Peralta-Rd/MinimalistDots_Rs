# docs/security/DEPENDENCIES.md
# Dependencias verificadas

Fecha de verificación: 2026-11-09. Los avisos no se pudieron consultar en OSV.dev, GitHub Advisories ni RustSec (acceso bloqueado desde mi entorno), así que **ninguna fila está verificada como "sí"**. T01 debe correr `pip-audit`, `cargo deny check` y `osv-scanner`.

| Pieza | Versión | Soporte hasta | Avisos abiertos | Verificada | Fuente |
|---|---|---|---|---|---|
| Python | 3.14.10 | 2030-10-31 | sin consultar | parcial | endoflife.date/python |
| Rust | 1.99.1 (fijado en rust-toolchain.toml) | solo la última recibe parches | sin consultar | parcial | endoflife.date/rust |
| uv | 0.13.7 (6 días: usar `exclude-newer`) | n/d | sin avisos en PyPI JSON al 2026-11-09 | parcial | pypi.org/pypi/uv/json |
| ruff | 0.17.3 (resuelto por el cooldown; 0.17.4 tiene 4 días) | n/d | sin consultar | parcial | uv.lock + PyPI |
| mypy · pytest · pip-audit | 2.4.1 · 9.2.0 · 2.10.1 | n/d | sin consultar | parcial | uv.lock |
| import-linter | resuelto por el lock (2.16 es de hace 3 días) | n/d | sin consultar | parcial | uv.lock |
| Slint | 1.18.1 · MSRV 1.92 | n/d | sin consultar | parcial | crates.io + Cargo.lock |
| clap · serde · toml · thiserror | 4.6.7 · 1.0.229 · 1.2.1 · 2.0.21 | n/d | sin consultar | parcial | Cargo.lock |
| cargo-deny | 0.20.2 | n/d | sin consultar | parcial | crates.io API |

**Excepciones y riesgos**
- Slint: licencia `GPL-3.0-only OR Royalty-free-2.0 OR Software-3.0`; el repo es MIT. Leer los términos antes de distribuir binarios. `deny.toml` permite `GPL-3.0-only` solo como marcador: **ajústalo a la licencia que elijas**.
- Sin pre-releases: `prerelease = "disallow"` en el `pyproject.toml` raíz.
