"""Paso: tema de SDDM (bk4_sddm_theme.sh)."""

import shutil
import tempfile
from pathlib import Path

from installer.context import Context
from installer.errors import StepError

ORDER = 52

FLAVOR = "mocha"
ACCENT = "blue"
THEME_NAME = f"catppuccin-{FLAVOR}-{ACCENT}"
ASSET_NAME = f"{THEME_NAME}-sddm.zip"
# Descarga directa de la última release: evita la API de GitHub (rate limit)
# y el parseo de JSON; GitHub redirige a la URL real del asset.
DOWNLOAD_URL = f"https://github.com/catppuccin/sddm/releases/latest/download/{ASSET_NAME}"
THEMES_DIR = "/usr/share/sddm/themes"
CONF_FILE = "/etc/sddm.conf.d/catppuccin.conf"


def run(ctx: Context) -> None:
    print(ctx.t("sddm.version_reciente"))

    if ctx.dry_run:
        print(f"  asset: {ASSET_NAME}")
        ctx.run(["sudo", "mkdir", "-p", THEMES_DIR, "/etc/sddm.conf.d"])
        ctx.write_sudo(CONF_FILE, f"[Theme]\nCurrent={THEME_NAME}\n")
        ctx.run(["sudo", "systemctl", "enable", "sddm"])
        print(ctx.t("sddm.instalado"))
        return

    print(ctx.t("sddm.descarga"), THEME_NAME)
    with tempfile.TemporaryDirectory(prefix="md-sddm-") as tmp:
        zip_path = Path(tmp) / "theme.zip"
        extracted = Path(tmp) / "extracted"
        ctx.run(["curl", "-fsSL", "-o", str(zip_path), DOWNLOAD_URL])
        extracted.mkdir()
        shutil.unpack_archive(zip_path, extracted)
        theme_dir = extracted / THEME_NAME
        if not theme_dir.is_dir():
            raise StepError(f"el zip descargado no contiene la carpeta '{THEME_NAME}'")

        ctx.run(["sudo", "mkdir", "-p", THEMES_DIR, "/etc/sddm.conf.d"])
        ctx.run(["sudo", "rm", "-rf", f"{THEMES_DIR}/{THEME_NAME}"])
        ctx.run(["sudo", "cp", "-r", str(theme_dir), f"{THEMES_DIR}/"])

    ctx.write_sudo(CONF_FILE, f"[Theme]\nCurrent={THEME_NAME}\n")
    ctx.run(["sudo", "systemctl", "enable", "sddm"])
    print(ctx.t("sddm.instalado"))