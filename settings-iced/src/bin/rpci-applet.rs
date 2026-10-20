use std::time::Duration;

use iced::widget::{button, text};
use iced::{Element, Subscription, Task, Theme, window};
use ime_probe::paths::state_path;

const APPLET_ID: &str = "com.github.alec.RpciImeApplet";

fn main() -> iced::Result {
    iced::application(App::default, App::update, App::view)
        .title(|_: &App| "COSMIC IME".to_string())
        .subscription(App::subscription)
        .window(window::Settings {
            size: (140.0, 40.0).into(),
            decorations: false,
            resizable: false,
            transparent: true,
            platform_specific: window::settings::PlatformSpecific {
                application_id: APPLET_ID.to_string(),
                ..Default::default()
            },
            ..window::Settings::default()
        })
        .theme(|_: &App| Theme::Dark)
        .run()
}

#[derive(Debug, Clone)]
enum Message {
    Tick,
    OpenSettings,
}

struct App {
    label: String,
}

impl Default for App {
    fn default() -> Self {
        Self {
            label: read_label(),
        }
    }
}

impl App {
    fn subscription(&self) -> Subscription<Message> {
        iced::time::every(Duration::from_millis(500)).map(|_| Message::Tick)
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Tick => self.label = read_label(),
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
        button(text(&self.label).size(15))
            .padding([4, 10])
            .on_press(Message::OpenSettings)
            .style(|theme: &Theme, _status: iced::widget::button::Status| {
                iced::widget::button::Style {
                    background: None,
                    text_color: theme.palette().text,
                    border: iced::Border::default(),
                    ..Default::default()
                }
            })
            .into()
    }
}

fn read_label() -> String {
    state_path()
        .and_then(|path| std::fs::read_to_string(path).ok())
        .map(|label| label.trim().to_string())
        .filter(|label| !label.is_empty())
        .unwrap_or_else(|| "IME".to_string())
}

#[allow(dead_code)]
const _: &str = APPLET_ID;
