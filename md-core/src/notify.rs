//! Notificaciones: tipos, historial (jsonl) y utilidades puras.
//!
//! Aquí no hay DBus ni E/S de archivos: solo estructuras, parseo y formato. El
//! servidor (zbus) y la lectura/escritura del historial viven en `md`.

use serde::{Deserialize, Serialize};

/// Urgencia de una notificación (espejo del byte de la especificación freedesktop).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Urgency {
    Low,
    Normal,
    Critical,
}

impl Urgency {
    /// Mapea el byte DBus (`0` low, `1` normal, `2` critical) a la urgencia.
    /// Cualquier valor fuera de rango se trata como normal.
    pub fn from_dbus(byte: u8) -> Self {
        match byte {
            0 => Self::Low,
            2 => Self::Critical,
            _ => Self::Normal,
        }
    }

    /// Clave de idioma / nombre canónico ("low", "normal", "critical").
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Low => "low",
            Self::Normal => "normal",
            Self::Critical => "critical",
        }
    }

    pub fn is_critical(self) -> bool {
        self == Self::Critical
    }
}

/// Una notificación tal y como se guarda en el historial (`notifications.jsonl`).
///
/// `time` son segundos Unix. El campo `app` se conserva en mayúsculas para pintarlo
/// (igual que el demonio original), así que se guarda tal cual.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Notification {
    pub time: i64,
    pub app: String,
    pub summary: String,
    pub body: String,
    pub urgency: Urgency,
}

impl Notification {
    /// Serializa a una línea de historial (sin salto de línea final). Una línea
    /// vacía significa que la serialización falló (no debería ocurrir).
    pub fn to_json_line(&self) -> String {
        serde_json::to_string(self).unwrap_or_default()
    }

    /// Parsea una línea de historial; `None` si está dañada (se ignora).
    pub fn from_json_line(line: &str) -> Option<Self> {
        serde_json::from_str(line).ok()
    }
}

/// Una acción de notificación (par `id`, `etiqueta`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Action {
    pub id: String,
    pub label: String,
}

/// Parsea el vector de acciones de DBus (`["id1", "etiqueta1", "id2", "etiqueta2", …]`).
/// Las entradas impares sobrantes se descartan.
pub fn parse_actions(actions: &[String]) -> Vec<Action> {
    actions
        .as_chunks::<2>()
        .0
        .iter()
        .map(|pair| Action {
            id: pair[0].clone(),
            label: pair[1].clone(),
        })
        .collect()
}

/// Convierte segundos Unix + desplazamiento local a "dd/mm HH:MM".
///
/// `utc_offset_seconds` es el desfase local respecto a UTC (positivo al este),
/// como el que se lee de `date +%z`.
pub fn format_timestamp(unix: i64, utc_offset_seconds: i64) -> String {
    let local = unix + utc_offset_seconds;
    let days = local.div_euclid(86_400);
    let secs = local.rem_euclid(86_400);
    let (_year, month, day) = civil_from_days(days);
    let hour = secs / 3600;
    let minute = (secs % 3600) / 60;
    format!("{day:02}/{month:02} {hour:02}:{minute:02}")
}

/// Parsea un desfase UTC en formato `+HHMM`/`-HHMM` a segundos.
pub fn parse_utc_offset(offset: &str) -> Option<i64> {
    let bytes = offset.as_bytes();
    if bytes.len() != 5 {
        return None;
    }
    let sign = match bytes[0] {
        b'+' => 1,
        b'-' => -1,
        _ => return None,
    };
    let hours: i64 = offset[1..3].parse().ok()?;
    let minutes: i64 = offset[3..5].parse().ok()?;
    if minutes >= 60 {
        return None;
    }
    Some(sign * (hours * 3600 + minutes * 60))
}

/// Formatea una entrada del historial para el menú, igual que el `jq` original:
/// `#N  dd/mm HH:MM  [APP] RESUMEN — CUERPO`, recortado a `max_chars`.
pub fn format_menu_line(
    notification: &Notification,
    index: usize,
    time_label: &str,
    max_chars: usize,
) -> String {
    let mut line = format!(
        "#{}  {}  [{}] {}",
        index + 1,
        time_label,
        notification.app,
        notification.summary
    );
    if !notification.body.is_empty() {
        let body = notification.body.replace(['\n', '\t'], " ");
        line.push_str(&format!(" — {body}"));
    }
    line.chars().take(max_chars).collect()
}

/// Algoritmo de Howard Hinnant: días desde la época civil (1970-01-01) a
/// `(año, mes, día)`.
fn civil_from_days(z: i64) -> (i64, u32, u32) {
    let z = z + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let day = (doy - (153 * mp + 2) / 5 + 1) as u32;
    let month = if mp < 10 { mp + 3 } else { mp - 9 } as u32;
    (y + i64::from(month <= 2), month, day)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn notification(body: &str) -> Notification {
        Notification {
            time: 0,
            app: "App".to_owned(),
            summary: "Resumen".to_owned(),
            body: body.to_owned(),
            urgency: Urgency::Normal,
        }
    }

    #[test]
    fn maps_dbus_urgency() {
        assert_eq!(Urgency::from_dbus(0), Urgency::Low);
        assert_eq!(Urgency::from_dbus(1), Urgency::Normal);
        assert_eq!(Urgency::from_dbus(2), Urgency::Critical);
        assert_eq!(Urgency::from_dbus(7), Urgency::Normal);
    }

    #[test]
    fn urgency_roundtrips_through_json() -> Result<(), &'static str> {
        for urgency in [Urgency::Low, Urgency::Normal, Urgency::Critical] {
            let json = serde_json::to_string(&urgency).map_err(|_| "serializar")?;
            let back: Urgency = serde_json::from_str(&json).map_err(|_| "deserializar")?;
            assert_eq!(back, urgency);
        }
        assert_eq!(Urgency::Critical.as_str(), "critical");
        assert!(Urgency::Critical.is_critical());
        assert!(!Urgency::Normal.is_critical());
        Ok(())
    }

    #[test]
    fn notification_roundtrips_through_json_line() {
        let n = Notification {
            time: 1_700_000_000,
            app: "Spotify".to_owned(),
            summary: "Canción".to_owned(),
            body: "Artista — Álbum".to_owned(),
            urgency: Urgency::Critical,
        };
        let line = n.to_json_line();
        assert_eq!(Notification::from_json_line(&line), Some(n));
    }

    #[test]
    fn ignores_corrupt_lines() {
        assert_eq!(Notification::from_json_line("no es json"), None);
        assert_eq!(Notification::from_json_line(""), None);
    }

    #[test]
    fn parses_actions_in_pairs() {
        let actions = parse_actions(&[
            "default".to_owned(),
            "Abrir".to_owned(),
            "reply".to_owned(),
            "Responder".to_owned(),
        ]);
        assert_eq!(
            actions,
            vec![
                Action {
                    id: "default".to_owned(),
                    label: "Abrir".to_owned()
                },
                Action {
                    id: "reply".to_owned(),
                    label: "Responder".to_owned()
                },
            ]
        );
    }

    #[test]
    fn parses_actions_discards_odd_tail() {
        let actions = parse_actions(&["a".to_owned(), "b".to_owned(), "suelto".to_owned()]);
        assert_eq!(actions.len(), 1);
        assert_eq!(actions[0].label, "b");
    }

    #[test]
    fn formats_timestamp_from_unix() {
        // 2024-01-01 00:00:00 UTC = 1_704_067_200.
        assert_eq!(format_timestamp(1_704_067_200, 0), "01/01 00:00");
        // Con +1h de desfase local.
        assert_eq!(format_timestamp(1_704_067_200, 3600), "01/01 01:00");
    }

    #[test]
    fn parses_utc_offsets() {
        assert_eq!(parse_utc_offset("+0100"), Some(3600));
        assert_eq!(parse_utc_offset("-0500"), Some(-5 * 3600));
        assert_eq!(parse_utc_offset("+0000"), Some(0));
        assert_eq!(parse_utc_offset("nada"), None);
        assert_eq!(parse_utc_offset("+0199"), None);
    }

    #[test]
    fn formats_menu_line_with_and_without_body() {
        let line = format_menu_line(&notification("Hola"), 0, "01/01 10:30", 220);
        assert_eq!(line, "#1  01/01 10:30  [App] Resumen — Hola");

        let no_body = format_menu_line(&notification(""), 3, "02/02 08:00", 220);
        assert_eq!(no_body, "#4  02/02 08:00  [App] Resumen");
    }

    #[test]
    fn formats_menu_line_replaces_newlines_and_truncates() {
        let n = notification("línea1\nlínea2\tcolumna");
        let line = format_menu_line(&n, 0, "03/03 09:00", 44);
        assert_eq!(line, "#1  03/03 09:00  [App] Resumen — línea1 líne");
    }
}
