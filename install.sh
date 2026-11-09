# install.sh
#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "${BASH_SOURCE[0]}")"

sudo pacman -S --needed --noconfirm uv rustup base-devel git
rustup show >/dev/null      # instala el canal fijado en rust-toolchain.toml
uv sync --frozen
exec uv run mdots install "$@"
