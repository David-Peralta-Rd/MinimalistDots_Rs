# MIGRATION.md
# Migración de MinimalistDots (bash + Quickshell) a Rust

Plan de trabajo incremental. Vamos **poco a poco**: cada tarea es pequeña, testeable y
marcable. El origen de cada tarea está en `temporal-antigua-config/` (que se borrará al final).

**Leyenda de estado:** `⬜` pendiente · `🟡` en curso · `✅` hecho

---

## Cómo usamos este archivo

1. Elegimos UNA tarea, la marcamos `🟡`, la implementamos y probamos.
2. Al terminar, la marcamos `✅` y elegimos la siguiente.
3. No se avanza a un hito sin que las tareas previas estén verdes.

---

## H0 · Fundación (casi lista)

| | Tarea | Origen → Destino |
|---|---|---|
| ✅ | Workspace Cargo + uv, ADRs 0001-0008, `install.sh`, CI base | ya hecho |
| ✅ | `theme/palette.toml` completo | `u2_palette.sh` → `theme/palette.toml` |
| ⬜ | **md-core: `Palette`** (serde desde TOML) + generador de colores (`0xRRGGBBAA` para Hyprland, `#AARRGGBB` para Slint) + tests | nuevo, en `md-core/src/` |
| ⬜ | **md-core: `LanguageCatalog`** (leer `<BCP47>.toml`, cadena de respaldo código→base→es→clave) + `validate_language_catalogs` + tests | ADR-0004/0007 |
| ⬜ | **md-core: búsqueda difusa** (`score(q, s)`: empieza-igual → palabra → contiene → fuzzy) + tests | `MdMenu.qml` (función `score`) |
| ⬜ | **md-core: `compute_tree_digest`** (blake3 + rayon, árbol determinista) + benchmark | ADR-0006 |
| ⬜ | **md-ui: `ShellHost` + ventana base Slint** (spike del ADR-0003: launcher con foco, OSD sin robar foco, popup de notificación) | `MdMenu.qml` / ADR-0003 |
| ⬜ | **md: binario `md`** con clap + subcomando `about` (AboutSlint) + carga de paleta/idioma | `md/src/main.rs` |

## H1 · Contratos de datos (la base que todo usa)

| | Tarea | Origen → Destino |
|---|---|---|
| ⬜ | **`packages.toml` completo**: todos los grupos de paquetes (`zsh`, `core`, `apps`, `cursor`, `dolphin`, `sddm`, `screenshot`, `programming`, `screenrecord`, `process`, `partition`, `quickshell`, `network`, `llm`) con `origen` y `obligatorio` | `c2_install_packages.sh` → `packages.toml` |
| ⬜ | **`lang/*.toml` completos**: rellenar es/en con TODAS las claves del instalador y de los menús | `src/setup/lang/options/lg*.cfg` → `lang/` |
| ⬜ | **`installer/language.py`**: descubrir catálogos, selección con buscador (7 filas), guardar en `settings.toml` | `load_lang.sh` → `language.py` |
| ⬜ | Tests de `language.py` (catálogos, respaldo, selección) | `installer/tests/` |

## H2 · Dotfiles (config Hyprland + archivos sueltos)

| | Tarea | Origen → Destino |
|---|---|---|
| ⬜ | Copiar la config Hyprland en Lua (`general`, `animations`, `env`, `monitors`, `windowrules`, `keybinds`, `vars`, `lib/`, `services/`) a `dotfiles/config/hypr/hyprland/` | `src/hyprland-dots/config/hypr/hyprland/*` |
| ⬜ | Adaptar `vars.lua` y `keybinds.lua`: las rutas de scripts apuntan a `md <subcomando>` en vez de los `.sh` | `vars.lua` / `keybinds.lua` |
| ⬜ | Copiar `foot/foot.ini` y `dolphinrc` | `src/hyprland-dots/config/{foot,dolphinrc}` |
| ⬜ | Copiar servicios Lua (`clipboard`, `polkit`, `wallpaper`, `desktop_clock`, `dbus_env`, `footclient`, `hypridle`, `export_keybinds`, `notifications`, `create_custom_config`, `load_direction`) a `dotfiles/hypr-services/` | `src/hyprland-dots/local/bin/MinimalistDots/services/*.lua` |
| ⬜ | Generación de `colors.lua` desde `theme/palette.toml` (sustituye `bk5_colors_lua.sh`) | `bk5_colors_lua.sh` → instalador/theme |
| ⬜ | Windowrules de Hyprland para las ventanas Slint (`app-id md.*`: float, center, pin, noborder) | `windowrules.lua` + ADR-0003 |

## H3 · Instalador Python (paridad con el bash)

| | Tarea | Origen → Destino |
|---|---|---|
| ⬜ | **steps/sudo.py**: sesión sudo sin caducidad (sudoers.d temporal, validación con `visudo`) | `c0_sudo_session.sh` |
| ⬜ | **steps/folders.py**: crear carpetas (`~/.local/bin/MinimalistDots/{scripts,services}`, `~/models`, etc.) | `c1_new_folders.sh` |
| ⬜ | **steps/packages.py**: `pacman -Syu`, asegurar `paru`, instalar grupos de `packages.toml` (dedupe) | `c2_install_packages.sh` |
| ⬜ | **steps/dotfiles.py**: descubrir `dotfiles/config/*`, backup con timestamp, confirmación única (cancelar = salir 10), desplegar, restaurar `custom/` | `c3_backup_and_install.sh` |
| ⬜ | **steps/dotfiles.py**: piezas específicas de Hyprland (colors.lua, etc.) como sub-pasos `bkN_*` | `c3_backup_hypr/bk*.sh` |
| ⬜ | **steps/zsh.py**: `.zshrc`, plugins, alias (sistema/pacman/git/docker) | `bk1_zsh.sh` |
| ⬜ | **steps/process.py**: gestor de procesos | `bk2_process_manager.sh` |
| ⬜ | **steps/sddm.py**: descargar e instalar tema SDDM desde GitHub releases | `bk4_sddm_theme.sh` |
| ⬜ | **steps/docker.py**: docker sin sudo + aviso `newgrp docker` | `bk5_docker.sh` |
| ⬜ | **steps/github.py**: login por navegador + firma GPG (omisible sin TTY) | `bk6_github.sh` |
| ⬜ | **steps/network.py**: BBR + CAKE, tarjeta persistente (udev/systemd), DNS con caché | `bk7_99_network_optimization.sh` |
| ⬜ | **steps/ai.py**: plantillas + `models.env` + config de OpenCode (instalar opencode) | `bk8_ai.sh` |
| ⬜ | Orquestación en `cli.py`/`steps`: recarga Hyprland al final, errores no fatales en c4, `--only`, `--dry-run` | `steps.sh` |
| ⬜ | Tests por paso + `mdots install --dry-run` verde de punta a punta | `installer/tests/` |

## H4 · `md` menús (launcher + dmenu genérico)

| | Tarea | Origen → Destino |
|---|---|---|
| ⬜ | **md launcher**: listar `.desktop`, búsqueda fuzzy con relevancia, lanzar apps (incl. Terminal=true) | `launcher.qml` |
| ⬜ | **md menu**: dmenu genérico (stdin → elegir), `--raw`, miniatura por TAB/`\x1f`, borrado (`deleteCmd`/`wipeCmd`) | `MdMenu.qml` + `md_menu` wrapper |
| ⬜ | Widget Slint `MdMenu` reutilizable en `md-ui` (buscar, navegar, seleccionar, borrar) | `MdMenu.qml` |
| ⬜ | Wrappers/symlinks para compatibilidad de atajos si hace falta | `md_menu`, `md_launcher` |

## H5 · Utilidades (`md` subcomandos)

| | Tarea | Origen → Destino |
|---|---|---|
| ⬜ | **md volume**: `wpctl` + OSD (`md osd`), icono/nivel, mute | `volume.sh` + `osd-volume.qml` |
| ⬜ | **md shot**: capturas (pantalla/área/congelado/OCR/QR) con notificación | `screenshot.sh` |
| ⬜ | **md record**: grabación con `gpu-screen-recorder` (área/monitor/audio/toggle) | `screenrecord.sh` |
| ⬜ | **md wallpaper**: selector con miniaturas + `hyprpaper` | `select_wallpaper.sh` |
| ⬜ | **md process**: gestor de procesos | `bk2_process_manager.sh` |
| ⬜ | **md binds**: visor de atajos agrupado por categoría (lee `keybinds.json`) | `show_binds` wrapper |
| ⬜ | **md clip**: historial de `cliphist` con miniaturas + decode a `wl-copy` | `md_cliphist_list` + `cliphist` |

## H6 · Notificaciones

| | Tarea | Origen → Destino |
|---|---|---|
| ⬜ | **md notify** (servidor notifd): tarjetas arriba-derecha, urgencia, progreso, acciones, historial `notifications.jsonl` | `notifd.qml` |
| ⬜ | **md notify history/dnd/dismiss/clear**: historial + copiar + no-molestar + cerrar | `notifications_menu.sh` |

## H7 · Pomodoro, reloj, IA e idioma

| | Tarea | Origen → Destino |
|---|---|---|
| ⬜ | **md pomodoro**: preguntas (foco/descanso/sesiones), temporizador en widget, fases, sonido (descarga/conversión), toggle | `pomodoro.qml` + `pomodoro.sh` |
| ⬜ | **md clock**: reloj de escritorio persistente | `clock.qml` |
| ⬜ | **md ai menu**: elegir modelos, levantar/apagar `llama-server` (:8080/:8081), descarga de `.gguf`, RAM, `doctor` | `llm_menu.sh` |
| ⬜ | **md ai dev**: `ai-up/down/doctor/bench/code/new`, `models.env`, config OpenCode | `ai.sh` |
| ⬜ | **Idioma conmutable en runtime**: `md` lee `settings.toml` al arrancar (cambio aplica al reiniciar, sin proceso residente) | ADR-0004 |

## H8 · Cierre y pulido

| | Tarea | Notas |
|---|---|---|
| ⬜ | Actualizar `README.md`, `CHANGELOG.md`, `ROADMAP.md` | reflejar lo migrado |
| ⬜ | Auditoría: `pip-audit`, `cargo deny check`, `cargo clippy -- -D warnings` | T01 de DEPENDENCIES.md |
| ⬜ | Prueba real en VM/imagen Arch: instalar de cero y usar todos los atajos | CI `archlinux` |
| ⬜ | Eliminar `temporal-antigua-config/` | solo cuando TODO esté ✅ |

---

## Notas de alcance (decisiones ya tomadas)

- El instalador es **Python/uv** y las herramientas son **Rust/Slint** (ADR-0001/0002). No se migra todo a Rust.
- Los servicios Lua de Hyprland y la config `hyprland.lua` **se conservan**: son dotfiles, no scripts del instalador.
- Los catálogos de idioma usan claves en **español** (excepción documentada, ADR-0004); los identificadores de código, en inglés.
- Las ventanas Slint son `xdg-toplevel` posicionadas con windowrules (sin layer-shell en v1, ADR-0003).
