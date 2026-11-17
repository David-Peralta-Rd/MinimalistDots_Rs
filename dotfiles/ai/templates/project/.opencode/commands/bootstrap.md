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

## Verificación post-generación (obligatoria, 2-3 revisiones)

Antes de cerrar, ejecuta las siguientes verificaciones en bucle (máximo 3 iteraciones; si alguna falla, corrige y repite):

1. **Estructura y módulos**
   - Revisa que cada módulo tenga `public.py`/`public.ts` como interfaz pública y que no haya imports cruzados no declarados en `ARCHITECTURE.md`.
   - Ejecuta `docs-check` (y `docs-check --write` si aplica) y el linter de dependencias de módulos.
   - Confirma que no existan `utils/`, `helpers/`, `common/` ni lógica de negocio fuera de su módulo.

2. **Dockerfiles y docker-compose**
   - Revisa cada `Dockerfile` y su par `Dockerfile.dev` (si existe) para que el entrypoint, puertos, variables de entorno y volúmenes sean coherentes entre dev y prod.
   - Ejecuta `docker compose config` (y `docker compose -f docker-compose.dev.yml config` si aplica) para validar sintaxis y resolución de servicios.
   - Verifica que los nombres de servicios, redes y volúmenes referenciados en `docker-compose*.yml` existan y coincidan con los usados en los Dockerfiles.

3. **Conexiones y humo**
   - Ejecuta el humo de importación del proyecto (p. ej. `python -c "import src"` o el comando equivalente del stack) para detectar imports rotos.
   - Verifica las conexiones a base de datos/servicios externos (URLs, credenciales desde `kernel/settings.py`, puertos) y que los healthchecks (si existen) apunten a endpoints reales.
   - Levanta los contenedores en modo dry-run si el entorno lo permite; si no, valida al menos que `docker compose up --no-start` no falle.

4. **Cierre**
   - Si tras 3 iteraciones persiste un fallo, PARA y reporta el comando exacto y su salida, sin forzar el cierre.
   - Solo marca el bootstrap como completo cuando las 3 verificaciones pasen en una misma iteración.
