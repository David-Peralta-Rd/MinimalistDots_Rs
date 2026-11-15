"""Paso 0: sesión de sudo sin caducidad durante la instalación (c0_sudo_session.sh)."""

import getpass
import os
import subprocess
import tempfile
from pathlib import Path

from installer.context import Context
from installer.errors import StepError

ORDER = 10

SUDOERS_DIR = Path("/etc/sudoers.d")


def _real_user() -> str:
    return os.environ.get("SUDO_USER") or getpass.getuser()


def run(ctx: Context) -> None:
    print(ctx.t("sudo.configurando_sudo"))

    user = _real_user()
    if user == "root":
        raise StepError(ctx.t("sudo.no_root"))

    line = f"Defaults:{user} timestamp_timeout=-1"
    dest = SUDOERS_DIR / f"sudo_timeout_{user}"

    if ctx.dry_run:
        print(f"  {line}")
        print("  $ visudo -c -f <temporal>")
        print(f"  $ sudo mv <temporal> {dest}")
        print(f"  $ sudo chmod 0440 {dest} && sudo chown root:root {dest}")
        return

    with tempfile.NamedTemporaryFile("w", suffix=".md-sudoers", delete=False) as tmp:
        tmp.write(f"{line}\n")
        tmp_path = Path(tmp.name)

    try:
        validate = ["visudo", "-c", "-f", str(tmp_path)]
        subprocess.run(validate, check=True, capture_output=True)  # noqa: S603
        ctx.run(["sudo", "mv", str(tmp_path), str(dest)])
        ctx.run(["sudo", "chmod", "0440", str(dest)])
        ctx.run(["sudo", "chown", "root:root", str(dest)])
    finally:
        tmp_path.unlink(missing_ok=True)
