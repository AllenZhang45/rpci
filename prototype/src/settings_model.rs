use std::path::Path;

use crate::config::{Config, ThemePreference};

pub const ROW_COUNT: usize = 4;
const UNSAVED: &str = "未保存（按 S 保存）";

#[derive(Debug, Clone)]
pub struct SettingsModel {
    pub config: Config,
    pub selected: usize,
    pub status: String,
    pub dirty: bool,
}

impl SettingsModel {
    pub fn new(config: Config) -> Self {
        Self {
            config,
            selected: 0,
            status: "就绪".to_string(),
            dirty: false,
        }
    }

    pub fn move_selection(&mut self, delta: isize) {
        let next = (self.selected as isize + delta).rem_euclid(ROW_COUNT as isize);
        self.selected = next as usize;
    }

    pub fn adjust(&mut self, delta: i32) {
        if delta == 0 {
            return;
        }
        match self.selected {
            0 => self.config.ascii_mode_default = !self.config.ascii_mode_default,
            1 => self.config.pinyin_fallback = !self.config.pinyin_fallback,
            2 => {
                self.config.candidate_count =
                    (self.config.candidate_count as i32 + delta).clamp(1, 9) as usize;
            }
            3 => {
                let index = match self.config.theme {
                    ThemePreference::Auto => 0,
                    ThemePreference::Dark => 1,
                    ThemePreference::Light => 2,
                };
                let next = (index + delta).rem_euclid(3);
                self.config.theme = match next {
                    0 => ThemePreference::Auto,
                    1 => ThemePreference::Dark,
                    _ => ThemePreference::Light,
                };
            }
            _ => {}
        }
        self.mark_dirty();
    }

    pub fn toggle(&mut self) {
        match self.selected {
            0 => self.config.ascii_mode_default = !self.config.ascii_mode_default,
            1 => self.config.pinyin_fallback = !self.config.pinyin_fallback,
            2 => {
                self.config.candidate_count = (self.config.candidate_count % 9) + 1;
            }
            3 => self.adjust(1),
            _ => {}
        }
        self.mark_dirty();
    }

    pub fn mark_dirty(&mut self) {
        self.dirty = true;
        self.status = UNSAVED.to_string();
    }

    pub fn mark_saved(&mut self, _path: &Path) {
        self.dirty = false;
        self.status = "配置已保存（重启输入法后生效）".to_string();
    }

    pub fn mark_reloaded(&mut self, config: Config) {
        self.config = config;
        self.dirty = false;
        self.status = "已重新加载配置".to_string();
    }

    pub fn mark_error(&mut self, error: impl std::fmt::Display) {
        self.status = format!("错误: {error}");
    }

    pub fn theme_label(&self) -> &'static str {
        match self.config.theme {
            ThemePreference::Auto => "auto",
            ThemePreference::Dark => "dark",
            ThemePreference::Light => "light",
        }
    }

    pub fn row_label(&self, row: usize) -> String {
        let marker = if row == self.selected { ">" } else { " " };
        match row {
            0 => format!(
                "{marker} 默认英文模式: {}",
                on_off(self.config.ascii_mode_default)
            ),
            1 => format!(
                "{marker} 拼音兜底:    {}",
                on_off(self.config.pinyin_fallback)
            ),
            2 => format!("{marker} 候选数量:    {}", self.config.candidate_count),
            3 => format!("{marker} 主题:        {}", self.theme_label()),
            _ => String::new(),
        }
    }

    pub fn lines(&self, config_path: &str) -> Vec<String> {
        let mut lines = vec![
            "COSMIC IME 设置".to_string(),
            "↑↓ 选择   ←→ 修改   Enter 切换".to_string(),
            "S 保存   B 备份   R 重载   Esc 关闭".to_string(),
            String::new(),
        ];
        for row in 0..ROW_COUNT {
            lines.push(self.row_label(row));
        }
        lines.push(String::new());
        lines.push(format!("配置文件: {config_path}"));
        lines.push(format!("状态: {}", self.status));
        lines
    }
}

fn on_off(value: bool) -> &'static str {
    if value { "开" } else { "关" }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_at_first_row() {
        let model = SettingsModel::new(Config::default());
        assert_eq!(model.selected, 0);
        assert!(!model.dirty);
    }

    #[test]
    fn toggle_ascii_and_mark_dirty() {
        let mut model = SettingsModel::new(Config::default());
        model.toggle();
        assert!(model.config.ascii_mode_default);
        assert!(model.dirty);
        assert!(model.status.contains("未保存"));
    }

    #[test]
    fn candidate_count_clamps() {
        let mut model = SettingsModel::new(Config::default());
        model.selected = 2;
        for _ in 0..20 {
            model.adjust(1);
        }
        assert_eq!(model.config.candidate_count, 9);
        for _ in 0..20 {
            model.adjust(-1);
        }
        assert_eq!(model.config.candidate_count, 1);
    }

    #[test]
    fn theme_cycles() {
        let mut model = SettingsModel::new(Config::default());
        model.selected = 3;
        model.adjust(1);
        assert_eq!(model.config.theme, ThemePreference::Dark);
        model.adjust(1);
        assert_eq!(model.config.theme, ThemePreference::Light);
        model.adjust(1);
        assert_eq!(model.config.theme, ThemePreference::Auto);
    }

    #[test]
    fn selection_wraps() {
        let mut model = SettingsModel::new(Config::default());
        model.move_selection(-1);
        assert_eq!(model.selected, ROW_COUNT - 1);
        model.move_selection(1);
        assert_eq!(model.selected, 0);
    }

    #[test]
    fn row_labels_mark_selection() {
        let model = SettingsModel::new(Config::default());
        assert!(model.row_label(0).starts_with("> "));
        assert!(model.row_label(1).starts_with("  "));
    }
}
