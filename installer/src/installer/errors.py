"""Excepciones controladas del instalador."""


class StepError(Exception):
    """Error de un paso: se muestra sin traza y detiene la instalación."""


class CancelledError(Exception):
    """El usuario canceló la instalación (salida limpia, no es un error)."""
