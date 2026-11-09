# README.md
# MinimalistDots_Rs

Dotfiles minimalistas de Hyprland para Arch/CachyOS. El instalador es Python (uv) y las herramientas de escritorio son Rust (Slint). Los pasos del instalador son archivos sueltos, y el idioma se elige desde un archivo `.toml` por idioma.

## Instalar
```bash
git clone https://github.com/David-Peralta-Rd/MinimalistDots_Rs.git
cd MinimalistDots_Rs
bash install.sh            # primera vez: compila Rust, puede tardar
```
Probar sin tocar nada: `uv run mdots install --dry-run`.

## Estructura
| Carpeta | Qué es |
|---|---|
| `installer/` | Python: `mdots`; un archivo por paso en `src/installer/steps/` |
| `md/` · `md-core/` · `md-ui/` | Rust: binario `md`, lógica pura, interfaz Slint |
| `dotfiles/` | Lo que se instala (`config/`, `hypr-services/`) |
| `lang/` | Un `.toml` por idioma |
| `theme/palette.toml` | La única paleta |
| `packages.toml` | Paquetes por grupo |
| `docs/` | ROADMAP, ADRs, MIGRATION, seguridad |

## Cómo agregar
- **Paso nuevo:** crea `installer/src/installer/steps/<nombre>.py` con `ORDER` y `run(ctx)`. Prueba con `uv run mdots install --only <nombre>`.
- **Idioma nuevo:** copia `lang/es.toml` a `lang/xx-YY.toml` y tradúcelo.
- **Subcomando Rust:** un archivo en `md/src/commands/` y una línea en el `enum` de `clap`.

## Novedades
- **Unreleased:** estructura base y esqueleto andante. Ver [CHANGELOG](CHANGELOG.md).

## Documentación
[ROADMAP](docs/ROADMAP.md) · [MIGRATION](docs/MIGRATION.md) · [SECURITY](SECURITY.md) · [DEPENDENCIES](docs/security/DEPENDENCIES.md)

Licencia MIT.
