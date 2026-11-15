# H4_PROMPT.md
# Prompt para continuar la migración en otro chat (H4: menús Rust)

Copia este texto en un chat nuevo para retomar la migración desde donde quedó.

---

```text
Estoy migrando mi config "MinimalistDots" de scripts bash + menús Quickshell a una
versión en Rust. El proyecto es un workspace en /home/crdavidperalta/projects/MinimalistDots_Rs
(repo git, rama main). La config antigua está en temporal-antigua-config/ (solo referencia).

ARQUITECTURA (ya decidida en docs/adr/):
- Instalador en Python/uv (comando `mdots`) y herramientas de escritorio en Rust/Slint (binario `md`).
- Crates: md-core (lógica pura), md-ui (Slint), md (binario con clap). Comunicación solo por archivos.
- Las ventanas Slint son xdg-toplevel posicionadas con windowrules de Hyprland (app-id md.<nombre>),
  SIN layer-shell en v1 (ADR-0003).

LO QUE YA ESTÁ HECHO (no lo rehagas):
- H0 md-core: Color/Palette (serde desde theme/palette.toml), catálogos de idioma
  (language.rs con validate_language_catalogs), fuzzy::score(query,target) -> Option<u8>
  (0=empieza, 1=palabra, 2=contiene, 3=fuzzy), compute_tree_digest (blake3+rayon).
- H0 md-ui: trait ShellHost + SlintHost, ventana base MdWindow y AboutWindow en ui/app.slint,
  slint_color() para convertir md_core::Color a slint::Color. build.rs compila el .slint.
- H0 md: binario con clap, subcomando `about` (AboutSlint) y --version.
- H1 datos: packages.toml (14 grupos), lang/es.toml + en.toml (71 claves).
- H2 dotfiles: config Hyprland Lua + servicios ya migrados a dotfiles/ (keybinds apuntan a md).
- H3 instalador Python: todos los pasos (sudo/folders/packages/dotfiles/services/zsh/sddm/
  docker/github/network/ai), 28 tests.

TAREA AHORA (H4): construir los menús interactivos en Rust.
1. `md launcher`: lista aplicaciones .desktop (nombre, icono, Terminal=true), búsqueda difusa
   con relevancia (usar md_core::fuzzy::score), navegación con teclado (↑/↓/Ctrl+j/k/n/p, Enter
   elige, Esc cierra), lanza la app elegida. Referencia: temporal-antigua-config/src/hyprland-dots/
   local/bin/MinimalistDots/quickshell/launcher.qml y MdMenu.qml.
2. `md menu <nombre> [--prompt X] [--raw]`: dmenu genérico: lee líneas por stdin, muestra menú,
   imprime la línea elegida por stdout. Soporta --raw (línea completa = etiqueta) y miniatura por
   TAB o \x1f. Referencia: MdMenu.qml + el wrapper md_menu en bk3_quickshell.sh.
3. Un widget Slint reutilizable "MdMenu" en md-ui (búsqueda, lista, navegación, selección) que
   ambos subcomandos usen. Reutiliza el ShellHost/SlintHost existente.
4. Añade los subcomandos launcher y menu al enum de clap en md/src/main.rs.

CONVENCIONES OBLIGATORIAS:
- Lints del workspace: unsafe_code="deny", clippy unwrap_used/expect_used/panic = "deny".
  No uses .unwrap()/.expect()/panic!. El código generado por Slint va en un módulo con
  #[allow(clippy::unwrap_used, expect_used, panic)] (mira md-ui/src/lib.rs).
- Mensajes/comentarios en español; identificadores de código en inglés.
- md-core no debe tener dependencias de UI; la lógica pura va en md-core, la vista en md-ui,
  la orquestación en md.
- Tests con `cargo test`, clippy con `cargo clippy --workspace --all-targets -- -D warnings`,
  formato con `cargo fmt`.
- Los subcomandos ya definidos (para no renombrar): launcher, menu, clip, volume, shot, record,
  wallpaper, process, binds, notify, pomodoro, ai, clock.

Plan de trabajo con checklist en docs/MIGRATION.md (sección H4). Marca las tareas ⬜→✅ al
terminarlas. Ve paso a paso: primero `md menu` (dmenu, más simple) y después `md launcher`.

Empieza leyendo: docs/MIGRATION.md (sección H4), docs/adr/0003, md-core/src/fuzzy.rs,
md-ui/src/lib.rs y md-ui/ui/app.slint, md/src/main.rs, y los .qml de referencia en
temporal-antigua-config. Pregúntame si algo no queda claro antes de codificar.
```
