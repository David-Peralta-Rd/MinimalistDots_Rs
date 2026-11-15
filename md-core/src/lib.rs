//! Núcleo puro de MinimalistDots: paleta, idioma y búsqueda difusa.
//!
//! Sin `unsafe`, sin E/S salvo lectura de archivos de datos, y sin `panic`:
//! todo lo fallable devuelve `Result` o `Option`.

mod clip;
mod color;
mod desktop;
mod digest;
mod fuzzy;
mod keybinds;
mod language;
mod menu;
mod palette;
mod process;
mod volume;
mod wallpaper;

pub use clip::{ClipEntry, parse_clip_line, parse_clip_list};
pub use color::{Color, ColorError};
pub use desktop::{DesktopEntry, IconIndex, data_dirs, discover_desktop_entries, exec_args};
pub use digest::{TreeDigestError, compute_tree_digest};
pub use fuzzy::{fuzzy, score};
pub use keybinds::{Keybind, parse_keybinds, to_menu_entries};
pub use language::{
    Catalog, CatalogError, CatalogReport, Translator, available_languages, base_of, load_chain,
    validate_language_catalogs,
};
pub use menu::{MenuEntry, MenuMode, filter, parse_dmenu_line};
pub use palette::{Palette, PaletteError};
pub use process::{ProcessEntry, parse_ps};
pub use volume::VolumeState;
pub use wallpaper::{Wallpaper, discover_wallpapers};

#[cfg(test)]
mod test_util {
    use std::path::{Path, PathBuf};
    use std::sync::atomic::{AtomicU64, Ordering};

    static COUNTER: AtomicU64 = AtomicU64::new(0);

    /// Directorio temporal único por prueba; se borra al soltarlo.
    pub struct TempDir(PathBuf);

    impl TempDir {
        pub fn new(name: &str) -> std::io::Result<Self> {
            let n = COUNTER.fetch_add(1, Ordering::Relaxed);
            let dir =
                std::env::temp_dir().join(format!("md_core_{name}_{}_{n}", std::process::id()));
            std::fs::create_dir_all(&dir)?;
            Ok(Self(dir))
        }

        pub fn path(&self) -> &Path {
            &self.0
        }

        pub fn write(&self, name: &str, contents: &str) -> std::io::Result<()> {
            std::fs::write(self.0.join(name), contents)
        }
    }

    impl Drop for TempDir {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }
}
