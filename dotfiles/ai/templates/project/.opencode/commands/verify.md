---
description: Verifica estructura, Dockerfiles/docker-compose y conexiones del proyecto (2-3 revisiones)
agent: plan
---

Ejecuta una verificación completa del proyecto generado, en bucle de hasta 3 iteraciones. Si alguna verificación falla, corrige y repite. No hagas commit.

1. **Estructura y módulos**
   - Cada módulo expone su interfaz pública (`public.py`/`public.ts`); sin imports cruzados no declarados.
   - Sin `utils/`, `helpers/`, `common/`.
   - `docs-check` y linter de dependencias de módulos en verde.

2. **Dockerfiles y docker-compose**
   - Coherencia entre `Dockerfile` y `Dockerfile.dev` (entrypoint, puertos, env, volúmenes).
   - `docker compose config` (y variante dev) sin errores.
   - Servicios, redes y volúmenes referenciados existen y coinciden.

3. **Conexiones y humo**
   - Humo de importación del proyecto en verde.
   - URLs/puertos/credenciales desde `kernel/settings.py` correctos.
   - Healthchecks apuntan a endpoints reales.
   - `docker compose up --no-start` (o equivalente) sin fallos.

Cierre: solo reporta éxito si las 3 secciones pasan en una misma iteración. Si persiste un fallo tras 3 intentos, reporta comando exacto y salida.
