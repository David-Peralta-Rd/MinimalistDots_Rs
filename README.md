# MinimalistDots_Rs

Dotfiles para Hyprland sobre CachyOS/Arch con una capa de desarrollo asistido por IA que corre en mi propia CPU (Ryzen 5 5600G, 32 GB de RAM). Migrado a Rust para máxima velocidad y un solo binario.

## Características

- **Entorno de escritorio**: Hyprland + QuickShell (QML) para menús, notificaciones, reloj y OSD. Waybar sustituido por `md` (un único binario) y QuickShell como capa visual.
- **IA local**: `md ai` levanta modelos con llama.cpp (`llama-server`) y los expone con una API compatible con OpenAI + tool calling. Mide rendimiento real con `llama-bench` y conecta OpenCode al modelo local.
- **Plantillas**: `AGENTS.md`, `SPEC.md` y `tasks.md` que separan lo que escribo yo de lo que delego al agente, con revisión humana de cada cambio.
- **Instalador modular** en Python (uv) con pasos ordenados, menús fuzzy y 7 idiomas (es, en, fr, pt, de, it, zh). Catálogos TOML en `lang/`.
- **Servicios en Lua** para Hyprland (reloj, notificaciones, bienvenida, etc.) y windowrules con `hyprland.lib.rules`.
- **Herramientas incluidas**: grabador de pantalla (gpu-screen-recorder), capturas con grimblast, OCR/QR, portapapeles, pomodoro, gestión de procesos, cambio de idioma en vivo, bienvenida en el primer arranque.

## Comandos (`md`)

| Comando | Descripción |
|---|---|
| `md language` | Selector de idioma con búsqueda fuzzy |
| `md welcome` | Pantalla de bienvenida (idioma, atajos, novedades, GitHub) |
| `md notify …` | Servidor de notificaciones, historial, no-molestar |
| `md record snip/monitor/toggle` | Grabación de pantalla (área o completa) |
| `md shot …` | Capturas (pantalla, área, congelado, OCR, QR) |
| `md ai …` | IA local: modelos, servidores, diagnóstico, OpenCode |
| `md pomodoro …` | Temporizador de concentración/descanso |
| `md clip` / `md volume` / `md process` | Portapapeles, OSD de volumen, gestor de procesos |
| `md clock` | Reloj de escritorio persistente |

## Arquitectura

- `md-core/` — lógica pura (sin I/O).
- `md/` — orquestación y subcomandos.
- `dotfiles/quickshell/` — componentes QML (QuickShell) + singleton visual.
- `lang/` — catálogos i18n (175 keys, mismas en todos los idiomas).
- `dotfiles/` — configuraciones de Hyprland, servicios Lua, quickshell.
- `installer/` — instalador Python (pasos en `steps/`, rutas en `paths.py`).

## Requisitos

- CachyOS/Arch Linux, Hyprland 0.57+
- Rust (rustup), Python 3.12+ (uv), base-devel
- Paquetes: ver `packages.toml`

## Instalación

```bash
git clone https://github.com/David-Peralta-Rd/MinimalistDots_Rs
cd MinimalistDots_Rs
bash install.sh
```

## Desarrollo

```bash
# Compilar
cargo build --release -p md

# Instalar binario (evita "Text file busy")
cp target/release/md /tmp/opencode/md.new && mv -f /tmp/opencode/md.new ~/.local/bin/md

# Lints y tests
cargo clippy --workspace --all-targets -- -D warnings
cargo fmt --all --check
cargo test --workspace
uv run pytest installer/tests
```

## Licencia

MIT
