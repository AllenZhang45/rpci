use std::io::Write;
use std::os::unix::net::UnixStream;
use std::time::Duration;

use iced::keyboard::{self, key::Named};
use iced::widget::{button, row, text, tooltip};
use iced::{Background, Border, Color, Element, Subscription, Task, Theme, window};
use ime_probe::paths::{control_path, state_path};
use ime_probe::popup::cosmic_is_dark;

const APPLET_ID: &str = "com.github.alec.RpciImeApplet";

fn main() -> iced::Result {
    iced::application(App::default, App::update, App::view)
        .title(|_: &App| "COSMIC IME".to_string())
        .subscription(App::subscription)
        .window(window::Settings {
            size: (120.0, 34.0).into(),
            decorations: false,
            resizable: false,
            transparent: true,
            platform_specific: window::settings::PlatformSpecific {
                application_id: APPLET_ID.to_string(),
                ..Default::default()
            },
            ..window::Settings::default()
        })
        .theme(App::theme)
        .run()
}

#[derive(Debug, Clone)]
enum Message {
    Tick,
    Toggle,
    OpenSettings,
}

struct App {
    label: String,
    dark: bool,
}

impl Default for App {
    fn default() -> Self {
        Self {
            label: read_label(),
            dark: cosmic_is_dark().unwrap_or(true),
        }
    }
}

impl App {
    fn theme(&self) -> Theme {
        if self.dark { Theme::Dark } else { Theme::Light }
    }

    fn subscription(&self) -> Subscription<Message> {
        Subscription::batch([
            iced::time::every(Duration::from_millis(500)).map(|_| Message::Tick),
            keyboard::listen().filter_map(|event| {
                if let keyboard::Event::KeyPressed {
                    key: keyboard::Key::Named(Named::Enter),
                    ..
                } = event
                {
                    Some(Message::Toggle)
                } else {
                    None
                }
            }),
        ])
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Tick => {
                self.label = read_label();
                self.dark = cosmic_is_dark().unwrap_or(self.dark);
            }
            Message::Toggle => {
                if let Some(path) = control_path() {
                    if let Ok(mut stream) = UnixStream::connect(path) {
                        let _ = stream.write_all(b"toggle\n");
                    }
                }
            }
            Message::OpenSettings => {
                let settings = std::env::current_exe()
                    .ok()
                    .and_then(|exe| exe.parent().map(|dir| dir.join("ime-settings-iced")))
                    .filter(|path| path.is_file())
                    .unwrap_or_else(|| std::path::PathBuf::from("ime-settings-iced"));
                let _ = std::process::Command::new(settings).spawn();
            }
        }
        Task::none()
    }

    fn view(&self) -> Element<'_, Message> {
        let toggle_hint = if self.label == "英" {
            "切换为中文"
        } else {
            "切换为英文"
        };
        let label = tooltip(
            button(text(&self.label).size(15))
                .padding([4, 8])
                .on_press(Message::Toggle)
                .style(applet_button),
            text(toggle_hint).size(12),
            tooltip::Position::Top,
        )
        .gap(4);

        let settings = tooltip(
            button(text("⚙").size(14))
                .padding([4, 6])
                .on_press(Message::OpenSettings)
                .style(applet_button),
            text("打开输入法设置").size(12),
            tooltip::Position::Top,
        )
        .gap(4);

        row![label, settings].spacing(2).into()
    }
}

fn applet_button(
    theme: &Theme,
    status: iced::widget::button::Status,
) -> iced::widget::button::Style {
    let background = match status {
        iced::widget::button::Status::Hovered => Some(Background::Color(Color::from_rgba(
            theme.palette().text.r,
            theme.palette().text.g,
            theme.palette().text.b,
            0.08,
        ))),
        iced::widget::button::Status::Pressed => Some(Background::Color(Color::from_rgba(
            theme.palette().text.r,
            theme.palette().text.g,
            theme.palette().text.b,
            0.16,
        ))),
        _ => None,
    };

    iced::widget::button::Style {
        background,
        text_color: theme.palette().text,
        border: Border {
            radius: 6.0.into(),
            ..Border::default()
        },
        ..Default::default()
    }
}

fn read_label() -> String {
    state_path()
        .and_then(|path| std::fs::read_to_string(path).ok())
        .map(|label| label.trim().to_string())
        .filter(|label| !label.is_empty())
        .unwrap_or_else(|| "IME".to_string())
}
