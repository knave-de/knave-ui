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
    let data = match std::env::var_os("XDG_DATA_HOME") {
        Some(value) if PathBuf::from(&value).is_absolute() => PathBuf::from(value),
        _ => PathBuf::from(std::env::var_os("HOME").ok_or("HOME is not set")?).join(".local/share"),
    };
    let appearance = &document.config().appearance;
    let theme = if appearance.theme == knave_config::Theme::Light {
        Theme::Light
    } else {
        Theme::Dark
    };
    let variant = if theme == Theme::Light {
        "light.css"
    } else {
        "dark.css"
    };
    Ok((
        config_path,
        data.join("knave/themes")
            .join(&appearance.theme_name)
            .join(variant),
        theme,
    ))
}

pub fn apply(scene: &mut Scene) -> Result<(Theme, bool), Box<dyn std::error::Error>> {
    use std::io::Read;
    let (_, css_path, theme) = selection()?;
    let css = match std::fs::File::open(css_path) {
        Ok(file) => {
            let mut bytes = Vec::new();
            file.take(1024 * 1024 + 1).read_to_end(&mut bytes)?;
            if bytes.len() > 1024 * 1024 {
                return Err("theme CSS exceeds 1 MiB".into());
            }
            String::from_utf8(bytes)?
        }
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => String::new(),
        Err(e) => return Err(e.into()),
    };
    let changed = scene.theme() != theme || scene.desktop_stylesheet() != css;
    scene.set_desktop_theme(theme, &css)?;
    Ok((theme, changed))
}

/// Use the selected bundled palette when the initial custom CSS is invalid.
pub fn apply_fallback(scene: &mut Scene) -> Result<Theme, Box<dyn std::error::Error>> {
    let (_, _, theme) = selection()?;
    scene.set_desktop_theme(theme, "")?;
    Ok(theme)
}

#[cfg(target_os = "linux")]
pub struct Watcher {
    shutdown: std::os::unix::net::UnixStream,
    thread: Option<std::thread::JoinHandle<()>>,
    pub pending: Arc<AtomicBool>,
}
#[cfg(target_os = "linux")]
impl Watcher {
    pub fn start(proxy: EventLoopProxy<()>) -> Result<Self, Box<dyn std::error::Error>> {
        use std::os::unix::net::UnixStream;
        let (shutdown, receiver) = UnixStream::pair()?;
        let pending = Arc::new(AtomicBool::new(false));
        let signal = pending.clone();
        let (ready_tx, ready_rx) = std::sync::mpsc::sync_channel(1);
        let thread = std::thread::Builder::new()
            .name("knave-theme".into())
            .spawn(move || watch(receiver, proxy, signal, ready_tx))?;
        ready_rx
            .recv()
            .map_err(|_| "theme watcher stopped before startup")??;
        Ok(Self {
            shutdown,
            thread: Some(thread),
            pending,
        })
    }
}
#[cfg(target_os = "linux")]
impl Drop for Watcher {
    fn drop(&mut self) {
        use std::io::Write;
        let _ = self.shutdown.write_all(&[1]);
        if let Some(thread) = self.thread.take() {
            let _ = thread.join();
        }
    }
}
#[cfg(target_os = "linux")]
fn watch(
    stop: std::os::unix::net::UnixStream,
    proxy: EventLoopProxy<()>,
    pending: Arc<AtomicBool>,
    ready: std::sync::mpsc::SyncSender<Result<(), String>>,
) {
    use rustix::{
        event::{PollFd, PollFlags, poll},
        fs::inotify::{self, CreateFlags, WatchFlags},
    };
    use std::{collections::HashMap, mem::MaybeUninit, os::unix::ffi::OsStrExt};
    let mut ready = Some(ready);
    let mut changed = false;
    loop {
        let fd = match inotify::init(CreateFlags::CLOEXEC | CreateFlags::NONBLOCK) {
            Ok(fd) => fd,
            Err(error) => {
                if let Some(ready) = ready.take() {
                    let _ = ready.send(Err(error.to_string()));
                } else {
                    eprintln!("theme watcher: {error}");
                }
                return;
            }
        };
        let mut watches: HashMap<i32, Vec<Vec<u8>>> = HashMap::new();
        let targets = match selection() {
            Ok((config, css, _)) => vec![config, css],
            Err(_) => match knave_config::ConfigDocument::default_path() {
                Ok(config) => vec![config],
                Err(_) => return,
            },
        };
        for target in targets {
            let mut child = target.as_path();
            while let (Some(parent), Some(name)) = (child.parent(), child.file_name()) {
                if parent.is_dir()
                    && let Ok(wd) = inotify::add_watch(
                        &fd,
                        parent,
                        WatchFlags::CREATE
                            | WatchFlags::DELETE
                            | WatchFlags::MOVED_FROM
                            | WatchFlags::MOVED_TO
                            | WatchFlags::CLOSE_WRITE
                            | WatchFlags::ATTRIB
                            | WatchFlags::DELETE_SELF
                            | WatchFlags::MOVE_SELF,
                    )
                {
                    watches
                        .entry(wd)
                        .or_default()
                        .push(name.as_bytes().to_vec());
                }
                child = parent;
            }
        }
        if watches.is_empty() {
            if let Some(ready) = ready.take() {
                let _ = ready.send(Err("no theme paths could be watched".into()));
            } else {
                eprintln!("theme watcher: no paths could be watched");
            }
            return;
        }
        if let Some(ready) = ready.take() {
            let _ = ready.send(Ok(()));
        }
        if changed && !pending.swap(true, Ordering::AcqRel) && proxy.send_event(()).is_err() {
            return;
        }
        let mut fds = [
            PollFd::new(&fd, PollFlags::IN),
            PollFd::new(&stop, PollFlags::IN),
        ];
        loop {
            if poll(&mut fds, None).is_err() {
                return;
            }
            if fds[1].revents().contains(PollFlags::IN) {
                return;
            }
            let mut buf = [MaybeUninit::uninit(); 8192];
            let mut reader = inotify::Reader::new(&fd, &mut buf);
            let mut relevant = false;
            while let Ok(event) = reader.next() {
                relevant |= match event.file_name() {
                    Some(name) => watches
                        .get(&event.wd())
                        .is_some_and(|names| names.iter().any(|n| n.as_slice() == name.to_bytes())),
                    None => true,
                };
            }
            if relevant {
                // Coalesce editor write bursts; idle watchers do not wake on a timer.
                std::thread::sleep(std::time::Duration::from_millis(50));
                changed = true;
                break;
            }
        }
    }
}
