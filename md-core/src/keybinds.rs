//! Atajos de teclado exportados por Hyprland (`keybinds.json`).

use serde::Deserialize;

use crate::menu::MenuEntry;

/// Un atajo exportado por `export_keybinds.lua`.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct Keybind {
    pub category: String,
    /// `mod` es palabra reservada de Rust: se lee del JSON con `rename`.
    #[serde(rename = "mod")]
    pub modifier: String,
    pub key: String,
    pub description: String,
    #[serde(default)]
    pub kind: String,
}

/// Parsea el contenido de `keybinds.json` (un array de objetos).
pub fn parse_keybinds(json: &str) -> Result<Vec<Keybind>, serde_json::Error> {
    serde_json::from_str(json)
}

/// Convierte los atajos en entradas de menú, agrupadas por categoría (orden
/// alfabético, como `jq group_by(.category)`) y precedidas de una cabecera.
pub fn to_menu_entries(keybinds: &[Keybind]) -> Vec<MenuEntry> {
    use std::collections::BTreeMap;

    let mut groups: BTreeMap<&str, Vec<&Keybind>> = BTreeMap::new();
    for keybind in keybinds {
        groups
            .entry(keybind.category.as_str())
            .or_default()
            .push(keybind);
    }

    let mut entries = Vec::new();
    for (category, keybinds) in groups {
        entries.push(MenuEntry::new(format!("󰌌  {category}")));
        for keybind in keybinds {
            let combo = if keybind.modifier.is_empty() {
                keybind.key.clone()
            } else {
                format!("{} + {}", keybind.modifier, keybind.key)
            };
            entries.push(MenuEntry::new(format!(
                "   {combo}  󰁔  {}",
                keybind.description
            )));
        }
    }
    entries
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_keybinds_array() -> Result<(), &'static str> {
        let json = r#"[
            { "category": "General", "mod": "SUPER", "key": "K", "description": "Abrir menú", "kind": "exec" },
            { "category": "Multimedia", "mod": "", "key": "XF86AudioMute", "description": "Silenciar", "kind": "media" }
        ]"#;
        let keybinds = parse_keybinds(json).map_err(|_| "no se pudo parsear")?;
        assert_eq!(keybinds.len(), 2);
        assert_eq!(keybinds[0].modifier, "SUPER");
        assert_eq!(keybinds[1].modifier, "");
        assert_eq!(keybinds[1].key, "XF86AudioMute");
        Ok(())
    }

    #[test]
    fn groups_by_category_alphabetically() {
        let keybinds = vec![
            Keybind {
                category: "Workspaces".to_owned(),
                modifier: "SUPER".to_owned(),
                key: "1".to_owned(),
                description: "Escritorio 1".to_owned(),
                kind: "dispatch".to_owned(),
            },
            Keybind {
                category: "General".to_owned(),
                modifier: "SUPER".to_owned(),
                key: "K".to_owned(),
                description: "Abrir menú".to_owned(),
                kind: "exec".to_owned(),
            },
            Keybind {
                category: "Multimedia".to_owned(),
                modifier: String::new(),
                key: "XF86AudioMute".to_owned(),
                description: "Silenciar".to_owned(),
                kind: "media".to_owned(),
            },
        ];
        let entries = to_menu_entries(&keybinds);
        let labels: Vec<&str> = entries.iter().map(|entry| entry.label.as_str()).collect();
        // Categorías alfabéticas: General, Multimedia, Workspaces.
        assert_eq!(
            labels,
            vec![
                "󰌌  General",
                "   SUPER + K  󰁔  Abrir menú",
                "󰌌  Multimedia",
                "   XF86AudioMute  󰁔  Silenciar",
                "󰌌  Workspaces",
                "   SUPER + 1  󰁔  Escritorio 1",
            ]
        );
    }

    #[test]
    fn omits_modifier_when_empty() {
        let keybinds = vec![Keybind {
            category: "Multimedia".to_owned(),
            modifier: String::new(),
            key: "XF86AudioRaiseVolume".to_owned(),
            description: "Subir volumen".to_owned(),
            kind: "media".to_owned(),
        }];
        let entries = to_menu_entries(&keybinds);
        assert_eq!(
            entries[1].label,
            "   XF86AudioRaiseVolume  󰁔  Subir volumen"
        );
    }
}
