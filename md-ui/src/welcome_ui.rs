//! Puerto de alto nivel para `MdWelcome`.

use slint::ComponentHandle;

use crate::ShellHost;
use crate::generated::MdWelcome;

pub struct WelcomeConfig {
    pub title: String,
    pub changelog: Vec<String>,
    pub bienvenida_on: bool,
}

pub fn run(
    host: &impl ShellHost,
    config: WelcomeConfig,
    on_language: impl Fn() + 'static,
    on_binds: impl Fn() + 'static,
    on_github: impl Fn() + 'static,
    on_toggle: impl Fn() + 'static,
) -> Result<(), slint::PlatformError> {
    let component = MdWelcome::new()?;
    slint::set_xdg_app_id("md.welcome".to_owned())?;
    component.set_md_title(config.title.into());
    component.set_changelog(slint::VecModel::from_slice(
        &config
            .changelog
            .iter()
            .map(|s| s.clone().into())
            .collect::<Vec<slint::SharedString>>(),
    ));
    component.set_bienvenida_on(config.bienvenida_on);

    component.on_language(move || on_language());
    component.on_binds(move || on_binds());
    component.on_github(move || on_github());
    component.on_toggle(move || on_toggle());
    // exit closes window
    let weak = component.as_weak();
    component.on_exit_app(move || {
        if let Some(c) = weak.upgrade() {
            let _ = c.hide();
        }
    });

    host.run(component)
}
