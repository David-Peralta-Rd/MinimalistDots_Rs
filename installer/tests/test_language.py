from pathlib import Path

from installer.language import (
    available_languages,
    base_of,
    filter_languages,
    load_translator,
    read_language,
    resolve_language,
    save_language,
    score,
)

ES = '[meta]\ncodigo = "es"\nnombre = "Español"\n\n[comun]\nbienvenido = "Bienvenido"\n'
EN = '[meta]\ncodigo = "en"\nnombre = "English"\n\n[comun]\nbienvenido = "Welcome"\n'


def test_available_languages_reads_meta(tmp_path: Path) -> None:
    (tmp_path / "es.toml").write_text(ES)
    (tmp_path / "en.toml").write_text(EN)
    assert available_languages(tmp_path) == [("en", "English"), ("es", "Español")]


def test_base_of_splits_bcp47() -> None:
    assert base_of("es-CO") == "es"
    assert base_of("en") == "en"


def test_translator_falls_back_to_es_then_key(tmp_path: Path) -> None:
    (tmp_path / "es.toml").write_text(ES)
    (tmp_path / "es-CO.toml").write_text(
        '[meta]\ncodigo = "es-CO"\nnombre = "Español (Colombia)"\n\n[comun]\nsaludo = "Hola"\n'
    )
    translator = load_translator(tmp_path, "es-CO")
    assert translator.t("comun.saludo") == "Hola"  # del catálogo específico
    assert translator.t("comun.bienvenido") == "Bienvenido"  # cae a es
    assert translator.t("comun.no_existe") == "comun.no_existe"  # la propia clave


def test_score_ranks_by_specificity() -> None:
    assert score("ter", "terminal") == 0
    assert score("ter", "mi terminal") == 1
    assert score("ter", "intercambio") == 2
    assert score("trm", "terminal") == 3
    assert score("xyz", "terminal") is None


def test_filter_languages_by_code_or_name() -> None:
    languages = [("es", "Español"), ("en", "English"), ("es-CO", "Español (Colombia)")]
    assert filter_languages(languages, "en") == [("en", "English")]
    assert filter_languages(languages, "") == languages


def test_save_and_read_language_roundtrip(tmp_path: Path) -> None:
    settings = tmp_path / "settings.toml"
    save_language(settings, "es-CO")
    assert read_language(settings) == "es-CO"


def test_read_language_missing_returns_none(tmp_path: Path) -> None:
    assert read_language(tmp_path / "no.toml") is None


def test_resolve_language_reuses_saved(tmp_path: Path) -> None:
    (tmp_path / "es.toml").write_text(ES)
    settings = tmp_path / "settings.toml"
    save_language(settings, "es")
    assert resolve_language(tmp_path, settings, interactive=False) == "es"


def test_resolve_language_noninteractive_returns_fallback(tmp_path: Path) -> None:
    (tmp_path / "es.toml").write_text(ES)
    settings = tmp_path / "settings.toml"
    assert resolve_language(tmp_path, settings, interactive=False) == "es"
