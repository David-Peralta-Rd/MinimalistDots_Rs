# AGENTS.md — __PROJECT__

## Stack
<lenguaje, framework, base de datos y versión de cada uno>

## Comandos
- Instalar: <comando>
- Test: <comando>
- Lint y tipos: <comando>
- Arquitectura: <comando>
- Humo de importación: <comando>
- Rutas: <comando>
- docs-check: <comando>
- new-module: <comando>
- suite: <comando>
- Migraciones: <comando>
- Levantar prod: `cd src && docker compose up --build -d`
- Levantar dev: `cd src && docker compose -f docker-compose.dev.yml up --build`
- Rust (si aplica): `cargo fmt --check`, `cargo clippy`, `cargo test`

## Estructura
- `src/`: todo el código y la configuración de ejecución.
- Módulos por concepto de negocio; interfaz pública en `public.py`/`public.ts`.
- `kernel/`: paths, settings, registry, errores base, logging, tiempo, ids, paginación.
- Dependencias entre módulos: acíclicas, declaradas en `ARCHITECTURE.md`, verificadas por linter.
- Los documentos de un solo módulo viven junto a su código, nunca en `docs/`.

## Nunca
- Crear a mano lo que un generador produce; importar algo interno de otro módulo.
- Consultar tablas de otro módulo; poner lógica de negocio en interface o cli.py.
- Crear `utils/`, `helpers/`, `common/`; componer rutas con strings; leer `os.environ` fuera de `kernel/settings.py`.
- Usar `sys.path`, `PYTHONPATH` o imports relativos que salgan del módulo.
- Poner en `docs/` documentos de un solo módulo; editar a mano los bloques autogenerados.
- Usar `CMD` en un Dockerfile si el entrypoint puede hacerlo; editar un Dockerfile sin revisar su par dev.
- Montar volúmenes de código en producción; pinear versiones a mano; usar pre-releases.
- Añadir dependencias sin que la tarea lo pida; hacer commit o push.
- Editar `tasks.md` o tarjetas; tocar archivos fuera de los de la tarjeta; modificar pruebas para que pasen.
- Leer o imprimir lockfiles, `.venv`, `dist` o migraciones autogeneradas.
- Escribir `unsafe`, `unwrap`, `expect` o `panic` en rutas públicas sin ADR.
- Poner lógica en el crate de enlace; llamar a la extensión elemento por elemento.
- Ejecutar `cargo update` o `cargo add` salvo que la tarea lo pida; commitear `target/`.
- Ejecutar `docker build`, `docker compose up` ni instalaciones salvo que la tarea lo pida.
- Dejar un proceso en primer plano.

## Cómo trabajas (modo delegado)
Trabajas sin supervisión: nadie contesta preguntas mientras corres. Decide con lo que dicen tu tarjeta, el README del módulo y este archivo; si algo no está claro, para y repórtalo.

1. Lee SOLO tu tarjeta `docs/tasks/<ID>.md` y los documentos o secciones que cita. No explores el árbol ni leas SPEC.md entero.
2. Haz únicamente la tarea que te pidan por ID, solo si es [DELEGAR] y sus dependencias figuran como hechas en `tasks.md`.
3. Toca SOLO los archivos que lista la tarjeta. Si necesitas otro, para y dilo.
4. Las pruebas de aceptación de la tarjeta son la especificación. Si no existen o no fallan antes de implementar, para y repórtalo (salvo que tu rol sea escribirlas). Nunca cambies una prueba para que pase.
5. Escribe un plan de máximo 3 líneas y continúa sin esperar respuesta.
6. Implementa respetando los límites entre módulos, nombres explícitos y tipos estrictos; rutas y opciones solo desde el kernel. Luego corre la verificación de la tarjeta y los comandos de este archivo. No releas archivos que ya leíste; usa las opciones de salida corta de cada comando.
7. Si algo falla: corrige y reintenta, máximo 3 veces. Si sigue fallando, PARA y reporta el comando exacto y su salida.
8. Si cambiaste comportamiento, actualiza los documentos que la tarjeta lista (README, unidad, STACK o DATA del módulo; Unreleased del CHANGELOG) y corre `docs-check --write`.
9. No hagas commit ni push, no instales dependencias salvo que la tarea lo pida, no edites `tasks.md` ni tarjetas.
10. Responde en 10 líneas como máximo: ID, archivos cambiados, resultado de cada comando (ok / falla) y dudas o decisiones tomadas. No pegues código ni el diff: lo reviso con git.
