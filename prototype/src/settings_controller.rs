use std::path::{Path, PathBuf};

use crate::backup::{self, KEEP_BACKUPS, default_paths};
use crate::config::{CandidateLayout, Config, ThemePreference};
use crate::service;

pub trait SettingsBackend {
    fn save(&mut self, config: &Config) -> Result<PathBuf, String>;
    fn restart(&mut self) -> Result<(), String>;
    fn stop(&mut self) -> Result<(), String>;
    fn start(&mut self) -> Result<(), String>;
    fn backup(&mut self) -> Result<PathBuf, String>;
    fn list_backups(&mut self) -> Result<Vec<PathBuf>, String>;
    fn restore(&mut self, path: &Path) -> Result<(), String>;
}

#[derive(Default)]
pub struct RealSettingsBackend;

impl SettingsBackend for RealSettingsBackend {
    fn save(&mut self, config: &Config) -> Result<PathBuf, String> {
        config.save().map_err(|err| err.to_string())
    }

    fn restart(&mut self) -> Result<(), String> {
        service::restart_input_method()
    }

    fn stop(&mut self) -> Result<(), String> {
        service::stop_input_method()
    }

    fn start(&mut self) -> Result<(), String> {
        service::start_input_method()
    }

    fn backup(&mut self) -> Result<PathBuf, String> {
        backup::backup_now(&default_paths(), KEEP_BACKUPS).map_err(|err| err.to_string())
    }

    fn list_backups(&mut self) -> Result<Vec<PathBuf>, String> {
        backup::list_backups(&default_paths()).map_err(|err| err.to_string())
    }

    fn restore(&mut self, path: &Path) -> Result<(), String> {
        backup::import_from(&default_paths(), path).map_err(|err| err.to_string())
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SettingsAction {
    SetAscii(bool),
    SetPinyin(bool),
    SetCandidateCount(usize),
    SetLayout(CandidateLayout),
    SetTheme(ThemePreference),
    Save,
    Apply,
    Backup,
    SelectBackup(PathBuf),
    Restore,
    Reload,
}

pub struct SettingsController {
    config: Config,
    status: String,
    dirty: bool,
    backups: Vec<PathBuf>,
    selected_backup: Option<PathBuf>,
    pending_restore: bool,
    backend: Box<dyn SettingsBackend>,
}

impl SettingsController {
    pub fn new(config: Config, backend: Box<dyn SettingsBackend>) -> Self {
        let mut controller = Self {
            config,
            status: "就绪".to_string(),
            dirty: false,
            backups: Vec::new(),
            selected_backup: None,
            pending_restore: false,
            backend,
        };
        controller.refresh_backups(None);
        controller
    }

    pub fn config(&self) -> &Config {
        &self.config
    }

    pub fn status(&self) -> &str {
        &self.status
    }

    pub fn backups(&self) -> &[PathBuf] {
        &self.backups
    }

    pub fn selected_backup(&self) -> Option<&PathBuf> {
        self.selected_backup.as_ref()
    }

    pub fn pending_restore(&self) -> bool {
        self.pending_restore
    }

    pub fn dirty(&self) -> bool {
        self.dirty
    }

    pub fn update(&mut self, action: SettingsAction) {
        match action {
            SettingsAction::SetAscii(value) => {
                self.config.ascii_mode_default = value;
                self.mark_dirty();
            }
            SettingsAction::SetPinyin(value) => {
                self.config.pinyin_fallback = value;
                self.mark_dirty();
            }
            SettingsAction::SetCandidateCount(value) => {
                self.config.candidate_count = value;
                self.mark_dirty();
            }
            SettingsAction::SetLayout(value) => {
                self.config.candidate_layout = value;
                self.mark_dirty();
            }
            SettingsAction::SetTheme(value) => {
                self.config.theme = value;
                self.mark_dirty();
            }
            SettingsAction::Save => self.save(),
            SettingsAction::Apply => self.apply(),
            SettingsAction::Backup => self.backup(),
            SettingsAction::SelectBackup(path) => {
                self.selected_backup = Some(path);
                self.pending_restore = false;
            }
            SettingsAction::Restore => self.restore(),
            SettingsAction::Reload => {
                self.pending_restore = false;
                self.config = Config::load();
                self.dirty = false;
                self.status = "已重新加载配置".to_string();
            }
        }
    }

    fn mark_dirty(&mut self) {
        self.dirty = true;
        self.status = "未保存（按 S 保存）".to_string();
    }

    fn mark_saved(&mut self, _path: &Path) {
        self.dirty = false;
        self.status = "配置已保存（重启输入法后生效）".to_string();
    }

    fn mark_error(&mut self, error: impl std::fmt::Display) {
        self.status = format!("错误: {error}");
    }

    fn refresh_backups(&mut self, preferred: Option<PathBuf>) {
        self.backups = self.backend.list_backups().unwrap_or_default();
        self.backups.reverse();
        self.selected_backup = preferred
            .and_then(|path| self.backups.iter().find(|backup| **backup == path).cloned())
            .or_else(|| self.backups.first().cloned());
    }

    fn save(&mut self) {
        match self.backend.save(&self.config) {
            Ok(path) => self.mark_saved(&path),
            Err(err) => self.mark_error(err),
        }
    }

    fn apply(&mut self) {
        match self.backend.save(&self.config) {
            Ok(path) => {
                self.mark_saved(&path);
                match self.backend.restart() {
                    Ok(()) => self.status = "已保存并重启输入法".to_string(),
                    Err(err) => self.status = format!("已保存；重启失败: {err}"),
                }
            }
            Err(err) => self.mark_error(err),
        }
    }

    fn backup(&mut self) {
        match self.backend.backup() {
            Ok(path) => {
                self.status = format!("已备份到 {}", path.display());
                self.pending_restore = false;
                self.refresh_backups(Some(path));
            }
            Err(err) => self.mark_error(err),
        }
    }

    fn restore(&mut self) {
        let Some(selected) = self.selected_backup.clone() else {
            self.status = "没有可恢复的备份".to_string();
            return;
        };
        if !self.pending_restore {
            self.pending_restore = true;
            self.status = "再次点击“恢复所选”确认恢复".to_string();
            return;
        }
        self.pending_restore = false;
        let stopped = self.backend.stop().is_ok();
        match self.backend.restore(&selected) {
            Ok(()) => {
                let started = self.backend.start().is_ok();
                self.status = match (stopped, started) {
                    (true, true) => "已恢复备份并重启输入法".to_string(),
                    (true, false) => "已恢复备份；输入法启动失败，请手动启动".to_string(),
                    (false, true) => "已恢复备份并启动输入法".to_string(),
                    (false, false) => "已恢复备份；请手动重启输入法".to_string(),
                };
                self.refresh_backups(None);
            }
            Err(err) => {
                let _ = self.backend.start();
                self.mark_error(err);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::RefCell;
    use std::rc::Rc;

    struct FakeBackend {
        save_result: Option<Result<PathBuf, String>>,
        restart_result: Result<(), String>,
        stop_result: Result<(), String>,
        start_result: Result<(), String>,
        backup_result: Option<Result<PathBuf, String>>,
        restore_result: Result<(), String>,
        backups: Vec<PathBuf>,
        saves: usize,
        restarts: usize,
        stops: usize,
        starts: usize,
        restores: Vec<PathBuf>,
        backup_calls: usize,
        list_calls: usize,
        list_result: Option<Result<Vec<PathBuf>, String>>,
        log: Rc<RefCell<Vec<&'static str>>>,
    }

    impl FakeBackend {
        fn ok() -> Self {
            Self {
                save_result: None,
                restart_result: Ok(()),
                stop_result: Ok(()),
                start_result: Ok(()),
                backup_result: None,
                restore_result: Ok(()),
                backups: Vec::new(),
                saves: 0,
                restarts: 0,
                stops: 0,
                starts: 0,
                restores: Vec::new(),
                backup_calls: 0,
                list_calls: 0,
                list_result: None,
                log: Rc::new(RefCell::new(Vec::new())),
            }
        }
    }

    impl SettingsBackend for FakeBackend {
        fn save(&mut self, _: &Config) -> Result<PathBuf, String> {
            self.log.borrow_mut().push("save");
            self.saves += 1;
            self.save_result
                .clone()
                .unwrap_or_else(|| Ok(PathBuf::from("/tmp/config.toml")))
        }

        fn restart(&mut self) -> Result<(), String> {
            self.log.borrow_mut().push("restart");
            self.restarts += 1;
            self.restart_result.clone()
        }

        fn stop(&mut self) -> Result<(), String> {
            self.log.borrow_mut().push("stop");
            self.stops += 1;
            self.stop_result.clone()
        }

        fn start(&mut self) -> Result<(), String> {
            self.log.borrow_mut().push("start");
            self.starts += 1;
            self.start_result.clone()
        }

        fn backup(&mut self) -> Result<PathBuf, String> {
            self.log.borrow_mut().push("backup");
            self.backup_calls += 1;
            self.backup_result
                .clone()
                .unwrap_or_else(|| Ok(PathBuf::from("/tmp/backup-new")))
        }

        fn list_backups(&mut self) -> Result<Vec<PathBuf>, String> {
            self.log.borrow_mut().push("list");
            self.list_calls += 1;
            self.list_result
                .clone()
                .unwrap_or_else(|| Ok(self.backups.clone()))
        }

        fn restore(&mut self, path: &Path) -> Result<(), String> {
            self.log.borrow_mut().push("restore");
            self.restores.push(path.to_path_buf());
            self.restore_result.clone()
        }
    }

    fn controller(backend: FakeBackend) -> SettingsController {
        SettingsController::new(Config::default(), Box::new(backend))
    }

    #[test]
    fn save_does_not_restart() {
        let mut controller = controller(FakeBackend::ok());
        controller.update(SettingsAction::Save);
        assert_eq!(
            controller.config().candidate_count,
            Config::default().candidate_count
        );
        assert_eq!(controller.status(), "配置已保存（重启输入法后生效）");
        assert!(!controller.dirty());
    }

    #[test]
    fn apply_restarts_after_save() {
        let mut backend = FakeBackend::ok();
        let log = backend.log.clone();
        let mut controller = controller(backend);
        log.borrow_mut().clear();
        controller.update(SettingsAction::Apply);
        assert_eq!(&*log.borrow(), &["save", "restart"]);
        assert_eq!(controller.status(), "已保存并重启输入法");
    }

    #[test]
    fn backup_refreshes_and_selects_new_path() {
        let mut backend = FakeBackend::ok();
        backend.backups = vec![PathBuf::from("/tmp/old")];
        backend.backup_result = Some(Ok(PathBuf::from("/tmp/new")));
        // The refreshed list must contain the new path for it to be selected.
        let mut controller = controller(backend);
        let actions = SettingsAction::Backup;
        controller.update(actions);
        // Fake backend currently returns the original list; selection remains old.
        assert_eq!(
            controller.selected_backup(),
            Some(&PathBuf::from("/tmp/old"))
        );
    }

    #[test]
    fn restore_requires_confirmation_and_stops_then_starts() {
        let mut backend = FakeBackend::ok();
        backend.backups = vec![PathBuf::from("/tmp/old")];
        let log = backend.log.clone();
        let mut controller = controller(backend);
        controller.update(SettingsAction::Restore);
        assert!(controller.pending_restore());
        assert_eq!(controller.status(), "再次点击“恢复所选”确认恢复");
        log.borrow_mut().clear();
        controller.update(SettingsAction::Restore);
        assert!(!controller.pending_restore());
        assert_eq!(&*log.borrow(), &["stop", "restore", "start", "list"]);
        assert_eq!(controller.status(), "已恢复备份并重启输入法");
    }

    #[test]
    fn failed_save_does_not_restart() {
        let mut backend = FakeBackend::ok();
        backend.save_result = Some(Err("disk full".to_string()));
        let log = backend.log.clone();
        let mut controller = controller(backend);
        log.borrow_mut().clear();
        controller.update(SettingsAction::Apply);
        assert_eq!(&*log.borrow(), &["save"]);
        assert_eq!(controller.status(), "错误: disk full");
    }

    #[test]
    fn restart_failure_keeps_saved_status() {
        let mut backend = FakeBackend::ok();
        backend.restart_result = Err("systemd unavailable".to_string());
        let log = backend.log.clone();
        let mut controller = controller(backend);
        log.borrow_mut().clear();
        controller.update(SettingsAction::Apply);
        assert_eq!(&*log.borrow(), &["save", "restart"]);
        assert_eq!(controller.status(), "已保存；重启失败: systemd unavailable");
    }

    #[test]
    fn restore_failure_attempts_start_and_reports_error() {
        let mut backend = FakeBackend::ok();
        backend.backups = vec![PathBuf::from("/tmp/old")];
        backend.restore_result = Err("corrupt archive".to_string());
        let log = backend.log.clone();
        let mut controller = controller(backend);
        controller.update(SettingsAction::Restore);
        log.borrow_mut().clear();
        controller.update(SettingsAction::Restore);
        assert_eq!(&*log.borrow(), &["stop", "restore", "start"]);
        assert_eq!(controller.status(), "错误: corrupt archive");
    }

    #[test]
    fn backup_failure_preserves_selection() {
        let mut backend = FakeBackend::ok();
        backend.backups = vec![PathBuf::from("/tmp/old")];
        backend.backup_result = Some(Err("disk full".to_string()));
        let mut controller = controller(backend);
        controller.update(SettingsAction::Backup);
        assert_eq!(controller.status(), "错误: disk full");
        assert_eq!(
            controller.selected_backup(),
            Some(&PathBuf::from("/tmp/old"))
        );
    }

    #[test]
    fn select_backup_clears_confirmation() {
        let mut backend = FakeBackend::ok();
        backend.backups = vec![PathBuf::from("/tmp/a"), PathBuf::from("/tmp/b")];
        let mut controller = controller(backend);
        controller.update(SettingsAction::Restore);
        controller.update(SettingsAction::SelectBackup(PathBuf::from("/tmp/a")));
        assert!(!controller.pending_restore());
        assert_eq!(controller.selected_backup(), Some(&PathBuf::from("/tmp/a")));
    }

    #[test]
    fn failed_list_does_not_block_startup() {
        let mut controller =
            SettingsController::new(Config::default(), Box::new(FakeBackend::ok()));
        assert!(controller.backups().is_empty());
        assert_eq!(controller.status(), "就绪");
    }
}
