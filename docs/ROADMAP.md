# Roadmap

## Ahora (v1)
Hitos 0-7 de docs/MIGRATION.md: instalador Python con paridad, `md` con menús, utilidades, notificaciones, Pomodoro, reloj, IA, idioma conmutable.

## Siguiente (v1.x)
- Identidad visual definitiva (tokens más allá de la paleta).
- Selector de idioma en el instalador con buscador y 7 filas.
- Binarios precompilados con atestaciones (ADR-0005).

## Después
- Layer-shell propio si el spike lo exige (ADR-0003).
- Soporte de más idiomas por contribución externa.

## Ideas y futuros desarrollos
| Idea | Por qué no ahora | Qué ya lo habilita |
|---|---|---|
| Modo claro/oscuro por hora (BACKLOG Feat-02) | Evita residentes | Paleta en TOML; basta un segundo `palette-light.toml` |
| Gestión de batería y brillo en laptops (Feat-01) | Fuera del core de escritorio | Subcomando nuevo en `md` + `md_system` |
| `mdots doctor` | Primero la paridad | Cada módulo expone `check()` en `public.py` |
| `md notify-send` propio | Estabilizar notifd primero | zbus ya en `md_system` |
| Instalar un tema desde URL con checksum | Riesgo de cadena de suministro | `theme` ya valida paleta |
| Perfil mínimo de paquetes | Primero el completo | `obligatorio` en packages.toml |
| Vista previa de idioma en el menú | Pulido | Catálogos ya cargables en md_core |
| Actualizador de modelos (`md ai pull`) | Depende de H6 | models.env ya es contrato |
| Empaquetar para el AUR | Requiere releases | `cargo install --locked` ya funciona |
| Pruebas de captura de pantalla (UI) | Backend de pruebas Slint inmaduro | md_ui separa lógica de vista |
