import shutil
from pathlib import Path

import pytest
from installer.context import Context
from installer.paths import Paths
from installer.steps import ai, lang, network, services, zsh


def _paths(tmp_path: Path) -> Paths:
    return Paths(repo=tmp_path / "repo", home=tmp_path / "home")


def test_update_zshrc_replaces_previous_block() -> None:
    current = (
        "# hola\n"
        "# >>> MinimalistDots zsh >>>\n"
        '[[ -f "$HOME/.zshrc.d/plugins.zsh" ]] && \\\n'
        '    source "$HOME/.zshrc.d/plugins.zsh"\n'
        "# <<< MinimalistDots zsh <<<\n"
        "# adios\n"
    )
    result = zsh.update_zshrc(current, zsh.ZSHRC_BLOCK)
    assert "# hola" in result and "# adios" in result
    assert result.count(zsh.ZSHRC_MARKER_START) == 1
    assert result.strip().endswith(zsh.ZSHRC_MARKER_END)


def test_aliases_point_to_md() -> None:
    assert 'alias ai-up="md ai up"' in zsh.ALIASES
    assert 'alias ai-config="md ai config"' in zsh.ALIASES


def test_plugins_zsh_sources_detected_paths() -> None:
    content = zsh._plugins_zsh("/a/autocomplete.zsh", "/b/autosuggestions.zsh", "/c/syntax.zsh")
    assert 'source "/a/autocomplete.zsh"' in content
    assert 'source "/b/autosuggestions.zsh"' in content
    assert 'source "/c/syntax.zsh"' in content


def test_set_parallel_downloads_updates_existing() -> None:
    assert network.set_parallel_downloads("[options]\nParallelDownloads = 5\n", 10) == (
        "[options]\nParallelDownloads = 10\n"
    )


def test_set_parallel_downloads_uncomments() -> None:
    assert network.set_parallel_downloads("[options]\n#ParallelDownloads = 5\n", 10) == (
        "[options]\nParallelDownloads = 10\n"
    )


def test_set_parallel_downloads_inserts_when_missing() -> None:
    result = network.set_parallel_downloads("[options]\nHoldPkg = pacman\n", 10)
    assert result.startswith("[options]\nParallelDownloads = 10\n")


def test_services_deploys_and_removes_old(tmp_path: Path) -> None:
    paths = _paths(tmp_path)
    (paths.hypr_services).mkdir(parents=True)
    (paths.hypr_services / "polkit.lua").write_text("-- polkit\n")
    (paths.hypr_services / "wallpaper.lua").write_text("-- wallpaper\n")

    # Un servicio antiguo que debe borrarse.
    paths.services.mkdir(parents=True)
    (paths.services / "old.lua").write_text("-- old\n")

    services.run(Context(paths=paths, dry_run=False, language="es"))

    assert (paths.services / "polkit.lua").exists()
    assert (paths.services / "wallpaper.lua").exists()
    assert not (paths.services / "old.lua").exists()


def test_lang_deploys_catalogs(tmp_path: Path) -> None:
    paths = _paths(tmp_path)
    paths.lang.mkdir(parents=True)
    (paths.lang / "es.toml").write_text(
        '[meta]\ncodigo = "es"\nnombre = "Español"\n'
    )
    (paths.lang / "en.toml").write_text(
        '[meta]\ncodigo = "en"\nnombre = "English"\n'
    )

    lang.run(Context(paths=paths, dry_run=False, language="es"))

    assert (paths.lang_dest / "es.toml").exists()
    assert (paths.lang_dest / "en.toml").exists()


def test_ai_copies_templates_and_configs(
    tmp_path: Path, monkeypatch: pytest.MonkeyPatch
) -> None:
    paths = _paths(tmp_path)
    (paths.ai_src / "templates" / "project").mkdir(parents=True)
    (paths.ai_src / "templates" / "project" / "AGENTS.md").write_text("# agents\n")
    (paths.ai_src / "config").mkdir(parents=True)
    (paths.ai_src / "config" / "models.env.example").write_text("# models\n")
    (paths.ai_src / "config" / "api_models.conf.example").write_text("# api\n")

    ctx = Context(paths=paths, dry_run=False, language="es")
    # Evita subprocesos (paru, md ai config): simula que opencode ya está y run_ok es True.
    monkeypatch.setattr(shutil, "which", lambda _name: "/usr/bin/opencode")
    monkeypatch.setattr(Context, "run_ok", lambda _self, *_a, **_k: True)

    ai.run(ctx)

    assert (paths.ai_dir / "templates" / "project" / "AGENTS.md").exists()
    assert (paths.ai_conf / "models.env").exists()
    assert (paths.ai_conf / "api_models.conf").exists()
