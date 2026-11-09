from pathlib import Path

from installer.paths import Paths


def test_paths_are_built_from_repo_and_home(tmp_path: Path) -> None:
    paths = Paths(repo=tmp_path / "repo", home=tmp_path / "home")
    assert paths.dots_config == tmp_path / "repo" / "dotfiles" / "config"
    assert paths.hypr == tmp_path / "home" / ".config" / "hypr"
    assert paths.settings.name == "settings.toml"
