# Skills — __PROJECT__

Los skills viven en `.agents/skills/` (espejo en `.opencode/skills/`).

Skills instalados por defecto al crear el proyecto:
- `find-skills` (vercel-labs/skills)
- `tdd` (mattpocock/skills)
- `code-review` (mattpocock/skills)
- `improve-codebase-architecture` (mattpocock/skills)
- `systematic-debugging` (obra/superpowers)

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
