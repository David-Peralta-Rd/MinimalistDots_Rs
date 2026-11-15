//! Historial de `cliphist`: parseo de las líneas de `cliphist list`.

/// Una entrada del historial.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClipEntry {
    /// Línea completa (`id<TAB>preview`): se devuelve a `cliphist decode`/`delete`.
    pub line: String,
    /// Texto visible (`preview`, sin el id).
    pub preview: String,
}

/// Parsea una línea de `cliphist list` ("id<TAB>preview").
///
/// El `preview` puede contener tabuladores; solo se separa el primer campo.
pub fn parse_clip_line(line: &str) -> ClipEntry {
    let preview = line.split_once('\t').map_or(line, |(_, preview)| preview);
    ClipEntry {
        line: line.to_owned(),
        preview: preview.to_owned(),
    }
}

/// Parsea la salida completa de `cliphist list` (ignora líneas vacías).
pub fn parse_clip_list(output: &str) -> Vec<ClipEntry> {
    output
        .lines()
        .filter(|line| !line.is_empty())
        .map(parse_clip_line)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn splits_id_from_preview() {
        let entry = parse_clip_line("42\tHello world");
        assert_eq!(entry.line, "42\tHello world");
        assert_eq!(entry.preview, "Hello world");
    }

    #[test]
    fn keeps_line_without_tab() {
        let entry = parse_clip_line("sin tab");
        assert_eq!(entry.line, "sin tab");
        assert_eq!(entry.preview, "sin tab");
    }

    #[test]
    fn preview_may_contain_tabs() {
        let entry = parse_clip_line("7\ta\tb");
        assert_eq!(entry.preview, "a\tb");
    }

    #[test]
    fn parses_list_and_ignores_empty_lines() {
        let entries = parse_clip_list("1\ta\n2\tb\n\n");
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[1].preview, "b");
    }
}
