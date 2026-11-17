---
description: Crea un nuevo modulo adaptado a la arquitectura del proyecto
agent: plan
---

Ejecuta `uv run nuevo-modulo <NOMBRE>` (o `python tools/nuevo_modulo.py <NOMBRE>`).

El comando usa las plantillas generadas por bootstrap en `docs/templates/module/`
y registra el modulo en `ARCHITECTURE.md`.

Luego corre `docs-check` y el humo de importacion. No hagas commit.
