use std::fmt;
use std::path::PathBuf;

use iced::widget::{
    Space, button, checkbox, column, container, pick_list, row, scrollable, slider, text,
};
use iced::{Alignment, Border, Color, Element, Length, Task, Theme};
use ime_probe::config::{CandidateLayout, Config, ThemePreference};
use ime_probe::settings_controller::{RealSettingsBackend, SettingsAction, SettingsController};
use ime_probe::user_dictionary::{RealUserDictionaryBackend, UserDictionary};

const THEMES: [ThemeChoice; 3] = [ThemeChoice::Auto, ThemeChoice::Dark, ThemeChoice::Light];
const LAYOUTS: [LayoutChoice; 3] = [
    LayoutChoice::Single,
    LayoutChoice::TwoRow,
    LayoutChoice::Auto,
];

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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum LayoutChoice {
    Single,
    TwoRow,
    Auto,
}

impl LayoutChoice {
    fn from_config(layout: CandidateLayout) -> Self {
        match layout {
            CandidateLayout::Single => Self::Single,
            CandidateLayout::TwoRow => Self::TwoRow,
            CandidateLayout::Auto => Self::Auto,
        }
    }

    fn to_config(self) -> CandidateLayout {
        match self {
            Self::Single => CandidateLayout::Single,
            Self::TwoRow => CandidateLayout::TwoRow,
            Self::Auto => CandidateLayout::Auto,
        }
    }
}

impl fmt::Display for LayoutChoice {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Single => "单行",
            Self::TwoRow => "两行",
            Self::Auto => "自动",
        })
    }
}

#[derive(Debug, Clone)]
enum Message {
    AsciiToggled(bool),
    PinyinToggled(bool),
    CountChanged(u8),
    LayoutSelected(LayoutChoice),
    ThemeSelected(ThemeChoice),
    Save,
    Apply,
    Backup,
    BackupSelected(Backup),
    Restore,
    Reload,
    UserDictSelected(String),
    UserDictLoad,
    UserDictExport,
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

struct App {
    controller: SettingsController,
    user_dictionary: UserDictionary,
}

impl Default for App {
    fn default() -> Self {
        Self {
            controller: SettingsController::new(Config::load(), Box::new(RealSettingsBackend)),
            user_dictionary: UserDictionary::new(Box::new(RealUserDictionaryBackend)),
        }
    }
}

impl App {
    fn update(&mut self, message: Message) -> Task<Message> {
        let action = match message {
            Message::AsciiToggled(value) => SettingsAction::SetAscii(value),
            Message::PinyinToggled(value) => SettingsAction::SetPinyin(value),
            Message::CountChanged(value) => SettingsAction::SetCandidateCount(value as usize),
            Message::LayoutSelected(value) => SettingsAction::SetLayout(value.to_config()),
            Message::ThemeSelected(value) => SettingsAction::SetTheme(value.to_config()),
            Message::Save => SettingsAction::Save,
            Message::Apply => SettingsAction::Apply,
            Message::Backup => SettingsAction::Backup,
            Message::BackupSelected(value) => SettingsAction::SelectBackup(value.path),
            Message::Restore => SettingsAction::Restore,
            Message::Reload => SettingsAction::Reload,
            Message::UserDictSelected(value) => {
                self.user_dictionary.select(value);
                return Task::none();
            }
            Message::UserDictLoad => {
                self.user_dictionary.load();
                return Task::none();
            }
            Message::UserDictExport => {
                if let Err(err) = self.user_dictionary.export() {
                    self.user_dictionary.set_status(format!("导出失败: {err}"));
                }
                return Task::none();
            }
        };
        self.controller.update(action);
        Task::none()
    }

    fn view(&self) -> Element<'_, Message> {
        let backups = self
            .controller
            .backups()
            .iter()
            .map(|path| Backup { path: path.clone() })
            .collect::<Vec<_>>();
        let selected_backup = self
            .controller
            .selected_backup()
            .map(|path| Backup { path: path.clone() });
        let pending_restore = self.controller.pending_restore();
        let user_dicts = self.user_dictionary.dicts().to_vec();
        let selected_user_dict = self.user_dictionary.selected().map(str::to_string);
        let user_entries = self.user_dictionary.entries().to_vec();
        let user_status = self.user_dictionary.status().to_string();
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
                checkbox(self.controller.config().ascii_mode_default)
                    .label("默认英文模式")
                    .on_toggle(Message::AsciiToggled),
                checkbox(self.controller.config().pinyin_fallback)
                    .label("拼音兜底（超过 4 码或没有五笔候选时启用）")
                    .on_toggle(Message::PinyinToggled),
            ]
            .spacing(14),
        );

        let count = self.controller.config().candidate_count as u8;
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
                    text("候选布局"),
                    Space::new().width(Length::Fill),
                    pick_list(
                        LAYOUTS,
                        Some(LayoutChoice::from_config(
                            self.controller.config().candidate_layout
                        )),
                        Message::LayoutSelected,
                    ),
                ]
                .align_y(Alignment::Center),
                row![
                    text("主题"),
                    Space::new().width(Length::Fill),
                    pick_list(
                        THEMES,
                        Some(ThemeChoice::from_config(self.controller.config().theme)),
                        Message::ThemeSelected,
                    ),
                ]
                .align_y(Alignment::Center),
            ]
            .spacing(14),
        );

        let user_dictionary = card(
            "用户词库（只读）",
            column![
                row![
                    pick_list(user_dicts, selected_user_dict, Message::UserDictSelected,)
                        .placeholder("没有用户词典")
                        .width(Length::Fill),
                    button("加载").on_press(Message::UserDictLoad),
                    button("导出").on_press(Message::UserDictExport),
                ]
                .spacing(10)
                .align_y(Alignment::Center),
                text(user_status).size(13),
                scrollable(
                    column![
                        row![
                            text("词条").width(Length::FillPortion(2)),
                            text("编码").width(Length::FillPortion(2)),
                            text("词频").width(Length::FillPortion(1)),
                        ]
                        .spacing(8),
                        column(
                            user_entries
                                .iter()
                                .map(|entry| {
                                    row![
                                        text(entry.text.clone()).width(Length::FillPortion(2)),
                                        text(entry.code.clone()).width(Length::FillPortion(2)),
                                        text(entry.commits.to_string())
                                            .width(Length::FillPortion(1)),
                                    ]
                                    .spacing(8)
                                    .into()
                                })
                                .collect::<Vec<_>>()
                        )
                        .spacing(6),
                    ]
                    .spacing(8),
                )
                .height(Length::Fixed(180.0)),
            ]
            .spacing(12),
        );

        let dictionary = card(
            "词库",
            column![
                text(format!(
                    "{} 个备份（最近 5 个）",
                    self.controller.backups().len()
                ))
                .size(13),
                row![
                    pick_list(backups, selected_backup, Message::BackupSelected)
                        .placeholder("没有备份")
                        .width(Length::Fill),
                    button("立即备份").on_press(Message::Backup),
                    button(if pending_restore {
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
            text(self.controller.status())
                .size(13)
                .style(|theme: &Theme| {
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
                column![
                    title,
                    general,
                    user_dictionary,
                    dictionary,
                    candidates,
                    footer
                ]
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
    fn layout_choice_roundtrip() {
        for choice in LAYOUTS {
            assert_eq!(LayoutChoice::from_config(choice.to_config()), choice);
        }
    }

    #[test]
    fn backup_label_is_file_name() {
        let backup = Backup {
            path: PathBuf::from("/tmp/backups/1792451348"),
        };
        assert_eq!(backup.to_string(), "1792451348");
    }

    #[test]
    fn update_changes_controller_config() {
        let mut app = App {
            controller: SettingsController::new(Config::default(), Box::new(RealSettingsBackend)),
            user_dictionary: UserDictionary::new(Box::new(RealUserDictionaryBackend)),
        };
        let _ = app.update(Message::AsciiToggled(true));
        assert!(app.controller.config().ascii_mode_default);
        let _ = app.update(Message::PinyinToggled(false));
        assert!(!app.controller.config().pinyin_fallback);
        let _ = app.update(Message::CountChanged(7));
        assert_eq!(app.controller.config().candidate_count, 7);
        let _ = app.update(Message::LayoutSelected(LayoutChoice::TwoRow));
        assert_eq!(
            app.controller.config().candidate_layout,
            CandidateLayout::TwoRow
        );
        let _ = app.update(Message::ThemeSelected(ThemeChoice::Light));
        assert_eq!(app.controller.config().theme, ThemePreference::Light);
    }
}
