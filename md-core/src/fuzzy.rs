//! Búsqueda difusa: replica el criterio de relevancia que usaba `MdMenu.qml`,
//! para que el lanzador y los menús Rust ordenen igual que los antiguos.

/// Puntúa la coincidencia de `query` dentro de `target` (ambas en minúsculas
/// al comparar). Menor = mejor. `None` = no coincide.
///
/// - `Some(0)` empieza igual
/// - `Some(1)` empieza una palabra (va precedido de espacio)
/// - `Some(2)` lo contiene en cualquier posición
/// - `Some(3)` coincide por subsecuencia (fuzzy)
pub fn score(query: &str, target: &str) -> Option<u8> {
    let query = query.to_lowercase();
    let target = target.to_lowercase();

    if query.is_empty() {
        return Some(0);
    }
    if target.starts_with(&query) {
        return Some(0);
    }
    if target.contains(&format!(" {query}")) {
        return Some(1);
    }
    if target.contains(&query) {
        return Some(2);
    }
    if fuzzy(&query, &target) {
        return Some(3);
    }
    None
}

/// ¿Todos los caracteres de `query` aparecen en `target`, en orden?
pub fn fuzzy(query: &str, target: &str) -> bool {
    let mut chars = target.chars();
    for wanted in query.chars() {
        if !chars.any(|candidate| candidate == wanted) {
            return false;
        }
    }
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_query_matches_best() {
        assert_eq!(score("", "cualquier cosa"), Some(0));
    }

    #[test]
    fn ranks_by_specificity() {
        assert_eq!(score("ter", "terminal"), Some(0));
        assert_eq!(score("ter", "mi terminal"), Some(1));
        assert_eq!(score("ter", "intercambio"), Some(2));
        assert_eq!(score("trm", "terminal"), Some(3)); // fuzzy
        assert_eq!(score("xyz", "terminal"), None);
    }

    #[test]
    fn is_case_insensitive() {
        assert_eq!(score("TER", "terminal"), Some(0));
        assert_eq!(score("Term", "terminal"), Some(0));
    }

    #[test]
    fn fuzzy_subsequence() {
        assert!(fuzzy("trm", "terminal"));
        assert!(!fuzzy("tmr", "terminal")); // no está en orden
        assert!(!fuzzy("zz", "terminal"));
    }
}
