"""Paso: optimización de red (bk7_99_network_optimization.sh)."""

import re
from pathlib import Path

from installer.context import Context
from installer.errors import StepError

ORDER = 55

RING_BUFFER_MAX = 2048
PACMAN_PARALLEL = 10

_MODULES = "tcp_bbr\nsch_fq\nsch_cake\n"

_SYSCTL = """\
# fq = cola recomendada para BBR (pacing por flujo, escala bien en NICs multicola).
net.core.default_qdisc = fq
net.ipv4.tcp_congestion_control = bbr

# Buffers TCP
net.core.rmem_max = 33554432
net.core.wmem_max = 33554432
net.ipv4.tcp_rmem = 4096 131072 33554432
net.ipv4.tcp_wmem = 4096 65536 33554432

# Colas del sistema bajo carga
net.core.netdev_max_backlog = 16384
net.core.somaxconn = 8192
net.ipv4.tcp_max_syn_backlog = 8192

# Muchas conexiones concurrentes
net.ipv4.ip_local_port_range = 10240 65535
net.ipv4.tcp_tw_reuse = 1
net.ipv4.tcp_fin_timeout = 15

# No volver a empezar lento tras una pausa entre descargas
net.ipv4.tcp_slow_start_after_idle = 0

net.ipv4.tcp_no_metrics_save = 1

# Detecta MTU problemático
net.ipv4.tcp_mtu_probing = 1

net.ipv4.tcp_notsent_lowat = 131072

# TCP Fast Open
net.ipv4.tcp_fastopen = 3

# Keepalives moderados
net.ipv4.tcp_keepalive_time = 120
net.ipv4.tcp_keepalive_intvl = 15
net.ipv4.tcp_keepalive_probes = 5

# --- SEGURIDAD Y ESTABILIDAD ---
net.ipv4.tcp_syncookies = 1
net.ipv4.tcp_synack_retries = 3
net.ipv4.conf.all.accept_redirects = 0
net.ipv6.conf.all.accept_redirects = 0
"""

_TUNE_SCRIPT = f"""\
#!/usr/bin/env bash
# Generado por MinimalistDots. Uso: md-network-tune <interfaz>
IFACE="${{1:?uso: md-network-tune <interfaz>}}"

# Wi-Fi: desactivar el ahorro de energía
case "$IFACE" in
    wl*) command -v iw > /dev/null && iw dev "$IFACE" set power_save off &> /dev/null || true ;;
esac

command -v ethtool > /dev/null || exit 0

read -r RX_MAX TX_MAX < <(ethtool -g "$IFACE" 2> /dev/null | awk '
    /Pre-set maximums/ {{s = 1; next}}
    /Current hardware settings/ {{s = 0}}
    s && /^RX:/ {{rx = $2}}
    s && /^TX:/ {{tx = $2}}
    END {{print rx + 0, tx + 0}}')

cap() {{ if [ "$1" -gt {RING_BUFFER_MAX} ]; then echo {RING_BUFFER_MAX}; else echo "$1"; fi; }}

if [ "${{RX_MAX:-0}}" -gt 0 ] && [ "${{TX_MAX:-0}}" -gt 0 ]; then
    ethtool -G "$IFACE" rx "$(cap "$RX_MAX")" tx "$(cap "$TX_MAX")" 2> /dev/null || true
fi

ethtool -K "$IFACE" rx on tx on tso on gso on gro on &> /dev/null || true
"""

_SERVICE = """\
[Unit]
Description=MinimalistDots: optimizacion de red para %i
After=sys-subsystem-net-devices-%i.device

[Service]
Type=oneshot
ExecStart=/usr/local/bin/md-network-tune %i
"""

_UDEV = (
    'ACTION=="add", SUBSYSTEM=="net", KERNEL=="en*|eth*|wl*", '
    'TAG+="systemd", ENV{SYSTEMD_WANTS}+="md-network-tune@%k.service"\n'
)

_DNS = """\
[Resolve]
DNS=1.1.1.1 1.0.0.1 8.8.8.8 8.8.4.4
FallbackDNS=9.9.9.9 149.112.112.112
Domains=~.
DNSSEC=allow-downgrade
DNSOverTLS=opportunistic
Cache=yes
"""


def set_parallel_downloads(content: str, parallel: int) -> str:
    """Ajusta `ParallelDownloads` en el contenido de `pacman.conf`."""
    if re.search(r"^[ \t]*#?[ \t]*ParallelDownloads", content, re.MULTILINE):
        return re.sub(
            r"^[ \t]*#?[ \t]*ParallelDownloads.*$",
            f"ParallelDownloads = {parallel}",
            content,
            flags=re.MULTILINE,
        )
    return re.sub(
        r"^\[options\]",
        f"[options]\nParallelDownloads = {parallel}",
        content,
        flags=re.MULTILINE,
        count=1,
    )


def _detect_interface(ctx: Context) -> str | None:
    virtual = ("lo", "docker", "br-", "veth", "virbr", "tun", "tap", "wg")
    for line in ctx.capture(["ip", "route", "show", "default"]).splitlines():
        parts = line.split()
        if "dev" in parts:
            dev = parts[parts.index("dev") + 1]
            if not dev.startswith(virtual):
                return dev
    return None


def run(ctx: Context) -> None:
    print(ctx.t("red.iniciando"))

    interface = _detect_interface(ctx) if not ctx.dry_run else "eth0"
    if not interface:
        raise StepError(ctx.t("red.sin_interfaz"))
    print(ctx.t("red.interfaz_detectada"), interface)

    print(ctx.t("red.aplicando_sysctl"))
    ctx.write_sudo("/etc/modules-load.d/network-optimization.conf", _MODULES)
    ctx.write_sudo("/etc/sysctl.d/99-network-optimization.conf", _SYSCTL)
    for module in ("tcp_bbr", "sch_fq", "sch_cake"):
        ctx.run_ok(["sudo", "modprobe", module])

    print(ctx.t("red.optimizando_nic"), interface)
    ctx.write_sudo("/usr/local/bin/md-network-tune", _TUNE_SCRIPT)
    ctx.run(["sudo", "chmod", "755", "/usr/local/bin/md-network-tune"])
    ctx.run(["sudo", "/usr/local/bin/md-network-tune", interface])

    print(ctx.t("red.persistencia"))
    ctx.write_sudo("/etc/systemd/system/md-network-tune@.service", _SERVICE)
    ctx.write_sudo("/etc/udev/rules.d/99-md-network-tune.rules", _UDEV)
    ctx.run(["sudo", "systemctl", "daemon-reload"])
    ctx.run(["sudo", "udevadm", "control", "--reload"])

    print(ctx.t("red.configurando_dns"))
    ctx.write_sudo("/etc/systemd/resolved.conf.d/dns_latency.conf", _DNS)
    ctx.run(["sudo", "ln", "-sf", "/run/systemd/resolve/stub-resolv.conf", "/etc/resolv.conf"])
    ctx.run(["sudo", "systemctl", "enable", "--now", "systemd-resolved"])

    pacman_conf = Path("/etc/pacman.conf")
    if PACMAN_PARALLEL > 0 and (pacman_conf.exists() or ctx.dry_run):
        if ctx.dry_run:
            ctx.write_sudo("/etc/pacman.conf", "<pacman.conf con ParallelDownloads>")
        else:
            content = set_parallel_downloads(pacman_conf.read_text(), PACMAN_PARALLEL)
            ctx.write_sudo("/etc/pacman.conf", content)

    print(ctx.t("red.listo"))
