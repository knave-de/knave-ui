//! Shared named-theme CSS path and loading helpers.
use super::Theme;
use std::{fmt, io, path::PathBuf};

const MAX_THEME_CSS_BYTES: u64 = 64 * 1024;

/// Errors loading a selected theme file. A missing file is represented as `Ok(None)`.
#[derive(Debug)]
pub enum ThemeFileError {
    InvalidName,
    Io(io::Error),
    TooLarge,
    InvalidUtf8(std::string::FromUtf8Error),
}

impl fmt::Display for ThemeFileError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidName => write!(
                f,
                "theme name must contain only ASCII letters, digits, '_' or '-'"
            ),
            Self::Io(error) => write!(f, "could not read theme CSS: {error}"),
            Self::TooLarge => write!(f, "theme CSS exceeds 64 KiB"),
            Self::InvalidUtf8(error) => write!(f, "theme CSS is not valid UTF-8: {error}"),
        }
    }
}

impl std::error::Error for ThemeFileError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Io(error) => Some(error),
            Self::InvalidUtf8(error) => Some(error),
            _ => None,
        }
    }
}

impl From<io::Error> for ThemeFileError {
    fn from(value: io::Error) -> Self {
        Self::Io(value)
    }
}

/// Resolve the data directory using XDG rules without changing process environment.
pub fn data_home() -> Result<PathBuf, ThemeFileError> {
    if let Some(value) = std::env::var_os("XDG_DATA_HOME") {
        let path = PathBuf::from(value);
        if path.is_absolute() {
            return Ok(path);
        }
    }
    let home = std::env::var_os("HOME").ok_or_else(|| {
        ThemeFileError::Io(io::Error::new(io::ErrorKind::NotFound, "HOME is not set"))
    })?;
    Ok(PathBuf::from(home).join(".local/share"))
}

/// Return the CSS path for a named theme and bundled light/dark mode.
pub fn theme_css_path(
    data_home: impl Into<PathBuf>,
    name: &str,
    theme: Theme,
) -> Result<PathBuf, ThemeFileError> {
    if name.is_empty()
        || !name
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || byte == b'_' || byte == b'-')
    {
        return Err(ThemeFileError::InvalidName);
    }
    let mode = if theme == Theme::Light {
        "light.css"
    } else {
        "dark.css"
    };
    Ok(data_home.into().join("knave/themes").join(name).join(mode))
}

/// Read selected CSS. Missing files mean no user CSS; invalid files preserve the active sheet.
pub fn load_css(path: impl AsRef<std::path::Path>) -> Result<Option<String>, ThemeFileError> {
    use std::io::Read;
    let mut file = match std::fs::File::open(path) {
        Ok(file) => file,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(None),
        Err(error) => return Err(error.into()),
    };
    let mut bytes = Vec::new();
    file.by_ref()
        .take(MAX_THEME_CSS_BYTES + 1)
        .read_to_end(&mut bytes)?;
    if bytes.len() as u64 > MAX_THEME_CSS_BYTES {
        return Err(ThemeFileError::TooLarge);
    }
    String::from_utf8(bytes)
        .map(Some)
        .map_err(ThemeFileError::InvalidUtf8)
}

/// Linux inotify watcher for config and CSS paths supplied by the host.
/// The path callback is re-run after every relevant event, so atomic config changes
/// can select a different CSS file without polling or a process-global config API.
#[cfg(target_os = "linux")]
pub struct FileWatcher {
    shutdown: rustix::fd::OwnedFd,
    thread: Option<std::thread::JoinHandle<()>>,
}

#[cfg(target_os = "linux")]
impl FileWatcher {
    pub fn start(
        paths: impl Fn() -> Vec<PathBuf> + Send + 'static,
        changed: impl Fn() + Send + 'static,
    ) -> Result<Self, String> {
        let (receiver, shutdown) = rustix::pipe::pipe_with(rustix::pipe::PipeFlags::CLOEXEC)
            .map_err(|error| error.to_string())?;
        let (ready_tx, ready_rx) = std::sync::mpsc::sync_channel(1);
        let thread = std::thread::Builder::new()
            .name("knave-theme-files".into())
            .spawn(move || watch_files(receiver, paths, changed, ready_tx))
            .map_err(|error| error.to_string())?;
        match ready_rx.recv() {
            Ok(Ok(())) => Ok(Self {
                shutdown,
                thread: Some(thread),
            }),
            Ok(Err(error)) => {
                let _ = thread.join();
                Err(error)
            }
            Err(_) => {
                let _ = thread.join();
                Err("theme file watcher stopped during startup".into())
            }
        }
    }
}

#[cfg(target_os = "linux")]
impl Drop for FileWatcher {
    fn drop(&mut self) {
        let _ = rustix::io::write(&self.shutdown, &[1]);
        if let Some(thread) = self.thread.take() {
            let _ = thread.join();
        }
    }
}

#[cfg(target_os = "linux")]
fn watch_files(
    stop: rustix::fd::OwnedFd,
    paths: impl Fn() -> Vec<PathBuf>,
    changed: impl Fn(),
    ready: std::sync::mpsc::SyncSender<Result<(), String>>,
) {
    use rustix::{
        event::{PollFd, PollFlags, poll},
        fs::inotify::{self, CreateFlags, WatchFlags},
    };
    use std::{collections::HashMap, mem::MaybeUninit, os::unix::ffi::OsStrExt};

    let mut ready = Some(ready);
    let mut dirty = false;
    loop {
        let fd = match inotify::init(CreateFlags::CLOEXEC | CreateFlags::NONBLOCK) {
            Ok(fd) => fd,
            Err(error) => {
                if let Some(ready) = ready.take() {
                    let _ = ready.send(Err(error.to_string()));
                } else {
                    eprintln!("theme file watcher: {error}");
                }
                return;
            }
        };
        let mut watches: HashMap<i32, Vec<Vec<u8>>> = HashMap::new();
        for target in paths() {
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
                let _ = ready.send(Err("no theme config or CSS paths could be watched".into()));
            } else {
                eprintln!("theme file watcher: no config or CSS paths could be watched");
            }
            return;
        }
        if let Some(ready) = ready.take() {
            let _ = ready.send(Ok(()));
        }
        if dirty {
            // Re-arm first so a write during host reload is not lost.
            changed();
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
            if !fds[0].revents().contains(PollFlags::IN) {
                continue;
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
                // Coalesce editor write bursts, then re-arm against current selection.
                std::thread::sleep(std::time::Duration::from_millis(50));
                dirty = true;
                break;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn selected_path_uses_mode_and_rejects_path_traversal() {
        let root = PathBuf::from("/private/data");
        assert_eq!(
            theme_css_path(root.clone(), "custom_theme-2", Theme::Light).unwrap(),
            root.join("knave/themes/custom_theme-2/light.css")
        );
        assert_eq!(
            theme_css_path(root.clone(), "dark", Theme::Dark).unwrap(),
            root.join("knave/themes/dark/dark.css")
        );
        assert!(matches!(
            theme_css_path(root, "../outside", Theme::Dark),
            Err(ThemeFileError::InvalidName)
        ));
    }

    #[test]
    fn loader_handles_missing_valid_oversized_and_invalid_utf8_files() {
        let dir = std::env::temp_dir().join(format!("knave-theme-file-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("theme.css");
        assert_eq!(load_css(&path).unwrap(), None);
        std::fs::write(&path, ".sample { color: #123; }").unwrap();
        assert_eq!(
            load_css(&path).unwrap().as_deref(),
            Some(".sample { color: #123; }")
        );
        std::fs::write(&path, vec![b'x'; MAX_THEME_CSS_BYTES as usize + 1]).unwrap();
        assert!(matches!(load_css(&path), Err(ThemeFileError::TooLarge)));
        std::fs::write(&path, [0xff, 0xfe]).unwrap();
        assert!(matches!(
            load_css(&path),
            Err(ThemeFileError::InvalidUtf8(_))
        ));
        std::fs::remove_dir_all(dir).unwrap();
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn watcher_rearms_after_atomic_file_replacement() {
        use std::{
            sync::{Arc, Mutex, mpsc},
            time::Duration,
        };

        let dir = std::env::temp_dir().join(format!("knave-theme-watch-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let css = dir.join("dark.css");
        std::fs::write(&css, ".sample { color: #112233; }").unwrap();
        let active_path = Arc::new(Mutex::new(css.clone()));
        let paths = active_path.clone();
        let (changed_tx, changed_rx) = mpsc::sync_channel(4);
        let watcher = FileWatcher::start(
            move || vec![paths.lock().unwrap().clone()],
            move || {
                let _ = changed_tx.try_send(());
            },
        )
        .unwrap();

        let replacement = dir.join("replacement.css");
        std::fs::write(&replacement, ".sample { color: #445566; }").unwrap();
        std::fs::rename(&replacement, &css).unwrap();
        changed_rx.recv_timeout(Duration::from_secs(3)).unwrap();
        assert_eq!(
            load_css(&css).unwrap().unwrap(),
            ".sample { color: #445566; }"
        );

        let next = dir.join("light.css");
        std::fs::write(&next, ".sample { color: #778899; }").unwrap();
        *active_path.lock().unwrap() = next.clone();
        // Trigger one more event on the currently watched file so the watcher
        // re-reads its path callback and arms the newly selected theme file.
        std::fs::write(&css, ".sample { color: #aabbcc; }").unwrap();
        changed_rx.recv_timeout(Duration::from_secs(3)).unwrap();
        std::fs::write(&next, ".sample { color: #ddeeff; }").unwrap();
        changed_rx.recv_timeout(Duration::from_secs(3)).unwrap();
        assert_eq!(
            load_css(&next).unwrap().unwrap(),
            ".sample { color: #ddeeff; }"
        );

        drop(watcher);
        std::fs::remove_dir_all(dir).unwrap();
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn watcher_pipe_is_close_on_exec() {
        use rustix::io::FdFlags;
        let (read, write) = rustix::pipe::pipe_with(rustix::pipe::PipeFlags::CLOEXEC).unwrap();
        assert!(
            rustix::io::fcntl_getfd(&read)
                .unwrap()
                .contains(FdFlags::CLOEXEC)
        );
        assert!(
            rustix::io::fcntl_getfd(&write)
                .unwrap()
                .contains(FdFlags::CLOEXEC)
        );
    }
}
