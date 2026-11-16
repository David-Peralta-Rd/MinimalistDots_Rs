# Arquitectura — __PROJECT__

## Contexto y contenedores
```mermaid
graph TD
  A[Cliente] --> B[Backend]
  B --> C[(Base de datos)]
```

## Mapa de módulos
- <módulo> — base | feature — <dependencias>

## Reglas de capas y datos
- interface → application → domain; infrastructure implementa puertos.
- Cada módulo es dueño de sus tablas; referencia por id.

## Esqueleto andante
<rebanada vertical mínima>

## Evolución prevista
| Señal medible | Umbral | Siguiente paso | Qué lo habilita ya |
|---|---|---|---|

## Para investigar
- <términos exactos>
