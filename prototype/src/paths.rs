use std::ffi::OsStr;
use std::path::PathBuf;

/// Resolve an XDG variable.
///
/// A relative value is invalid per the XDG Base Directory Specification and is
/// ignored, falling back to the default directory.
fn resolve_xdg(value: Option<&OsStr>, fallback: Option<PathBuf>) -> Option<PathBuf> {
    if let Some(value) = value {
        let path = PathBuf::from(value);
        if path.is_absolute() {
            return Some(path);
        }
    }
    fallback
}

fn home_dir() -> Option<PathBuf> {
    std::env::var_os("HOME").map(PathBuf::from)
}

fn config_home() -> Option<PathBuf> {
    resolve_xdg(
        std::env::var_os("XDG_CONFIG_HOME").as_deref(),
        home_dir().map(|home| home.join(".config")),
    )
}

fn data_home() -> Option<PathBuf> {
    resolve_xdg(
        std::env::var_os("XDG_DATA_HOME").as_deref(),
        home_dir().map(|home| home.join(".local/share")),
    )
}

fn cache_home() -> Option<PathBuf> {
    resolve_xdg(
        std::env::var_os("XDG_CACHE_HOME").as_deref(),
        home_dir().map(|home| home.join(".cache")),
    )
}

fn resolve_runtime(value: Option<&OsStr>) -> Option<PathBuf> {
    let path = PathBuf::from(value?);
    path.is_absolute().then_some(path)
}

/// `$XDG_RUNTIME_DIR` has no default and must be an absolute path.
pub fn runtime_dir() -> Option<PathBuf> {
    resolve_runtime(std::env::var_os("XDG_RUNTIME_DIR").as_deref())
}

pub fn app_config_dir() -> Option<PathBuf> {
    config_home().map(|base| base.join("rpci"))
}

pub fn app_data_dir() -> Option<PathBuf> {
    data_home().map(|base| base.join("rpci"))
}

pub fn app_cache_dir() -> Option<PathBuf> {
    cache_home().map(|base| base.join("rpci"))
}

pub fn app_runtime_dir() -> Option<PathBuf> {
    runtime_dir().map(|base| base.join("rpci"))
}

pub fn state_path() -> Option<PathBuf> {
    app_runtime_dir().map(|dir| dir.join("state"))
}

pub fn control_path() -> Option<PathBuf> {
    app_runtime_dir().map(|dir| dir.join("control.sock"))
}

pub fn config_path() -> Option<PathBuf> {
    app_config_dir().map(|dir| dir.join("config.toml"))
}

pub fn rime_user_dir() -> PathBuf {
    if let Some(value) = std::env::var_os("IME_RIME_USER_DIR") {
        return PathBuf::from(value);
    }
    app_data_dir()
        .unwrap_or_else(|| PathBuf::from("/tmp"))
        .join("rime")
}

pub fn exports_dir() -> Option<PathBuf> {
    app_data_dir().map(|dir| dir.join("exports"))
}

pub fn backups_dir() -> Option<PathBuf> {
    app_data_dir().map(|dir| dir.join("backups"))
}

/// Testable helper exposed for path resolution tests.
#[cfg(test)]
fn test_resolve(value: Option<&str>, fallback: Option<&str>) -> Option<PathBuf> {
    resolve_xdg(value.map(OsStr::new), fallback.map(PathBuf::from))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn absolute_xdg_value_wins() {
        assert_eq!(
            test_resolve(Some("/xdg/config"), Some("/home/u/.config")),
            Some(PathBuf::from("/xdg/config"))
        );
    }

    #[test]
    fn relative_xdg_value_is_ignored() {
        assert_eq!(
            test_resolve(Some("relative/config"), Some("/home/u/.config")),
            Some(PathBuf::from("/home/u/.config"))
        );
    }

    #[test]
    fn unset_xdg_value_uses_fallback() {
        assert_eq!(
            test_resolve(None, Some("/home/u/.config")),
            Some(PathBuf::from("/home/u/.config"))
        );
    }

    #[test]
    fn runtime_dir_requires_absolute_path() {
        assert_eq!(
            resolve_runtime(Some(OsStr::new("/run/user/1000"))),
            Some(PathBuf::from("/run/user/1000"))
        );
        assert_eq!(resolve_runtime(Some(OsStr::new("relative"))), None);
        assert_eq!(resolve_runtime(None), None);
    }
}
