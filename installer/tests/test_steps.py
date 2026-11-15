from installer.steps import discover_steps


def test_steps_are_discovered_in_order() -> None:
    orders = [step.order for step in discover_steps()]
    assert orders == sorted(orders) and orders


def test_base_steps_run_in_expected_order() -> None:
    names = [step.name for step in discover_steps()]
    index = {name: names.index(name) for name in ("sudo", "folders", "packages", "dotfiles")}
    assert index["sudo"] < index["folders"] < index["packages"] < index["dotfiles"]
