"""Paso: Docker sin sudo (bk5_docker.sh)."""

import getpass
import os

from installer.context import Context

ORDER = 53


def _real_user() -> str:
    return os.environ.get("SUDO_USER") or getpass.getuser()


def run(ctx: Context) -> None:
    user = _real_user()

    ctx.run(["sudo", "systemctl", "enable", "--now", "docker.service"])

    if not ctx.run_ok(["getent", "group", "docker"]):
        ctx.run(["sudo", "groupadd", "docker"])
    ctx.run(["sudo", "usermod", "-aG", "docker", user])

    if ctx.dry_run or ctx.run_ok(["sg", "docker", "-c", "docker info"]):
        print(ctx.t("docker.funciona"), f"'{user}'.")
    else:
        print(ctx.t("docker.sin_respuesta"))

    print(ctx.t("docker.reiniciar_sesion"))
