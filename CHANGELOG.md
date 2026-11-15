# CHANGELOG.md
# Changelog
Formato [Keep a Changelog](https://keepachangelog.com/es-ES/1.1.0/) · [SemVer](https://semver.org/lang/es/).

## [Unreleased]
### Added
- Estructura base: `installer/` (uv), `md/`, `md-core/`, `md-ui/` (Cargo), `lang/`, `theme/`, `packages.toml`.
- Descubrimiento automático de pasos con `ORDER`; contrato `import-linter` de independencia.
- CI de Python y Rust.
- Migración completa de los scripts bash/Quickshell a Rust (`md`): launcher, menu, volume, shot, record, wallpaper, process, binds, clip, pomodoro, clock, notificaciones, IA (`md ai menu`/`md ai dev`), idioma en runtime.
- Catálogos `lang/es.toml` y `lang/en.toml` con claves `[escritorio]` para menús y notificaciones.
