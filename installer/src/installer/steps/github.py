"""Paso: Git y GitHub (bk6_github.sh). Login por navegador + firma GPG."""

import shutil
import socket
import subprocess
import sys
import tempfile
from datetime import date
from pathlib import Path

from installer.context import Context
from installer.errors import StepError

ORDER = 54

GH_SCOPES = "admin:public_key,write:gpg_key"


def _git_config_get(ctx: Context, key: str) -> str:
    command = ["git", "config", "--global", key]
    result = subprocess.run(  # noqa: S603
        command, capture_output=True, text=True
    )
    return result.stdout.strip() if result.returncode == 0 else ""


def _confirm(question: str, yes: str) -> bool:
    answer = input(f"{question} ({yes}/n): ").strip().lower()
    return answer in ("", yes.lower())


def _prompt(label: str, default: str) -> str:
    answer = input(f"{label} [{default}]: ").strip()
    return answer or default


def _find_key_id(ctx: Context, email: str) -> str | None:
    output = ctx.capture(["gpg", "--list-secret-keys", "--with-colons", email])
    for line in output.splitlines():
        parts = line.split(":")
        if len(parts) > 4 and parts[0] == "sec":
            return parts[4]
    return None


def _upload_key(ctx: Context, key_id: str) -> None:
    title = f"{socket.gethostname()} {date.today().isoformat()}"
    with tempfile.NamedTemporaryFile("w", suffix=".asc", delete=False) as tmp:
        tmp.write(ctx.capture(["gpg", "--armor", "--export", key_id]))
        pubkey = Path(tmp.name)
    try:
        if not ctx.run_ok(["gh", "gpg-key", "add", str(pubkey), "--title", title]):
            print(ctx.t("github.gpg_subida_error"))
    finally:
        pubkey.unlink(missing_ok=True)


def run(ctx: Context) -> None:
    if not sys.stdin.isatty():
        print(ctx.t("github.sin_terminal"))
        return

    if not _confirm(ctx.t("github.pregunta"), ctx.t("dotfiles.letra_confirmacion")):
        print(ctx.t("github.omitido"))
        return

    name = _prompt(ctx.t("github.pedir_nombre"), _git_config_get(ctx, "user.name"))
    email = _prompt(ctx.t("github.pedir_correo"), _git_config_get(ctx, "user.email"))
    if not name or not email:
        raise StepError(ctx.t("github.datos_obligatorios"))

    ctx.run(["git", "config", "--global", "user.name", name])
    ctx.run(["git", "config", "--global", "user.email", email])
    ctx.run(["git", "config", "--global", "init.defaultBranch", "main"])

    if shutil.which("brave"):
        ctx.run(["gh", "config", "set", "browser", "brave"])

    if ctx.run_ok(["gh", "auth", "status", "--hostname", "github.com"]):
        print(ctx.t("github.sesion_activa"))
        status = ctx.capture(["gh", "auth", "status", "--hostname", "github.com"])
        if "gpg_key" not in status:
            ctx.run(["gh", "auth", "refresh", "--hostname", "github.com", "--scopes", GH_SCOPES])
    else:
        print(ctx.t("github.abriendo_navegador"))
        ctx.run(
            [
                "gh", "auth", "login", "--hostname", "github.com",
                "--git-protocol", "https", "--web", "--scopes", GH_SCOPES,
            ]
        )

    ctx.run(["gh", "auth", "setup-git", "--hostname", "github.com"])

    key_id = _find_key_id(ctx, email)
    if key_id is None:
        print(ctx.t("github.generando_gpg"))
        ctx.run(["gpg", "--quick-generate-key", f"{name} <{email}>", "ed25519", "sign", "2y"])
        key_id = _find_key_id(ctx, email)
    if key_id is None:
        raise StepError(ctx.t("github.gpg_error"))

    _upload_key(ctx, key_id)

    ctx.run(["git", "config", "--global", "user.signingkey", key_id])
    ctx.run(["git", "config", "--global", "commit.gpgsign", "true"])
    ctx.run(["git", "config", "--global", "tag.gpgSign", "true"])

    print(ctx.t("github.listo"), f"'{name} <{email}>' (GPG {key_id}).")
