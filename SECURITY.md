# SECURITY.md
# Seguridad

## Reportar una vulnerabilidad
Abre un aviso privado en la pestaña Security del repositorio (GitHub Security Advisories). No publiques detalles en un issue.

## Política de actualización
- Las dependencias se fijan en `uv.lock` y `Cargo.lock` (comiteados) y se instalan con `--frozen` / `--locked`.
- Las actualizaciones rutinarias esperan 7 días desde su publicación. Las correcciones de seguridad se adoptan de inmediato, tras confirmar su origen oficial.
- Auditoría en CI: `pip-audit` y `cargo deny check`.

## Qué toca este proyecto
Escribe en `~/.config`, instala paquetes con sudo y crea una regla temporal en sudoers. Revisa `docs/security/THREAT_MODEL.md` antes de ejecutarlo.
