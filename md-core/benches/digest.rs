use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};

use criterion::{Criterion, criterion_group, criterion_main};

static COUNTER: AtomicU64 = AtomicU64::new(0);

/// Crea un árbol de `files` archivos pequeños en un directorio temporal.
fn build_tree(files: usize) -> std::io::Result<PathBuf> {
    let n = COUNTER.fetch_add(1, Ordering::Relaxed);
    let dir = std::env::temp_dir().join(format!("md_core_bench_{}_{n}", std::process::id()));
    fs::create_dir_all(&dir)?;
    for i in 0..files {
        let content = format!("contenido de prueba {i}\n").repeat(4);
        fs::write(dir.join(format!("f{i:05}.txt")), content)?;
    }
    Ok(dir)
}

fn bench_digest(c: &mut Criterion) {
    let dir = match build_tree(1000) {
        Ok(dir) => dir,
        Err(error) => {
            eprintln!("no se pudo preparar el árbol del benchmark: {error}");
            return;
        }
    };

    let no_ignore: &[String] = &[];
    c.bench_function("compute_tree_digest/1000_files", |b| {
        b.iter(|| md_core::compute_tree_digest(&dir, no_ignore))
    });

    let _ = fs::remove_dir_all(&dir);
}

criterion_group!(benches, bench_digest);
criterion_main!(benches);
