from pathlib import Path

from installer.language import available_languages


def test_available_languages_reads_meta(tmp_path: Path) -> None:
    (tmp_path / "es.toml").write_text('[meta]\ncodigo = "es"\nnombre = "Español"\n')
    (tmp_path / "en.toml").write_text('[meta]\ncodigo = "en"\nnombre = "English"\n')
    assert available_languages(tmp_path) == [("en", "English"), ("es", "Español")]
