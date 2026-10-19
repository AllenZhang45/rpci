use std::fmt;

use iced::widget::{Space, button, checkbox, column, container, pick_list, row, slider, text};
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
        .window_size((620.0, 500.0))
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
    Reload,
}

struct App {
    model: SettingsModel,
}

impl Default for App {
    fn default() -> Self {
        Self {
            model: SettingsModel::new(Config::load()),
        }
    }
}

impl App {
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
                Ok(path) => self.model.status = format!("已备份到 {}", path.display()),
                Err(err) => self.model.mark_error(err),
            },
            Message::Reload => self.model.mark_reloaded(Config::load()),
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

        let actions = row![
            button("保存").on_press(Message::Save),
            button("保存并应用").on_press(Message::Apply),
            button("备份词库").on_press(Message::Backup),
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
            column![title, general, candidates, footer]
                .spacing(20)
                .width(Length::Fill),
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
    fn update_changes_config() {
        let mut app = App {
            model: SettingsModel::new(Config::default()),
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
