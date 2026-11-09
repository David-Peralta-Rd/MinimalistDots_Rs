from installer.steps import discover_steps


def test_steps_are_discovered_in_order() -> None:
    orders = [step.order for step in discover_steps()]
    assert orders == sorted(orders) and orders
