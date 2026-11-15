//! Lógica pura de los menús: parseo de las líneas de `dmenu` y filtrado.
//!
//! Aquí no hay UI: solo estructuras de datos y funciones. La vista (Slint) y la
//! orquestación (lanzar procesos, leer stdin) viven en `md-ui` y `md`.

use crate::fuzzy::score;

/// Una entrada de menú en su forma de datos. `icon` es una ruta o un nombre de
/// icono (texto); convertirlo a imagen es responsabilidad de la capa de UI.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MenuEntry {
    pub label: String,
    pub icon: String,
    pub extra: String,
}

impl MenuEntry {
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            icon: String::new(),
            extra: String::new(),
        }
    }
}

/// Cómo filtra y ordena un menú.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MenuMode {
    /// `dmenu`: respeta el orden original y solo filtra por etiqueta.
    Dmenu,
    /// Lanzador: ordena por relevancia (etiqueta y, como respaldo, `extra`).
    Launcher,
}

/// Separa una línea de `dmenu` en `(etiqueta, icono)`.
///
/// - Normal: el separador de miniatura es TAB (`etiqueta<TAB>ruta`).
/// - Raw: la línea entera es la etiqueta; opcionalmente puede llevar una
///   miniatura tras `\x1f` (`línea\x1fruta`), como hace `cliphist`.
pub fn parse_dmenu_line(line: &str, raw: bool) -> MenuEntry {
    let separator = if raw { '\x1f' } else { '\t' };
    match line.split_once(separator) {
        Some((label, icon)) => MenuEntry {
            label: label.to_owned(),
            icon: icon.to_owned(),
            extra: String::new(),
        },
        None => MenuEntry::new(line),
    }
}

/// Filtra y ordena `entries` según `query`, devolviendo los índices en el orden
/// en que deben mostrarse.
pub fn filter(entries: &[MenuEntry], query: &str, mode: MenuMode) -> Vec<usize> {
    let query = query.trim();
    if query.is_empty() {
        let mut indices: Vec<usize> = (0..entries.len()).collect();
        if mode == MenuMode::Launcher {
            // El lanzador siempre muestra alfabéticamente cuando no hay búsqueda.
            indices.sort_by(|&a, &b| {
                entries[a]
                    .label
                    .to_lowercase()
                    .cmp(&entries[b].label.to_lowercase())
                    .then(a.cmp(&b))
            });
        }
        return indices;
    }

    let query = query.to_lowercase();
    match mode {
        MenuMode::Dmenu => (0..entries.len())
            .filter(|&index| score(&query, &entries[index].label).is_some())
            .collect(),
        MenuMode::Launcher => {
            let mut hits: Vec<(u8, usize)> = Vec::new();
            for (index, entry) in entries.iter().enumerate() {
                let relevance = match score(&query, &entry.label) {
                    Some(value) => Some(value),
                    None => {
                        if entry.extra.is_empty() {
                            None
                        } else {
                            // La coincidencia por `extra` (p. ej. el nombre genérico)
                            // puntúa por debajo de cualquier coincidencia de etiqueta.
                            score(&query, &entry.extra).map(|value| value.saturating_add(4))
                        }
                    }
                };
                if let Some(value) = relevance {
                    hits.push((value, index));
                }
            }
            // (relevancia, índice original): empates resueltos por orden original.
            hits.sort_unstable();
            hits.into_iter().map(|(_, index)| index).collect()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn entry(label: &str, icon: &str, extra: &str) -> MenuEntry {
        MenuEntry {
            label: label.to_owned(),
            icon: icon.to_owned(),
            extra: extra.to_owned(),
        }
    }

    #[test]
    fn parses_plain_line() {
        assert_eq!(parse_dmenu_line("hola", false), MenuEntry::new("hola"));
    }

    #[test]
    fn parses_tab_separator() {
        assert_eq!(
            parse_dmenu_line("hola\t/ruta/img.png", false),
            entry("hola", "/ruta/img.png", "")
        );
    }

    #[test]
    fn parses_raw_with_unit_separator() {
        assert_eq!(
            parse_dmenu_line("línea completa\x1f/ruta/img.png", true),
            entry("línea completa", "/ruta/img.png", "")
        );
        // En raw sin separador, toda la línea es la etiqueta.
        assert_eq!(
            parse_dmenu_line("línea completa", true),
            MenuEntry::new("línea completa")
        );
    }

    #[test]
    fn dmenu_preserves_order_and_filters_by_label() {
        let entries = vec![
            entry("firefox", "", ""),
            entry("terminal", "", ""),
            entry("editor", "", ""),
        ];
        assert_eq!(filter(&entries, "t", MenuMode::Dmenu), vec![1, 2]);
        assert_eq!(filter(&entries, "", MenuMode::Dmenu), vec![0, 1, 2]);
    }

    #[test]
    fn launcher_ranks_by_relevance() {
        let entries = vec![
            entry("mi terminal", "", ""),
            entry("Terminal", "", ""),
            entry("intercambio", "", ""),
            entry("editor", "", "Editor de texto"),
        ];
        // "ter" empieza en "Terminal" (0), luego palabra en "mi terminal" (1),
        // luego contiene en "intercambio" (2).
        assert_eq!(filter(&entries, "ter", MenuMode::Launcher), vec![1, 0, 2]);
    }

    #[test]
    fn launcher_falls_back_to_extra() {
        let entries = vec![
            entry("código", "", ""),
            entry("navegador", "", "editor de texto"),
        ];
        // "editor" no coincide con etiquetas; cae al `extra` de "navegador".
        assert_eq!(filter(&entries, "editor", MenuMode::Launcher), vec![1]);
    }

    #[test]
    fn launcher_sorts_alphabetically_without_query() {
        let entries = vec![
            entry("zeta", "", ""),
            entry("alfa", "", ""),
            entry("medio", "", ""),
        ];
        assert_eq!(filter(&entries, "", MenuMode::Launcher), vec![1, 2, 0]);
    }
}
