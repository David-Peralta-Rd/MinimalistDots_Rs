//! Gestor de procesos: parseo del listado de `ps`.

/// Un proceso listable en el gestor.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProcessEntry {
    pub pid: u32,
    pub label: String,
}

/// Parsea la salida de `ps -o pid=,comm=,%mem=,%cpu= --sort=-%mem` (sin cabecera).
///
/// Cada línea es `PID COMM %MEM %CPU`; `COMM` es el nombre del ejecutable (sin
/// espacios). Las líneas sin PID numérico se ignoran (p. ej. la cabecera).
pub fn parse_ps(output: &str) -> Vec<ProcessEntry> {
    output
        .lines()
        .filter_map(|line| {
            let mut fields = line.split_whitespace();
            let pid: u32 = fields.next()?.parse().ok()?;
            let comm = fields.next()?;
            let mem = fields.next().unwrap_or("");
            let cpu = fields.next().unwrap_or("");
            Some(ProcessEntry {
                pid,
                label: format!("⚙️  {comm} | RAM: {mem}% | CPU: {cpu}% (PID: {pid})"),
            })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_ps_lines() {
        let output = "  123 firefox 5.2 1.1\n  456 code 3.0 0.4\n";
        let processes = parse_ps(output);
        assert_eq!(processes.len(), 2);
        assert_eq!(processes[0].pid, 123);
        assert_eq!(
            processes[0].label,
            "⚙️  firefox | RAM: 5.2% | CPU: 1.1% (PID: 123)"
        );
        assert_eq!(processes[1].pid, 456);
    }

    #[test]
    fn ignores_header_and_malformed_lines() {
        let output = "PID COMM %MEM %CPU\n  123 firefox 5.2 1.1\n  no-pid line\n";
        let processes = parse_ps(output);
        assert_eq!(processes.len(), 1);
        assert_eq!(processes[0].pid, 123);
    }

    #[test]
    fn handles_missing_cpu_column() {
        let output = "  123 firefox 5.2\n";
        let processes = parse_ps(output);
        assert_eq!(processes.len(), 1);
        assert_eq!(
            processes[0].label,
            "⚙️  firefox | RAM: 5.2% | CPU: % (PID: 123)"
        );
    }
}
