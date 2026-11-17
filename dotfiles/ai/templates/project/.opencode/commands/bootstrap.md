---
description: Ejecuta project-architect con REQUIREMENTS.md y activa skills derivadas
agent: plan
---

Lee `REQUIREMENTS.md` y ejecuta el skill `project-architect`.

Si faltan datos, pídelos brevemente y actualiza `REQUIREMENTS.md`.

Luego:
1. Genera o actualiza `SPEC.md`, ADRs, `ARCHITECTURE.md`, datos, seguridad, árbol `src/` y `tasks.md`.
2. Usa `improve-codebase-architecture` para validar la estructura.
3. Usa `tdd` para definir el primer test de aceptación.
4. Usa `code-review` para revisar los documentos generados.
5. Resume cambios y próximos pasos.

No hagas commit.