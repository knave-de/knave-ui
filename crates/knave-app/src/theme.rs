//! Knave's desktop theme selection and live file notifications.
use knave_ui::toolkit::{Scene, Theme};
use std::{
    path::PathBuf,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
};
use winit::event_loop::EventLoopProxy;

fn selection() -> Result<(PathBuf, PathBuf, Theme), Box<dyn std::error::Error>> {
    let mut config_path = knave_config::ConfigDocument::default_path()?;
    if config_path.is_relative() {
        config_path = std::env::current_dir()?.join(config_path);
    }
    let document = knave_config::ConfigDocument::load(&config_path)?;
    let data = knave_ui::toolkit::theme_file::data_home()?;
    let appearance = &document.config().appearance;
    let theme = if appearance.theme == knave_config::Theme::Light {
        Theme::Light
    } else {
        Theme::Dark
    };
    let css_path =
        knave_ui::toolkit::theme_file::theme_css_path(data, &appearance.theme_name, theme)?;
    Ok((config_path, css_path, theme))
}

pub fn apply(scene: &mut Scene) -> Result<(Theme, bool), Box<dyn std::error::Error>> {
    let (_, css_path, theme) = selection()?;
    let css = knave_ui::toolkit::theme_file::load_css(css_path)?.unwrap_or_default();
    let changed = scene.theme() != theme || scene.user_theme_stylesheet() != css;
    scene.set_theme_stylesheets(theme, "", &css)?;
    Ok((theme, changed))
}

/// Use the selected bundled palette when the initial custom CSS is invalid.
pub fn apply_fallback(scene: &mut Scene) -> Result<Theme, Box<dyn std::error::Error>> {
    let (_, _, theme) = selection()?;
    scene.set_theme_stylesheets(theme, "", "")?;
    Ok(theme)
}

#[cfg(target_os = "linux")]
pub struct Watcher {
    _watcher: knave_ui::toolkit::theme_file::FileWatcher,
    pub pending: Arc<AtomicBool>,
}
#[cfg(target_os = "linux")]
impl Watcher {
    pub fn start(proxy: EventLoopProxy<()>) -> Result<Self, Box<dyn std::error::Error>> {
        let pending = Arc::new(AtomicBool::new(false));
        let signal = pending.clone();
        let watcher = knave_ui::toolkit::theme_file::FileWatcher::start(
            || match selection() {
                Ok((config, css, _)) => vec![config, css],
                Err(_) => knave_config::ConfigDocument::default_path()
                    .map(|path| vec![path])
                    .unwrap_or_default(),
            },
            move || {
                if !signal.swap(true, Ordering::AcqRel) && proxy.send_event(()).is_err() {
                    signal.store(false, Ordering::Release);
                }
            },
        )?;
        Ok(Self {
            _watcher: watcher,
            pending,
        })
    }
}
