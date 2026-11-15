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
| ✅ | **md-core: `Palette`** + `Color` (serde desde TOML; conversión a `rgba(RRGGBBaa)` para Hyprland y `#AARRGGBB` para Slint) + tests | nuevo, en `md-core/src/{color,palette}.rs` |
| ✅ | **md-core: idioma** (`Catalog`, cadena de respaldo código→base→es→clave, `available_languages`, `validate_language_catalogs`) + tests | ADR-0004/0007 |
| ✅ | **md-core: búsqueda difusa** (`score(q, s)`: empieza-igual → palabra → contiene → fuzzy) + tests | `MdMenu.qml` (función `score`) |
| ✅ | **md-core: `compute_tree_digest`** (blake3 + rayon, árbol determinista, symlinks/permisos como metadato) + benchmark (criterion) | ADR-0006 |
| ✅ | **md-ui: `ShellHost` + ventana base Slint** (`MdWindow`, `AboutWindow`, `slint_color`) | `MdMenu.qml` / ADR-0003 |
| ✅ | **md: binario `md`** con clap + subcomando `about` (AboutSlint) + `--version` | `md/src/main.rs` |

> H0 completo. La verificación real del spike (launcher con foco, OSD sin robar foco,
> popup arriba-derecha) necesita una sesión Wayland + las windowrules de H2; se comprobará ahí.

## H1 · Contratos de datos (la base que todo usa)

| | Tarea | Origen → Destino |
|---|---|---|
| ✅ | **`packages.toml` completo**: 14 grupos de paquetes con `origen` y `obligatorio` (se eliminó `quickshell`/`qt6-declarative`, reemplazados por Rust/Slint) | `c2_install_packages.sh` → `packages.toml` |
| ✅ | **`lang/es.toml` + `lang/en.toml` completos**: 71 claves del instalador (paridad verificada; `validate_language_catalogs` → válido) | `src/setup/lang/options/lg*.cfg` → `lang/` |
| ✅ | **`installer/language.py`**: descubrir catálogos, cadena de respaldo (código→base→es→clave), búsqueda difusa, selección y guardado en `settings.toml` | `load_lang.sh` → `language.py` |
| ✅ | Tests de `language.py` (catálogos, respaldo, selección, búsqueda) + integración en `context.py` (`Context.t` con cadena de respaldo) | `installer/tests/` |

> Las claves de los **menús** (launcher, portapapeles, pomodoro…) se añadirán en H4–H7 junto a cada menú. Quedan `fr/de/pt` como idiomas futuros (ROADMAP: por contribución).

## H2 · Dotfiles (config Hyprland + archivos sueltos)

| | Tarea | Origen → Destino |
|---|---|---|
| ✅ | Copiar la config Hyprland en Lua (`general`, `animations`, `env`, `monitors`, `windowrules`, `keybinds`, `vars`, `lib/`, `services/`) + punto de entrada `hyprland.lua` a `dotfiles/config/hypr/` | `src/hyprland-dots/config/hypr/*` |
| ✅ | Adaptar `vars.lua` y `keybinds.lua`: las rutas apuntan a `~/.local/bin/md <subcomando>` en vez de los `.sh` | `vars.lua` / `keybinds.lua` |
| ✅ | Copiar `foot/foot.ini` y `dolphinrc` | `src/hyprland-dots/config/{foot,dolphinrc}` |
| ✅ | Copiar servicios Lua a `dotfiles/hypr-services/` (adaptando `desktop_clock` y `notifications`: Quickshell → `md`) | `src/hyprland-dots/local/bin/MinimalistDots/services/*.lua` |
| ✅ | `colors.lua` estático generado desde `theme/palette.toml` (corrige el bug de `border_inactive` del `bk5_colors_lua.sh`); la regeneración en el instalador va en H3 | `bk5_colors_lua.sh` → `colors.lua` |
| ✅ | Windowrules para ventanas Slint (`app-id md.*`: float + center; pin/noborder/noanim se afinan en H4) | `windowrules.lua` + ADR-0003 |

## H3 · Instalador Python (paridad con el bash)

| | Tarea | Origen → Destino |
|---|---|---|
| ✅ | **steps/sudo.py**: sesión sudo sin caducidad (sudoers.d temporal, validación con `visudo`) | `c0_sudo_session.sh` |
| ✅ | **steps/folders.py**: crear carpetas (`~/.config/hypr/{hyprland,custom}`, `~/.local/bin/MinimalistDots/services`, `~/.cache/hypr`, `~/models`) | `c1_new_folders.sh` |
| ✅ | **steps/packages.py**: `pacman -Syu`, asegurar `paru`, instalar grupos de `packages.toml` (dedupe) + módulo `installer/packages.py` | `c2_install_packages.sh` |
| ✅ | **steps/dotfiles.py**: descubrir `dotfiles/config/*`, backup con timestamp, confirmación única (cancelar = `CancelledError`), desplegar, restaurar `custom/` | `c3_backup_and_install.sh` |
| ✅ | **steps/dotfiles.py + `installer/theme.py`**: regeneración de `colors.lua` desde `theme/palette.toml`; `hypridle/hyprlock/hyprpaper.conf` pasados a dotfiles estáticos | `c3_backup_hypr/bk*.sh` |
| ✅ | **steps/services.py**: despliegue de `dotfiles/hypr-services/*.lua` → `~/.local/bin/MinimalistDots/services/` | `c4_install_services_and_scripts.sh` |
| ✅ | **steps/zsh.py**: `.aliaszsh` (alias IA → `md ai`), `.zshrc.d/plugins.zsh`, bloque delimitado en `.zshrc` | `bk1_zsh.sh` |
| ➖ | **gestor de procesos**: obsoleto, reemplazado por `md process` (H5) | `bk2_process_manager.sh` → `md process` |
| ✅ | **steps/sddm.py**: descargar e instalar tema SDDM desde GitHub releases | `bk4_sddm_theme.sh` |
| ✅ | **steps/docker.py**: docker sin sudo + aviso `newgrp docker` | `bk5_docker.sh` |
| ✅ | **steps/github.py**: login por navegador + firma GPG (omisible sin TTY) | `bk6_github.sh` |
| ✅ | **steps/network.py**: BBR + fq, tarjeta persistente (udev/systemd), DNS con caché, `ParallelDownloads` | `bk7_99_network_optimization.sh` |
| ✅ | **steps/ai.py**: plantillas (`dotfiles/ai/`) + `models.env` + OpenCode | `bk8_ai.sh` |
| ⬜ | Orquestación en `cli.py`/`steps`: recarga Hyprland al final, errores no fatales en c4, `--only`, `--dry-run` | `steps.sh` |
| ✅ | Tests por paso (28 tests: language, theme, packages, dotfiles, c4) + `--list`/dry-run por paso verificados; el dry-run **completo** requiere entorno Arch real | `installer/tests/` |

## H4 · `md` menús (launcher + dmenu genérico)

| | Tarea | Origen → Destino |
|---|---|---|
| ✅ | **md launcher**: listar `.desktop`, búsqueda fuzzy con relevancia, lanzar apps (incl. Terminal=true) | `launcher.qml` |
| ✅ | **md menu**: dmenu genérico (stdin → elegir), `--raw`, miniatura por TAB/`\x1f`, borrado (`--delete-cmd`/`--wipe-cmd`) | `MdMenu.qml` + `md_menu` wrapper |
| ✅ | Widget Slint `MdMenu` reutilizable en `md-ui` (buscar, navegar, seleccionar, borrar) | `MdMenu.qml` |
| ➖ | Wrappers/symlinks para compatibilidad de atajos: no hacen falta, `vars.lua` ya apunta a `md <subcomando>` | `md_menu`, `md_launcher` |

> Pendiente de verificación real: el app-id de las ventanas Slint y las windowrules
> (`pin`, `no_border`, `no_anim`, `stay_focused`) necesitan una sesión Wayland; se
> comprobará en el spike de H8. La resolución de iconos del launcher es "mejor
> esfuerzo" (rutas absolutas + índice de `png`/`svg`/`xpm` en los temas de iconos).

## H5 · Utilidades (`md` subcomandos)

| | Tarea | Origen → Destino |
|---|---|---|
| ✅ | **md volume**: `wpctl` (up/down/mute) + OSD `MdOsd` (icono/nivel, auto-cierre ~1.2 s) | `volume.sh` + `osd-volume.qml` |
| ✅ | **md shot**: capturas con `grimblast` (pantalla/área/congelado/monitor/OCR/QR) | `screenshot.sh` |
| ✅ | **md record**: grabación con `gpu-screen-recorder` (área/monitor/audio/toggle) | `screenrecord.sh` |
| ✅ | **md wallpaper**: selector con miniaturas + `hyprpaper` | `select_wallpaper.sh` |
| ✅ | **md process**: gestor de procesos (lista `ps` y mata por PID) | `bk2_process_manager.sh` |
| ✅ | **md binds**: visor de atajos agrupado por categoría (lee `keybinds.json`) | `show_binds` wrapper |
| ✅ | **md clip**: historial de `cliphist` (elegir→`wl-copy`, borrar, wipe) | `md_cliphist_list` + `cliphist` |

> Notificaciones (shot/record/process/volume) y las miniaturas reales de `clip`/`wallpaper`
> (hoy se carga la imagen completa) quedan para H6/H8. El OSD de volumen no roba foco
> (regla `md.*` sin `stay_focused`); su anclado arriba-centrado se afina en H8.

## H6 · Notificaciones

| | Tarea | Origen → Destino |
|---|---|---|
| ✅ | **md notify serve** (servidor notifd, zbus): tarjetas (urgencia, progreso, no-molestar), historial `notifications.jsonl` | `notifd.qml` |
| ✅ | **md notify history/dnd/dismiss/clear/status**: historial + copiar + no-molestar + cerrar | `notifications_menu.sh` |

> Diferido a H8: invocación de acciones (`ActionInvoked`) y anclado arriba-derecha del
> popup (hoy se muestra centrado, como el OSD). Ambas requieren verificación en sesión
> Wayland. El demonio es residente (`md notify serve`, lanzado por el servicio
> `notifications.lua`) y el popup no roba foco (regla `md.*` sin `stay_focused`).

## H7 · Pomodoro, reloj, IA e idioma

| | Tarea | Origen → Destino |
|---|---|---|
| ✅ | **md pomodoro**: preguntas (foco/descanso/sesiones), temporizador en widget, fases, sonido (descarga/conversión), toggle | `pomodoro.qml` + `pomodoro.sh` |
| ✅ | **md clock**: reloj de escritorio persistente | `clock.qml` |
| ✅ | **md ai menu**: elegir modelos, levantar/apagar `llama-server` (:8080/:8081), descarga de `.gguf`, RAM, `doctor` | `llm_menu.sh` |
| ✅ | **md ai dev**: `ai-up/down/doctor/bench/code/new`, `models.env`, config OpenCode | `ai.sh` |
| ✅ | **Idioma conmutable en runtime**: `md` lee `settings.toml` al arrancar (cambio aplica al reiniciar, sin proceso residente) | ADR-0004 |

> Idioma runtime: `md/i18n.rs` lee `~/.config/minimalistdots/settings.toml` (clave
> `idioma`) y carga la cadena de catálogos desde `~/.config/minimalistdots/lang/`
> (sección `[escritorio]` de `es.toml`/`en.toml`, 36 claves). El instalador despliega
> `lang/*.toml` en el paso `lang` (ORDER 25). Las app names de `notify-send`
> (Screenshot, Screen Recorder, Process Manager, Historial) son identificadores y no
> se traducen. `md` traduce títulos/prompt de menús y los avisos de shot/record/process.

## H8 · Cierre y pulido

| | Tarea | Notas |
|---|---|---|
| ✅ | Actualizar `README.md`, `CHANGELOG.md`, `ROADMAP.md` | reflejar lo migrado |
| ⬜ | Auditoría: `pip-audit`, `cargo deny check`, `cargo clippy -- -D warnings` | T01 de DEPENDENCIES.md |
| ⬜ | Prueba real en VM/imagen Arch: instalar de cero y usar todos los atajos | CI `archlinux` |
| ⬜ | Eliminar `temporal-antigua-config/` | solo cuando TODO esté ✅ |

---

## Notas de alcance (decisiones ya tomadas)

- El instalador es **Python/uv** y las herramientas son **Rust/Slint** (ADR-0001/0002). No se migra todo a Rust.
- Los servicios Lua de Hyprland y la config `hyprland.lua` **se conservan**: son dotfiles, no scripts del instalador.
- Los catálogos de idioma usan claves en **español** (excepción documentada, ADR-0004); los identificadores de código, en inglés.
- Las ventanas Slint son `xdg-toplevel` posicionadas con windowrules (sin layer-shell en v1, ADR-0003).
- `unsafe_code` del workspace pasó de `forbid` a `deny`: el código generado por Slint usa `unsafe` interno con `#[allow]`; nuestro propio código sigue sin poder usar `unsafe`.
