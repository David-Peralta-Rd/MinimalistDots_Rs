"""Paso: Docker sin sudo (bk5_docker.sh)."""

import getpass
import os
import shutil

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

    if ctx.dry_run:
        docker_ok = True
    elif shutil.which("sg"):
        docker_ok = ctx.run_ok(["sg", "docker", "-c", "docker info"])
    else:
        # `sg` (shadow-utils) no está instalado: probamos docker directamente.
        docker_ok = ctx.run_ok(["docker", "info"])
    if docker_ok:
        print(ctx.t("docker.funciona"), f"'{user}'.")
    else:
        print(ctx.t("docker.sin_respuesta"))

    print(ctx.t("docker.reiniciar_sesion"))
