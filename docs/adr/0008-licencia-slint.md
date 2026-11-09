# docs/adr/0008-licencia-slint.md
# ADR-0008: Slint bajo Royalty-free 2.0
Estado: aceptado · Fecha: 2026-11-09

## Contexto
Slint es triple licencia (GPL-3.0-only, Royalty-free 2.0, Software 3.0). El repo es MIT, de escritorio, sin ingresos.

## Decisión
Usar **Royalty-free 2.0**. El código propio sigue MIT. Condición cumplida con el widget `AboutSlint` en `md about` y el badge de Slint en el README.

## Alternativas descartadas
- GPLv3: obligaría a que el binario `md` se ofrezca bajo GPL.
- Commercial: de pago, innecesaria.

## Consecuencias
- No se puede usar en sistemas embebidos, ni distribuir Slint suelto, ni exponer sus APIs.
- La licencia no aclara si mi código puede quedar bajo MIT al distribuir; consulta pendiente a info@slint.dev antes de publicar binarios.
- Texto vinculante: `LICENSES/` de la versión de Slint que compilo.

## Reconsiderar
Si publico binarios precompilados, si Slint cambia sus términos, o si respondieron que MIT no es compatible.

Para investigar: AboutSlint, Slint attribution badge, LicenseRef, cargo-deny licenses
