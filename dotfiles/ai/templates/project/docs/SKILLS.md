# Skills — __PROJECT__

Los skills viven en `.agents/skills/` (espejo en `.opencode/skills/`).

Skills instalados por defecto al crear el proyecto:
- `project-architect` (local): lee `REQUIREMENTS.md` y genera el diseño completo.
- `find-skills` (vercel-labs/skills)
- `tdd` (mattpocock/skills)
- `code-review` (mattpocock/skills)
- `improve-codebase-architecture` (mattpocock/skills)
- `systematic-debugging` (obra/superpowers)

Flujo recomendado:
1. Llena `REQUIREMENTS.md`.
2. Ejecuta `/bootstrap` en OpenCode.
3. Revisa y ajusta.
4. Continúa con las tareas de `tasks.md`.

## Comandos

- Buscar: `npx skills find <consulta>`
- Instalar: `npx skills add <paquete>`
- Instalar uno concreto: `npx skills add <paquete> --skill <nombre>`
- Actualizar todos: `npx skills update`
- Crear uno propio: `npx skills init <nombre>`

## Convenciones

- Un skill = una carpeta con `SKILL.md`.
- Los skills instalados no se editan a mano; se actualizan con `npx skills update`.
- Los skills propios del proyecto van en `.agents/skills/<nombre>/`.
- Mantén `skills-lock.json` en el repositorio para reproducir la instalación.
