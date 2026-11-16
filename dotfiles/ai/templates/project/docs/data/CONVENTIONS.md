# Convenciones de datos — __PROJECT__

Columnas base (definidas una vez, heredadas):
- id, created_at, updated_at (deleted_at solo si hay borrado lógico).

Tipos exactos de PostgreSQL; parámetros NOT NULL, DEFAULT, UNIQUE, CHECK, FK con ON DELETE, índices para consultas reales de v1.
Valores que cambian con el tiempo se copian en el registro.
Máx. 2 columnas reservadas por tabla, nulables, con razón.
