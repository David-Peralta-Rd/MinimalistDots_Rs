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

# Estructura de carpetas:
```xml
MinimalistDots_Rs/
├── install.sh              # ~10 líneas: instala uv y ejecuta `uv run mdots`
├── pyproject.toml          # workspace de uv (lo genera uv)
├── Cargo.toml              # workspace de Cargo
├── installer/              # Python: el instalador (comando `mdots`)
│   └── src/installer/
│       ├── cli.py          # ≤25 líneas
│       ├── context.py      # rutas, idioma, paleta (reemplaza u3_env.sh)
│       ├── path.py         # carga de rutas generales del dotfiles
│       ├── language.py     # selección de lenguaje para la instalación de dotfiles(aun en desarrollo)
│       └── steps/          # UN archivo por paso (los antiguos bk*.sh)
│           ├── sudo.py  packages.py  dotfiles.py  zsh.py
│           ├── docker.py  github.py  network.py  sddm.py  ai.py
│       └── test/           # Carpeta de test de pasos, lenguajes y rutas.
│           ├── test_steps.py
│           ├── test_path.py
│           ├── test_language.py
├── md/                     # Rust: binario `md` (md menu, md volume, md shot…)
│   └── src/commands/       # un archivo por subcomando
├── md-core/                # Rust lib pura: paleta, idioma, búsqueda difusa
├── md-ui/                  # Slint: ui/*.slint (menú, OSD, notificaciones)
├── dotfiles/               # lo que se instala: config/, hypr-services/
├── lang/                   # es.toml, en.toml, fr.toml… (uno por idioma)
├── theme/palette.toml      # la única paleta
├── packages.toml           # paquetes por grupo
└── docs/                   # ROADMAP, ADRs, MIGRATION (lo que ya te di)
```

## Cómo agregar
- **Paso nuevo:** crea `installer/src/installer/steps/<nombre>.py` con `ORDER` y `run(ctx)`. Prueba con `uv run mdots install --only <nombre>`.
- **Idioma nuevo:** copia `lang/es.toml` a `lang/xx-YY.toml` y tradúcelo.
- **Subcomando Rust:** un archivo en `md/src/commands/` y una línea en el `enum` de `clap`.

## Novedades
- **Unreleased:** estructura base y esqueleto andante. Ver [CHANGELOG](CHANGELOG.md).

## Documentación
[ROADMAP](docs/ROADMAP.md) · [MIGRATION](docs/MIGRATION.md) · [SECURITY](SECURITY.md) · [DEPENDENCIES](docs/security/DEPENDENCIES.md)

Licencia MIT.
