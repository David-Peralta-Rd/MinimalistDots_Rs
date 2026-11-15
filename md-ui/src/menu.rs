//! Puerto de alto nivel para el widget `MdMenu`: configura la ventana, conecta
//! los callbacks y devuelve lo que hizo el usuario.
//!
//! El filtrado/orden viven en `md_core::menu`; aquí solo se cargan las imágenes
//! y se rellena el modelo que pinta Slint. Los índices que emite el widget son
//! posiciones dentro de la lista *filtrada*; aquí se traducen al índice original
//! de la lista de entradas.

use std::cell::RefCell;
use std::rc::Rc;

use md_core::{MenuEntry, MenuMode};
use slint::ComponentHandle;

use crate::generated::{MdMenu, MenuItem};
use crate::{ShellHost, slint_color};

/// Configuración de una instancia del menú.
#[derive(Debug, Clone)]
pub struct MenuConfig {
    /// Identificador de ventana (app-id) para las windowrules de Hyprland.
    pub app_id: String,
    pub title: String,
    pub prompt: String,
    pub accent: md_core::Color,
    pub row_height: f32,
    pub width: f32,
    pub height: f32,
    pub can_delete: bool,
    pub can_wipe: bool,
    /// Etiqueta del botón "borrar todo" (Slint). Si es `None`, usa el texto por defecto.
    pub wipe_label: Option<String>,
    /// Etiqueta de confirmación del borrado total. Si es `None`, usa el texto por defecto.
    pub confirm_wipe_label: Option<String>,
    /// Si es `true`, Enter con la lista vacía devuelve el texto tecleado
    /// (`MenuEvent::Typed`). Útil para `dmenu` sin `--raw`.
    pub submit_empty: bool,
}

impl Default for MenuConfig {
    fn default() -> Self {
        Self {
            app_id: "md.menu".to_owned(),
            title: "md".to_owned(),
            prompt: String::new(),
            accent: md_core::Color::new(0x7e, 0xc7, 0xa2),
            row_height: 40.0,
            width: 600.0,
            height: 400.0,
            can_delete: false,
            can_wipe: false,
            wipe_label: None,
            confirm_wipe_label: None,
            submit_empty: false,
        }
    }
}

/// Lo que pasó al cerrar el menú. Los índices son sobre la lista *original*.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MenuEvent {
    /// Se eligió la entrada `index`.
    Selected(usize),
    /// Se pulsó Enter con la lista vacía: el usuario tecleó `query`.
    Typed(String),
    /// Se canceló con Esc.
    Cancelled,
    /// Se pidió borrar la entrada `index`.
    Delete(usize),
    /// Se pidió borrar todo.
    Wipe,
}

struct State {
    entries: Vec<MenuEntry>,
    all_items: Vec<MenuItem>,
    /// Índices originales que están visibles en este momento.
    filtered: Vec<usize>,
    query: String,
    outcome: Option<MenuEvent>,
}

/// Convierte una entrada de datos a la fila que pinta Slint (cargando el icono).
fn to_menu_item(entry: &MenuEntry) -> MenuItem {
    let icon = if entry.icon.is_empty() {
        slint::Image::default()
    } else {
        slint::Image::load_from_path(std::path::Path::new(&entry.icon)).unwrap_or_default()
    };
    MenuItem {
        label: entry.label.clone().into(),
        icon,
    }
}

/// Proyecta los índices filtrados sobre las filas ya cargadas.
fn build_shown(all_items: &[MenuItem], indices: &[usize]) -> Vec<MenuItem> {
    indices
        .iter()
        .map(|&index| all_items[index].clone())
        .collect()
}

/// Traduce un índice de la lista filtrada al índice original; `None` si se sale
/// de rango (no debería ocurrir).
fn original_index(filtered: &[usize], shown_index: i32) -> Option<usize> {
    usize::try_from(shown_index)
        .ok()
        .and_then(|index| filtered.get(index).copied())
}

/// Ejecuta el menú y devuelve la elección del usuario.
pub fn run(
    host: &impl ShellHost,
    entries: &[MenuEntry],
    mode: MenuMode,
    config: &MenuConfig,
) -> Result<MenuEvent, slint::PlatformError> {
    let component = MdMenu::new()?;
    // `MdMenu::new()` inicializa el backend, así que el app-id ya se puede fijar
    // antes de mostrar la ventana (lo necesitan las windowrules de Hyprland).
    slint::set_xdg_app_id(config.app_id.clone())?;
    component.set_md_title(config.title.clone().into());
    component.set_prompt(config.prompt.clone().into());
    component.set_accent(slint_color(config.accent));
    component.set_row_height(config.row_height);
    component.set_md_width(config.width);
    component.set_md_height(config.height);
    component.set_can_delete(config.can_delete);
    component.set_can_wipe(config.can_wipe);
    if let Some(label) = &config.wipe_label {
        component.set_wipe_label(label.clone().into());
    }
    if let Some(label) = &config.confirm_wipe_label {
        component.set_confirm_wipe_label(label.clone().into());
    }
    component.set_submit_empty(config.submit_empty);

    let state = Rc::new(RefCell::new(State {
        entries: entries.to_vec(),
        all_items: entries.iter().map(to_menu_item).collect(),
        filtered: Vec::new(),
        query: String::new(),
        outcome: None,
    }));

    // Filtrado inicial (consulta vacía) y modelo de partida.
    {
        let mut state = state.borrow_mut();
        state.filtered = md_core::filter(&state.entries, "", mode);
        component.set_shown(slint::VecModel::from_slice(&build_shown(
            &state.all_items,
            &state.filtered,
        )));
    }

    let weak = component.as_weak();

    component.on_query_changed({
        let state = state.clone();
        let weak = weak.clone();
        move |query: slint::SharedString| {
            let shown = {
                let mut state = state.borrow_mut();
                state.query = query.to_string();
                state.filtered = md_core::filter(&state.entries, &state.query, mode);
                build_shown(&state.all_items, &state.filtered)
            };
            if let Some(component) = weak.upgrade() {
                component.set_shown(slint::VecModel::from_slice(&shown));
                component.set_current_index(0);
            }
        }
    });

    component.on_activated({
        let state = state.clone();
        let weak = weak.clone();
        move |index: i32| {
            let outcome = {
                let state = state.borrow();
                if index < 0 {
                    MenuEvent::Typed(state.query.clone())
                } else {
                    original_index(&state.filtered, index)
                        .map(MenuEvent::Selected)
                        .unwrap_or(MenuEvent::Cancelled)
                }
            };
            finish(&state, &weak, outcome);
        }
    });

    component.on_cancelled({
        let state = state.clone();
        let weak = weak.clone();
        move || finish(&state, &weak, MenuEvent::Cancelled)
    });

    component.on_delete_requested({
        let state = state.clone();
        let weak = weak.clone();
        move |index: i32| {
            let outcome = original_index(&state.borrow().filtered, index)
                .map(MenuEvent::Delete)
                .unwrap_or(MenuEvent::Cancelled);
            finish(&state, &weak, outcome);
        }
    });

    component.on_wipe_requested({
        let state = state.clone();
        let weak = weak.clone();
        move || finish(&state, &weak, MenuEvent::Wipe)
    });

    host.run(component)?;

    Ok(state
        .borrow()
        .outcome
        .clone()
        .unwrap_or(MenuEvent::Cancelled))
}

/// Guarda el resultado y cierra la ventana.
fn finish(state: &Rc<RefCell<State>>, weak: &slint::Weak<MdMenu>, outcome: MenuEvent) {
    state.borrow_mut().outcome = Some(outcome);
    if let Some(component) = weak.upgrade() {
        let _ = component.hide();
    }
}
