"""Paso: configuración de Zsh (bk1_zsh.sh)."""

import subprocess
from pathlib import Path

from installer.context import Context
from installer.errors import StepError

ORDER = 51

ZSHRC_MARKER_START = "# >>> MinimalistDots zsh >>>"
ZSHRC_MARKER_END = "# <<< MinimalistDots zsh <<<"

ALIASES = """\
# Evita que el alias md='mkdir -p' de oh-my-zsh eclipse el binario md.
unalias md 2>/dev/null || true

# ============================================================
# SISTEMA
# ============================================================

alias t="touch"
alias mk="mkdir -p"


# ============================================================
# PACMAN / PARU
# ============================================================

# Instalar
alias in="paru -S --needed --noconfirm"
# Desinstalar
alias un="paru -Rns --noconfirm"
# Actualizar sistema
alias up="paru -Syu --noconfirm"
# Eliminar paquetes huérfanos
alias po='sudo pacman -Rns $(pacman -Qdtq)'


# ============================================================
# GIT
# ============================================================

alias gs="git status"
alias ga="git add"
alias gm="git commit -m"
alias gpl="git pull"
alias gps="git push"


# ============================================================
# DOCKER
# ============================================================

alias dk="docker"
alias dk-up="docker compose up -d"
alias dk-dev="docker compose -f docker-compose.dev.yml up -d"
alias dk-dev-dw="docker compose -f docker-compose.dev.yml down"
alias dk-dw="docker compose down"
alias dk-lg="docker logs -f"
alias dk-pl="docker pull"
alias dk-clean="docker system prune -a -f"
alias dk-run="docker compose run --entrypoint bash"


# ============================================================
# IA PARA DESARROLLO (md ai; ver md ai --help)
# ============================================================

alias ai-up="$HOME/.local/bin/md ai up"
alias ai-down="$HOME/.local/bin/md ai stop"
alias ai-status="$HOME/.local/bin/md ai status"
alias ai-doctor="$HOME/.local/bin/md ai doctor"
alias ai-bench="$HOME/.local/bin/md ai bench"
alias ai-new="$HOME/.local/bin/md ai new"
alias ai-code="$HOME/.local/bin/md ai code"
alias ai-config="$HOME/.local/bin/md ai config"
"""

ZSHRC_BLOCK = f"""
{ZSHRC_MARKER_START}
[[ -f "$HOME/.zshrc.d/plugins.zsh" ]] && \\
    source "$HOME/.zshrc.d/plugins.zsh"
{ZSHRC_MARKER_END}
"""


def _plugins_zsh(autocomplete: str, autosuggestions: str, syntax: str) -> str:
    return f"""\
# ============================================================
# ZSH PLUGINS
# ============================================================

PLUGIN_DIR="$HOME/.local/share/zsh/plugins"

# zsh-autocomplete (debe cargarse temprano; NO ejecutar compinit a mano)
if [[ -f "{autocomplete}" ]]; then
    source "{autocomplete}"
fi

# zsh-completions
fpath+=(/usr/share/zsh/site-functions)

ZSH_AUTOSUGGEST_STRATEGY=(history completion)
ZSH_AUTOSUGGEST_HIGHLIGHT_STYLE='fg=8'

if [[ -f "{autosuggestions}" ]]; then
    source "{autosuggestions}"
fi

# zsh-syntax-highlighting (debe cargarse al final)
if [[ -f "{syntax}" ]]; then
    source "{syntax}"
fi

[[ -f "$HOME/.aliaszsh" ]] && source "$HOME/.aliaszsh"
"""


def update_zshrc(current: str, block: str) -> str:
    """Reemplaza el bloque delimitado de MinimalistDots por `block`."""
    kept: list[str] = []
    inside = False
    for line in current.splitlines():
        stripped = line.strip()
        if stripped == ZSHRC_MARKER_START:
            inside = True
            continue
        if inside:
            if stripped == ZSHRC_MARKER_END:
                inside = False
            continue
        if ".zshrc.d/plugins.zsh" in line:
            continue
        kept.append(line)
    return "\n".join(kept).rstrip("\n") + "\n" + block


def _find_plugin(package: str, suffix: str) -> str | None:
    command = ["pacman", "-Ql", package]
    result = subprocess.run(  # noqa: S603
        command, capture_output=True, text=True
    )
    if result.returncode != 0:
        return None
    for line in result.stdout.splitlines():
        parts = line.split()
        if len(parts) >= 2 and parts[1].endswith(suffix):
            return parts[1]
    return None


def run(ctx: Context) -> None:
    autocomplete: str | None
    autosuggestions: str | None
    syntax: str | None
    if ctx.dry_run:
        autocomplete = autosuggestions = syntax = "<ruta-del-plugin>"
    else:
        autocomplete = _find_plugin("zsh-autocomplete", "zsh-autocomplete.plugin.zsh")
        autosuggestions = _find_plugin("zsh-autosuggestions", "zsh-autosuggestions.zsh")
        syntax = _find_plugin("zsh-syntax-highlighting", "zsh-syntax-highlighting.zsh")
        if not (autocomplete and autosuggestions and syntax):
            raise StepError(
                "no se encontraron los plugins de zsh "
                "(zsh-autocomplete, zsh-autosuggestions, zsh-syntax-highlighting)"
            )

    home = ctx.paths.home
    ctx.write(home / ".aliaszsh", ALIASES)
    plugins_zsh = _plugins_zsh(autocomplete, autosuggestions, syntax)
    ctx.write(home / ".zshrc.d" / "plugins.zsh", plugins_zsh)

    zshrc: Path = home / ".zshrc"
    current = zshrc.read_text(encoding="utf-8") if zshrc.exists() else ""
    ctx.write(zshrc, update_zshrc(current, ZSHRC_BLOCK))

    print(ctx.t("zsh.instalado"))
