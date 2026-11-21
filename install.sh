#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "${BASH_SOURCE[0]}")"

sudo pacman -S --needed --noconfirm uv rustup base-devel git
rustup default stable
rustup show >/dev/null      # instala el canal fijado en rust-toolchain.toml
uv sync --frozen
uv run mdots install "$@"

# Recargar Hyprland al terminar (si hay sesión activa).
if command -v hyprctl >/dev/null 2>&1; then
    hyprctl reload >/dev/null 2>&1 || true
fi
