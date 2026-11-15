from pathlib import Path

from installer.packages import all_packages, load_groups

SAMPLE = """\
[[grupo]]
nombre = "zsh"
origen = "aur"
obligatorio = true
paquetes = ["zsh", "git"]

[[grupo]]
nombre = "core"
origen = "repo"
obligatorio = true
paquetes = ["git", "hyprland"]

[[grupo]]
nombre = "apps"
origen = "aur"
obligatorio = false
paquetes = ["foot", "brave-bin"]
"""


def test_load_groups_parses(tmp_path: Path) -> None:
    path = tmp_path / "packages.toml"
    path.write_text(SAMPLE)
    groups = load_groups(path)
    assert [group.nombre for group in groups] == ["zsh", "core", "apps"]
    assert groups[0].origen == "aur"
    assert groups[0].obligatorio is True
    assert groups[0].paquetes == ("zsh", "git")
    assert groups[2].obligatorio is False


def test_all_packages_dedupes_preserving_order(tmp_path: Path) -> None:
    path = tmp_path / "packages.toml"
    path.write_text(SAMPLE)
    # "git" aparece en zsh y core: se conserva solo la primera aparición.
    assert all_packages(load_groups(path)) == ["zsh", "git", "hyprland", "foot", "brave-bin"]
