use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

pub const KEEP_BACKUPS: usize = 5;

#[derive(Debug, Clone)]
pub struct Paths {
    pub rime: PathBuf,
    pub config: Option<PathBuf>,
    pub backups: PathBuf,
}

pub fn default_paths() -> Paths {
    Paths {
        rime: crate::paths::rime_user_dir(),
        config: crate::paths::config_path(),
        backups: crate::paths::backups_dir().unwrap_or_else(|| PathBuf::from("/tmp/rpci/backups")),
    }
}

pub fn export_to(paths: &Paths, destination: &Path) -> io::Result<()> {
    if !paths.rime.is_dir() {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            format!("RIME user data not found: {}", paths.rime.display()),
        ));
    }
    fs::create_dir_all(destination)?;
    copy_tree(&paths.rime, &destination.join("rime"), true)?;

    if let Some(config) = &paths.config {
        if config.is_file() {
            fs::copy(config, destination.join("config.toml"))?;
        }
    }

    let manifest = format!(
        "version=1\nexported_at={}\nsource={}\n",
        now_secs(),
        paths.rime.display()
    );
    fs::write(destination.join("manifest.txt"), manifest)?;
    Ok(())
}

pub fn import_from(paths: &Paths, source: &Path) -> io::Result<()> {
    let source_rime = source.join("rime");
    if !source_rime.is_dir() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            format!("not a backup directory: {}", source.display()),
        ));
    }

    // Safety net: keep the current state before overwriting it.
    if paths.rime.is_dir() {
        let _ = backup_now(paths, KEEP_BACKUPS);
    }

    fs::create_dir_all(&paths.rime)?;
    copy_tree(&source_rime, &paths.rime, false)?;

    let source_config = source.join("config.toml");
    if source_config.is_file() {
        if let Some(config) = &paths.config {
            if let Some(parent) = config.parent() {
                fs::create_dir_all(parent)?;
            }
            fs::copy(source_config, config)?;
        }
    }
    Ok(())
}

pub fn backup_now(paths: &Paths, keep: usize) -> io::Result<PathBuf> {
    fs::create_dir_all(&paths.backups)?;
    let destination = unique_destination(&paths.backups)?;
    export_to(paths, &destination)?;
    prune(&paths.backups, keep)?;
    Ok(destination)
}

pub fn list_backups(paths: &Paths) -> io::Result<Vec<PathBuf>> {
    if !paths.backups.is_dir() {
        return Ok(Vec::new());
    }
    let mut backups = Vec::new();
    for entry in fs::read_dir(&paths.backups)? {
        let entry = entry?;
        if entry.file_type()?.is_dir() {
            backups.push(entry.path());
        }
    }
    backups.sort();
    Ok(backups)
}

fn unique_destination(root: &Path) -> io::Result<PathBuf> {
    let base = now_secs().to_string();
    for index in 0..1000 {
        let candidate = if index == 0 {
            root.join(&base)
        } else {
            root.join(format!("{base}-{index}"))
        };
        if !candidate.exists() {
            return Ok(candidate);
        }
    }
    Err(io::Error::new(
        io::ErrorKind::AlreadyExists,
        "too many backups in the same second",
    ))
}

fn prune(root: &Path, keep: usize) -> io::Result<()> {
    let mut backups = Vec::new();
    for entry in fs::read_dir(root)? {
        let entry = entry?;
        if entry.file_type()?.is_dir() {
            backups.push(entry.path());
        }
    }
    backups.sort();
    while backups.len() > keep {
        let oldest = backups.remove(0);
        fs::remove_dir_all(oldest)?;
    }
    Ok(())
}

fn copy_tree(source: &Path, destination: &Path, skip_cache: bool) -> io::Result<()> {
    fs::create_dir_all(destination)?;
    for entry in fs::read_dir(source)? {
        let entry = entry?;
        let name = entry.file_name();
        let name_str = name.to_string_lossy();

        if skip_cache && matches!(name_str.as_ref(), "build" | "log" | "sync" | "trash") {
            continue;
        }
        if name_str == "LOCK" || name_str.ends_with(".lock") {
            continue;
        }

        let file_type = entry.file_type()?;
        if file_type.is_symlink() {
            continue;
        }
        let target = destination.join(&name);
        if file_type.is_dir() {
            copy_tree(&entry.path(), &target, skip_cache)?;
        } else if file_type.is_file() {
            fs::copy(entry.path(), &target)?;
        }
    }
    Ok(())
}

fn now_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn scratch(name: &str) -> PathBuf {
        let path =
            std::env::temp_dir().join(format!("rpci-backup-test-{name}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&path);
        fs::create_dir_all(&path).unwrap();
        path
    }

    fn sample_paths(root: &Path) -> Paths {
        let rime = root.join("rime");
        fs::create_dir_all(rime.join("wubi86.userdb")).unwrap();
        fs::create_dir_all(rime.join("build")).unwrap();
        fs::create_dir_all(rime.join("log")).unwrap();
        fs::write(rime.join("user.yaml"), "var: 1\n").unwrap();
        fs::write(rime.join("wubi86.userdb/CURRENT"), "MANIFEST\n").unwrap();
        fs::write(rime.join("build/cache.bin"), "cache").unwrap();
        fs::write(rime.join("log/rime.log"), "log").unwrap();
        fs::write(rime.join("LOCK"), "lock").unwrap();

        let config = root.join("config.toml");
        fs::write(&config, "theme = \"light\"\n").unwrap();

        Paths {
            rime,
            config: Some(config),
            backups: root.join("backups"),
        }
    }

    #[test]
    fn export_skips_cache_and_locks() {
        let root = scratch("export");
        let paths = sample_paths(&root);
        let destination = root.join("export");
        export_to(&paths, &destination).unwrap();

        assert!(destination.join("rime/user.yaml").is_file());
        assert!(destination.join("rime/wubi86.userdb/CURRENT").is_file());
        assert!(destination.join("config.toml").is_file());
        assert!(destination.join("manifest.txt").is_file());
        assert!(!destination.join("rime/build").exists());
        assert!(!destination.join("rime/log").exists());
        assert!(!destination.join("rime/LOCK").exists());
    }

    #[test]
    fn backup_prunes_to_keep() {
        let root = scratch("prune");
        let paths = sample_paths(&root);
        for _ in 0..3 {
            backup_now(&paths, 2).unwrap();
        }
        let backups = list_backups(&paths).unwrap();
        assert_eq!(backups.len(), 2);
    }

    #[test]
    fn import_restores_export() {
        let root = scratch("import");
        let paths = sample_paths(&root);
        let exported = root.join("export");
        export_to(&paths, &exported).unwrap();

        let target = root.join("target");
        let target_paths = Paths {
            rime: target.join("rime"),
            config: Some(target.join("config.toml")),
            backups: target.join("backups"),
        };
        import_from(&target_paths, &exported).unwrap();

        assert!(target_paths.rime.join("user.yaml").is_file());
        assert!(target_paths.rime.join("wubi86.userdb/CURRENT").is_file());
        assert_eq!(
            fs::read_to_string(target_paths.config.unwrap()).unwrap(),
            "theme = \"light\"\n"
        );
    }

    #[test]
    fn backup_names_are_unique_within_same_second() {
        let root = scratch("unique");
        let paths = sample_paths(&root);
        let first = backup_now(&paths, 10).unwrap();
        let second = backup_now(&paths, 10).unwrap();
        assert_ne!(first, second);
    }
}
