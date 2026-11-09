# docs/adr/0002-arquitectura.md
# ADR-0002: Monolito modular en dos mitades
Estado: propuesto · Fecha: 2026-11-09

## Contexto
Hay que evitar la V1 sin fronteras. Hay dos binarios con ciclos de vida distintos: instalar (una vez) y herramientas de escritorio (a diario).

## Decisión
Python (`src/python`): módulos por concepto, cada uno con interface → application → domain → infrastructure y `public.py` único. Sin cola, sin eventos.
- installation: orquesta los pasos (reemplaza steps.sh).
- language: catálogos, selección, traducción.
- theme: paleta y generación de colores.
- packages: grupos de paquetes y pacman/paru.
- dotfiles: descubrir, backup, desplegar, restaurar `custom`.
- system_tuning: sudo, red, SDDM.
- developer_tools: docker, git/GitHub, plantillas IA.
Rust (`src/rust`): crates por capa: md_core (puro) ← md_system, md_ui, md_native ← md (bin).
Comunicación entre mitades: solo archivos (palette, catálogos, settings), nunca IPC.

## Alternativas descartadas
- Un solo binario Rust que instala y es shell: un fallo de UI rompería la instalación.
- Un crate por menú: duplica el runtime de Slint y el peso.

## Consecuencias
Un contrato de archivos (docs/data/) mantiene a Python y Rust sincronizados.

## Reconsiderar
Si `md` supera ~15 MB o un subcomando necesita ciclo de release propio → extraer a binario separado.

Para investigar: import-linter contracts, cargo-deny bans wrappers, pub(crate), hexagonal architecture
