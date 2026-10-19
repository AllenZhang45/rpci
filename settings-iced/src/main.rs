use std::fmt;
use std::path::PathBuf;

use iced::widget::{
    Space, button, checkbox, column, container, pick_list, row, scrollable, slider, text,
};
use iced::{Alignment, Border, Color, Element, Length, Task, Theme};
use ime_probe::backup::{self, KEEP_BACKUPS, default_paths};
use ime_probe::config::{Config, ThemePreference};
use ime_probe::service;
use ime_probe::settings_model::SettingsModel;

const THEMES: [ThemeChoice; 3] = [ThemeChoice::Auto, ThemeChoice::Dark, ThemeChoice::Light];

fn main() -> iced::Result {
    iced::application(App::default, App::update, App::view)
        .title(|_: &App| "COSMIC IME 设置".to_string())
        .theme(App::theme)
        .window_size((620.0, 580.0))
        .run()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ThemeChoice {
    Auto,
    Dark,
    Light,
}

impl ThemeChoice {
    fn from_config(theme: ThemePreference) -> Self {
        match theme {
            ThemePreference::Auto => Self::Auto,
            ThemePreference::Dark => Self::Dark,
            ThemePreference::Light => Self::Light,
        }
    }

    fn to_config(self) -> ThemePreference {
        match self {
            Self::Auto => ThemePreference::Auto,
            Self::Dark => ThemePreference::Dark,
            Self::Light => ThemePreference::Light,
        }
    }
}

impl fmt::Display for ThemeChoice {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Auto => "auto",
            Self::Dark => "dark",
            Self::Light => "light",
        })
    }
}

#[derive(Debug, Clone)]
enum Message {
    AsciiToggled(bool),
    PinyinToggled(bool),
    CountChanged(u8),
    ThemeSelected(ThemeChoice),
    Save,
    Apply,
    Backup,
    BackupSelected(Backup),
    Restore,
    Reload,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Backup {
    path: PathBuf,
}

impl fmt::Display for Backup {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = self
            .path
            .file_name()
            .map(|name| name.to_string_lossy().into_owned())
            .unwrap_or_else(|| self.path.display().to_string());
        f.write_str(&name)
    }
}

fn load_backups() -> Vec<Backup> {
    backup::list_backups(&default_paths())
        .unwrap_or_default()
        .into_iter()
        .rev()
        .map(|path| Backup { path })
        .collect()
}

struct App {
    model: SettingsModel,
    backups: Vec<Backup>,
    selected_backup: Option<Backup>,
    pending_restore: bool,
}

impl Default for App {
    fn default() -> Self {
        let backups = load_backups();
        Self {
            model: SettingsModel::new(Config::load()),
            selected_backup: backups.first().cloned(),
            backups,
            pending_restore: false,
        }
    }
}

impl App {
    fn refresh_backups(&mut self, preferred: Option<PathBuf>) {
        self.backups = load_backups();
        self.selected_backup = preferred
            .and_then(|path| {
                self.backups
                    .iter()
                    .find(|backup| backup.path == path)
                    .cloned()
            })
            .or_else(|| self.backups.first().cloned());
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::AsciiToggled(value) => {
                self.model.config.ascii_mode_default = value;
                self.model.mark_dirty();
            }
            Message::PinyinToggled(value) => {
                self.model.config.pinyin_fallback = value;
                self.model.mark_dirty();
            }
            Message::CountChanged(value) => {
                self.model.config.candidate_count = value as usize;
                self.model.mark_dirty();
            }
            Message::ThemeSelected(value) => {
                self.model.config.theme = value.to_config();
                self.model.mark_dirty();
            }
            Message::Save => match self.model.config.save() {
                Ok(path) => self.model.mark_saved(&path),
                Err(err) => self.model.mark_error(err),
            },
            Message::Apply => match self.model.config.save() {
                Ok(path) => {
                    self.model.mark_saved(&path);
                    match service::restart_input_method() {
                        Ok(()) => self.model.status = "已保存并重启输入法".to_string(),
                        Err(err) => self.model.status = format!("已保存；重启失败: {err}"),
                    }
                }
                Err(err) => self.model.mark_error(err),
            },
            Message::Backup => match backup::backup_now(&default_paths(), KEEP_BACKUPS) {
                Ok(path) => {
                    self.model.status = format!("已备份到 {}", path.display());
                    self.pending_restore = false;
                    self.refresh_backups(Some(path));
                }
                Err(err) => self.model.mark_error(err),
            },
            Message::BackupSelected(value) => {
                self.selected_backup = Some(value);
                self.pending_restore = false;
            }
            Message::Restore => {
                let Some(selected) = self.selected_backup.clone() else {
                    self.model.status = "没有可恢复的备份".to_string();
                    return Task::none();
                };
                if !self.pending_restore {
                    self.pending_restore = true;
                    self.model.status = "再次点击“恢复所选”确认恢复".to_string();
                    return Task::none();
                }
                self.pending_restore = false;
                let paths = default_paths();
                let stopped = service::stop_input_method().is_ok();
                match backup::import_from(&paths, &selected.path) {
                    Ok(()) => {
                        let started = service::start_input_method().is_ok();
                        self.model.status = match (stopped, started) {
                            (true, true) => "已恢复备份并重启输入法".to_string(),
                            (true, false) => "已恢复备份；输入法启动失败，请手动启动".to_string(),
                            (false, true) => "已恢复备份并启动输入法".to_string(),
                            (false, false) => "已恢复备份；请手动重启输入法".to_string(),
                        };
                        self.refresh_backups(None);
                    }
                    Err(err) => {
                        let _ = service::start_input_method();
                        self.model.mark_error(err);
                    }
                }
            }
            Message::Reload => {
                self.pending_restore = false;
                self.model.mark_reloaded(Config::load());
            }
        }
        Task::none()
    }

    fn view(&self) -> Element<'_, Message> {
        let title = column![
            text("COSMIC IME 设置").size(26),
            text("五笔 / 拼音输入法 · COSMIC")
                .size(13)
                .style(|theme: &Theme| text::Style {
                    color: Some(theme.palette().text.scale_alpha(0.65)),
                }),
        ]
        .spacing(4);

        let general = card(
            "常规",
            column![
                checkbox(self.model.config.ascii_mode_default)
                    .label("默认英文模式")
                    .on_toggle(Message::AsciiToggled),
                checkbox(self.model.config.pinyin_fallback)
                    .label("拼音兜底（超过 4 码或没有五笔候选时启用）")
                    .on_toggle(Message::PinyinToggled),
            ]
            .spacing(14),
        );

        let count = self.model.config.candidate_count as u8;
        let candidates = card(
            "候选",
            column![
                row![
                    text("候选数量"),
                    Space::new().width(Length::Fill),
                    text(count.to_string()).size(16),
                ]
                .align_y(Alignment::Center),
                slider(1..=9, count, Message::CountChanged),
                row![
                    text("主题"),
                    Space::new().width(Length::Fill),
                    pick_list(
                        THEMES,
                        Some(ThemeChoice::from_config(self.model.config.theme)),
                        Message::ThemeSelected,
                    ),
                ]
                .align_y(Alignment::Center),
            ]
            .spacing(14),
        );

        let dictionary = card(
            "词库",
            column![
                text(format!("{} 个备份（最近 5 个）", self.backups.len())).size(13),
                row![
                    pick_list(
                        self.backups.clone(),
                        self.selected_backup.clone(),
                        Message::BackupSelected,
                    )
                    .placeholder("没有备份")
                    .width(Length::Fill),
                    button("立即备份").on_press(Message::Backup),
                    button(if self.pending_restore {
                        "确认恢复"
                    } else {
                        "恢复所选"
                    })
                    .on_press(Message::Restore),
                ]
                .spacing(10)
                .align_y(Alignment::Center),
            ]
            .spacing(12),
        );

        let actions = row![
            button("保存").on_press(Message::Save),
            button("保存并应用").on_press(Message::Apply),
            button("重新加载").on_press(Message::Reload),
        ]
        .spacing(10);

        let footer = column![
            actions,
            text(&self.model.status).size(13).style(|theme: &Theme| {
                text::Style {
                    color: Some(theme.palette().text.scale_alpha(0.7)),
                }
            }),
            text("“保存并应用”会通过 systemd 重启输入法")
                .size(12)
                .style(|theme: &Theme| text::Style {
                    color: Some(theme.palette().text.scale_alpha(0.55)),
                }),
        ]
        .spacing(12);

        container(
            scrollable(
                column![title, general, dictionary, candidates, footer]
                    .spacing(20)
                    .width(Length::Fill),
            )
            .height(Length::Fill),
        )
        .padding(24)
        .width(Length::Fill)
        .height(Length::Fill)
        .style(|theme: &Theme| container::Style {
            background: Some(theme.palette().background.into()),
            ..Default::default()
        })
        .into()
    }

    fn theme(&self) -> Theme {
        let palette = iced::theme::Palette {
            background: Color::from_rgb8(0x1e, 0x1e, 0x1e),
            text: Color::from_rgb8(0xf2, 0xf2, 0xf2),
            primary: Color::from_rgb8(0x35, 0x84, 0xe4),
            success: Color::from_rgb8(0x2e, 0xc2, 0x7a),
            warning: Color::from_rgb8(0xf5, 0xa6, 0x23),
            danger: Color::from_rgb8(0xe0, 0x5a, 0x5a),
        };
        Theme::custom("COSMIC Dark", palette)
    }
}

fn card<'a>(title: &'a str, content: iced::widget::Column<'a, Message>) -> Element<'a, Message> {
    container(column![text(title).size(16), content].spacing(14))
        .padding(18)
        .width(Length::Fill)
        .style(|_: &Theme| container::Style {
            background: Some(Color::from_rgb8(0x2b, 0x2b, 0x2b).into()),
            border: Border {
                color: Color::from_rgb8(0x3a, 0x3a, 0x3a),
                width: 1.0,
                radius: 12.0.into(),
            },
            ..Default::default()
        })
        .into()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn theme_choice_roundtrip() {
        for choice in THEMES {
            assert_eq!(ThemeChoice::from_config(choice.to_config()), choice);
        }
    }

    #[test]
    fn restore_requires_confirmation() {
        let mut app = App {
            model: SettingsModel::new(Config::default()),
            backups: vec![Backup {
                path: PathBuf::from("/tmp/backups/1"),
            }],
            selected_backup: Some(Backup {
                path: PathBuf::from("/tmp/backups/1"),
            }),
            pending_restore: false,
        };
        let _ = app.update(Message::Restore);
        assert!(app.pending_restore);
        assert!(app.model.status.contains("确认"));
    }

    #[test]
    fn backup_label_is_file_name() {
        let backup = Backup {
            path: PathBuf::from("/tmp/backups/1792451348"),
        };
        assert_eq!(backup.to_string(), "1792451348");
    }

    #[test]
    fn update_changes_config() {
        let mut app = App {
            model: SettingsModel::new(Config::default()),
            backups: Vec::new(),
            selected_backup: None,
            pending_restore: false,
        };
        let _ = app.update(Message::AsciiToggled(true));
        assert!(app.model.config.ascii_mode_default);
        let _ = app.update(Message::PinyinToggled(false));
        assert!(!app.model.config.pinyin_fallback);
        let _ = app.update(Message::CountChanged(7));
        assert_eq!(app.model.config.candidate_count, 7);
        let _ = app.update(Message::ThemeSelected(ThemeChoice::Light));
        assert_eq!(app.model.config.theme, ThemePreference::Light);
    }
}
