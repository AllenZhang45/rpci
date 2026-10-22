use std::collections::HashSet;

use crate::engine::{InputEngine, PageInfo};

const CTRL_MASK: u32 = 1 << 2;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct KeyEvent {
    pub keycode: u32,
    pub pressed: bool,
    pub is_ctrl: bool,
    pub keysym: u32,
    pub text: String,
    pub mods: u32,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Action {
    Forward {
        keycode: u32,
        pressed: bool,
    },
    Tap {
        keycode: u32,
    },
    Commit(String),
    ShowCandidates {
        preedit: String,
        candidates: Vec<String>,
        page: PageInfo,
    },
    Hide,
    Status(String),
    SetModifiers(u32),
}

pub struct InputSession {
    engine: Box<dyn InputEngine>,
    deferred_ctrl: Option<u32>,
    ctrl_toggled: bool,
    forwarded_keys: HashSet<u32>,
    pending_taps: HashSet<u32>,
}

impl InputSession {
    pub fn new(engine: Box<dyn InputEngine>) -> Self {
        Self {
            engine,
            deferred_ctrl: None,
            ctrl_toggled: false,
            forwarded_keys: HashSet::new(),
            pending_taps: HashSet::new(),
        }
    }

    pub fn status_label(&self) -> String {
        self.engine.status_label()
    }

    pub fn cancel(&mut self) -> Vec<Action> {
        self.engine.cancel();
        self.deferred_ctrl = None;
        self.ctrl_toggled = false;
        self.forwarded_keys.clear();
        self.pending_taps.clear();
        vec![Action::Hide, Action::Status(self.engine.status_label())]
    }

    pub fn toggle_ascii(&mut self) -> Vec<Action> {
        self.engine.toggle_ascii();
        vec![Action::Hide, Action::Status(self.engine.status_label())]
    }

    pub fn handle(&mut self, key: KeyEvent) -> Vec<Action> {
        let mut actions = Vec::new();

        if key.is_ctrl {
            if key.pressed {
                self.deferred_ctrl = Some(key.keycode);
                return actions;
            }
            if self.ctrl_toggled {
                self.ctrl_toggled = false;
                self.deferred_ctrl = None;
                actions.push(Action::SetModifiers(key.mods & !CTRL_MASK));
            } else if self.deferred_ctrl.take().is_some() {
                // Bar Ctrl is intentionally swallowed.
            } else if self.forwarded_keys.remove(&key.keycode) {
                actions.push(Action::Forward {
                    keycode: key.keycode,
                    pressed: false,
                });
            }
            return actions;
        }

        if key.pressed {
            if key.keycode == 57 && self.deferred_ctrl.is_some() {
                self.deferred_ctrl = None;
                self.ctrl_toggled = true;
                self.engine.toggle_ascii();
                actions.push(Action::SetModifiers(key.mods & !CTRL_MASK));
                actions.push(Action::Hide);
                actions.push(Action::Status(self.engine.status_label()));
                return actions;
            }

            if let Some(ctrl_key) = self.deferred_ctrl.take() {
                self.forwarded_keys.insert(ctrl_key);
                actions.push(Action::Forward {
                    keycode: ctrl_key,
                    pressed: true,
                });
            }

            let output = self.engine.handle(key.keysym, &key.text, key.mods);
            if output.passthrough {
                if is_modifier_key(key.keycode) {
                    self.forwarded_keys.insert(key.keycode);
                    actions.push(Action::Forward {
                        keycode: key.keycode,
                        pressed: true,
                    });
                } else {
                    self.pending_taps.insert(key.keycode);
                }
                return actions;
            }

            if let Some(commit) = output.commit {
                if !commit.is_empty() {
                    actions.push(Action::Commit(commit));
                }
            }

            if output.preedit.is_empty() && output.candidates.is_empty() {
                actions.push(Action::Hide);
            } else {
                actions.push(Action::ShowCandidates {
                    preedit: output.preedit,
                    candidates: output.candidates,
                    page: output.page,
                });
            }
            actions.push(Action::Status(self.engine.status_label()));
            return actions;
        }

        if self.pending_taps.remove(&key.keycode) {
            actions.push(Action::Tap {
                keycode: key.keycode,
            });
        } else if self.forwarded_keys.remove(&key.keycode) {
            actions.push(Action::Forward {
                keycode: key.keycode,
                pressed: false,
            });
        }
        actions
    }
}

fn is_modifier_key(keycode: u32) -> bool {
    matches!(keycode, 29 | 97 | 42 | 54 | 56 | 100 | 125 | 126 | 58)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::{EngineOutput, Mode};
    use std::collections::VecDeque;

    struct FakeEngine {
        outputs: VecDeque<EngineOutput>,
        toggled: usize,
        cancelled: usize,
        seen: Vec<(u32, String, u32)>,
        label: String,
    }

    impl FakeEngine {
        fn new(outputs: Vec<EngineOutput>) -> Self {
            Self {
                outputs: outputs.into(),
                toggled: 0,
                cancelled: 0,
                seen: Vec::new(),
                label: "中 五".to_string(),
            }
        }
    }

    impl InputEngine for FakeEngine {
        fn handle(&mut self, keysym: u32, text: &str, mods: u32) -> EngineOutput {
            self.seen.push((keysym, text.to_string(), mods));
            self.outputs.pop_front().unwrap_or_else(empty_output)
        }

        fn cancel(&mut self) {
            self.cancelled += 1;
        }

        fn toggle_ascii(&mut self) {
            self.toggled += 1;
            self.label = "英".to_string();
        }

        fn status_label(&self) -> String {
            self.label.clone()
        }
    }

    fn empty_output() -> EngineOutput {
        EngineOutput {
            mode: Mode::Wubi,
            preedit: String::new(),
            candidates: Vec::new(),
            commit: None,
            passthrough: false,
            page: PageInfo::default(),
        }
    }

    fn pass_output() -> EngineOutput {
        EngineOutput {
            passthrough: true,
            ..empty_output()
        }
    }

    fn key(keycode: u32, keysym: u32, text: &str, mods: u32) -> KeyEvent {
        KeyEvent {
            keycode,
            pressed: true,
            is_ctrl: false,
            keysym,
            text: text.to_string(),
            mods,
        }
    }

    fn released(keycode: u32, keysym: u32, text: &str, mods: u32) -> KeyEvent {
        KeyEvent {
            keycode,
            pressed: false,
            is_ctrl: false,
            keysym,
            text: text.to_string(),
            mods,
        }
    }

    fn ctrl(keycode: u32, pressed: bool, mods: u32) -> KeyEvent {
        KeyEvent {
            keycode,
            pressed,
            is_ctrl: true,
            keysym: 0xffe3,
            text: String::new(),
            mods,
        }
    }

    #[test]
    fn ctrl_space_toggles_without_leaking_ctrl() {
        let engine = FakeEngine::new(Vec::new());
        let mut session = InputSession::new(Box::new(engine));
        assert!(session.handle(ctrl(29, true, 4)).is_empty());
        let actions = session.handle(key(57, 0x20, " ", 4));
        assert!(actions.contains(&Action::SetModifiers(0)));
        assert!(actions.contains(&Action::Hide));
        assert!(actions.contains(&Action::Status("英".to_string())));
        assert!(
            !actions
                .iter()
                .any(|action| matches!(action, Action::Forward { .. }))
        );
        assert_eq!(
            session.handle(ctrl(29, false, 0)),
            vec![Action::SetModifiers(0)]
        );
    }

    #[test]
    fn bare_ctrl_is_swallowed() {
        let mut session = InputSession::new(Box::new(FakeEngine::new(Vec::new())));
        assert!(session.handle(ctrl(29, true, 4)).is_empty());
        assert!(session.handle(ctrl(29, false, 0)).is_empty());
    }

    #[test]
    fn passthrough_key_becomes_a_tap_on_release() {
        let engine = FakeEngine::new(vec![pass_output()]);
        let mut session = InputSession::new(Box::new(engine));
        assert!(session.handle(key(38, 0x61, "a", 0)).is_empty());
        assert_eq!(
            session.handle(released(38, 0x61, "a", 0)),
            vec![Action::Tap { keycode: 38 }]
        );
    }

    #[test]
    fn commit_then_hide_then_status() {
        let mut output = empty_output();
        output.commit = Some("你".to_string());
        let engine = FakeEngine::new(vec![output]);
        let mut session = InputSession::new(Box::new(engine));
        let actions = session.handle(key(25, 0x77, "w", 0));
        assert_eq!(
            actions,
            vec![
                Action::Commit("你".to_string()),
                Action::Hide,
                Action::Status("中 五".to_string())
            ]
        );
    }

    #[test]
    fn deferred_ctrl_is_forwarded_before_engine() {
        let engine = FakeEngine::new(vec![empty_output()]);
        let mut session = InputSession::new(Box::new(engine));
        assert!(session.handle(ctrl(29, true, 4)).is_empty());
        let actions = session.handle(key(38, 0x61, "a", 4));
        assert_eq!(
            actions[0],
            Action::Forward {
                keycode: 29,
                pressed: true
            }
        );
        assert!(actions.contains(&Action::Hide));
    }

    #[test]
    fn cancel_clears_key_state() {
        let mut session = InputSession::new(Box::new(FakeEngine::new(Vec::new())));
        session.handle(key(38, 0x61, "a", 0));
        assert_eq!(
            session.cancel(),
            vec![Action::Hide, Action::Status("中 五".to_string())]
        );
    }
}
