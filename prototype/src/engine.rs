use rime_api::{Session, Traits};

use crate::config::Config;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Mode {
    Wubi,
    Pinyin,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PageInfo {
    pub page_no: usize,
    pub page_size: usize,
    pub is_last_page: bool,
}

impl Default for PageInfo {
    fn default() -> Self {
        Self {
            page_no: 0,
            page_size: 0,
            is_last_page: true,
        }
    }
}

impl PageInfo {
    pub fn label(self) -> Option<String> {
        if self.page_no == 0 && self.is_last_page {
            None
        } else {
            Some(format!("第 {} 页", self.page_no + 1))
        }
    }

    pub fn nav(self) -> &'static str {
        match (self.page_no > 0, !self.is_last_page) {
            (false, false) => "",
            (true, false) => "‹ ›",
            (false, true) => "›",
            (true, true) => "‹ ›",
        }
    }
}

pub struct EngineOutput {
    pub mode: Mode,
    pub preedit: String,
    pub candidates: Vec<String>,
    pub commit: Option<String>,
    pub passthrough: bool,
    pub page: PageInfo,
}

const MOD_CTRL: u32 = 1 << 2;
const MOD_ALT: u32 = 1 << 3;
const MOD_SUPER: u32 = 1 << 6;

fn should_toggle(keysym: u32, _mods: u32) -> bool {
    // Ctrl+Space is deliberately not bound: COSMIC consumes it in a way that
    // drops text-input focus on this setup. F2 is the reliable shortcut.
    keysym == 0xffbf
}

fn is_page_key(keysym: u32) -> bool {
    matches!(keysym, 0xff55 | 0xff56)
}

fn is_punctuation(keysym: u32) -> bool {
    matches!(keysym, 0x21..=0x2f | 0x3a..=0x40 | 0x5b..=0x60 | 0x7b..=0x7e)
}

fn should_passthrough(keysym: u32, text: &str, mods: u32, composing: bool) -> bool {
    if mods & (MOD_CTRL | MOD_ALT | MOD_SUPER) != 0 {
        return true;
    }
    if !text.is_empty() && text.chars().all(|c| c.is_ascii_alphabetic()) {
        return false;
    }
    if is_punctuation(keysym) {
        return false;
    }
    match keysym {
        0x20 | 0x31..=0x39 => !composing,
        0xff08 => !composing,
        0xff1b | 0xff0d => !composing,
        _ => true,
    }
}

fn status_label_for(ascii_mode: bool, mode: Mode) -> &'static str {
    if ascii_mode {
        "英"
    } else {
        match mode {
            Mode::Wubi => "中 五",
            Mode::Pinyin => "中 拼",
        }
    }
}

fn choose_mode(letters_len: usize, wubi_candidate_count: usize) -> Mode {
    if letters_len > 4 || wubi_candidate_count == 0 {
        Mode::Pinyin
    } else {
        Mode::Wubi
    }
}

fn choose_mode_with_fallback(
    letters_len: usize,
    wubi_candidate_count: usize,
    fallback_enabled: bool,
) -> Mode {
    if fallback_enabled {
        choose_mode(letters_len, wubi_candidate_count)
    } else {
        Mode::Wubi
    }
}

pub trait InputEngine {
    fn handle(&mut self, keysym: u32, text: &str, mods: u32) -> EngineOutput;
    fn cancel(&mut self);
    fn toggle_ascii(&mut self);
    fn status_label(&self) -> String;
}

pub struct HybridEngine {
    _traits: Traits,
    wubi: Session,
    pinyin: Option<Session>,
    letters: String,
    mode: Mode,
    ascii_mode: bool,
}

impl HybridEngine {
    pub fn new(config: &Config) -> Self {
        let shared_dir = std::env::var("IME_RIME_SHARED_DIR")
            .unwrap_or_else(|_| "/usr/share/rime-data".to_string());
        let user_dir = crate::paths::rime_user_dir();
        let log_dir = user_dir.join("log");
        let user_dir = user_dir.to_string_lossy().into_owned();
        let log_dir = log_dir.to_string_lossy().into_owned();

        let mut traits = Traits::new();
        traits
            .set_shared_data_dir(&shared_dir)
            .set_user_data_dir(&user_dir)
            .set_distribution_name("ime-probe")
            .set_distribution_code_name("ime-probe")
            .set_distribution_version("0.0.1")
            .set_log_dir(&log_dir)
            .set_min_log_level(3);

        rime_api::setup(&mut traits);
        rime_api::initialize(&mut traits);
        let deploy = rime_api::full_deploy_and_wait();
        println!("RIME deploy: {deploy:?}");

        let wubi = rime_api::create_session().expect("wubi session");
        wubi.select_schema("wubi86").expect("select wubi86");
        let pinyin = if config.pinyin_fallback {
            let pinyin = rime_api::create_session().expect("pinyin session");
            pinyin
                .select_schema("pinyin_simp")
                .expect("select pinyin_simp");
            Some(pinyin)
        } else {
            None
        };

        Self {
            _traits: traits,
            wubi,
            pinyin,
            letters: String::new(),
            mode: Mode::Wubi,
            ascii_mode: config.ascii_mode_default,
        }
    }

    fn feed(&self, session: &Session, keysym: u32) {
        session.process_key(rime_api::KeyEvent::new(keysym, 0));
    }

    fn feed_pinyin(&self, keysym: u32) {
        if let Some(pinyin) = &self.pinyin {
            self.feed(pinyin, keysym);
        }
    }

    fn candidates_of(&self, session: &Session) -> (String, Vec<String>, PageInfo) {
        if let Some(ctx) = session.context() {
            let preedit = ctx.composition().preedit.unwrap_or_default().to_string();
            let menu = ctx.menu();
            let page = PageInfo {
                page_no: menu.page_no,
                page_size: menu.page_size,
                is_last_page: menu.is_last_page,
            };
            let candidates = menu.candidates.iter().map(|c| c.text.to_string()).collect();
            (preedit, candidates, page)
        } else {
            (String::new(), Vec::new(), PageInfo::default())
        }
    }

    fn recur_mode(&mut self) {
        let (_, wubi_candidates, _) = self.candidates_of(&self.wubi);
        self.mode = choose_mode_with_fallback(
            self.letters.len(),
            wubi_candidates.len(),
            self.pinyin.is_some(),
        );
    }

    fn active(&self) -> &Session {
        match self.mode {
            Mode::Wubi => &self.wubi,
            Mode::Pinyin => self.pinyin.as_ref().unwrap_or(&self.wubi),
        }
    }

    fn reset(&mut self) {
        self.feed(&self.wubi, 0xff1b);
        self.feed_pinyin(0xff1b);
        self.letters.clear();
        self.mode = Mode::Wubi;
    }

    /// Clear any in-progress composition without changing the language mode.
    pub fn cancel(&mut self) {
        self.reset();
    }

    /// Toggle Chinese/English from an out-of-band control command.
    pub fn toggle_ascii(&mut self) {
        self.reset();
        self.ascii_mode = !self.ascii_mode;
    }

    pub fn status_label(&self) -> String {
        status_label_for(self.ascii_mode, self.mode).to_string()
    }

    pub fn handle(&mut self, keysym: u32, text: &str, mods: u32) -> EngineOutput {
        if should_toggle(keysym, mods) {
            self.reset();
            self.ascii_mode = !self.ascii_mode;
            let label = if self.ascii_mode { "English" } else { "中文" };
            return EngineOutput {
                mode: self.mode,
                preedit: label.to_string(),
                candidates: Vec::new(),
                commit: None,
                passthrough: false,
                page: PageInfo::default(),
            };
        }

        if self.ascii_mode {
            return EngineOutput {
                mode: self.mode,
                preedit: String::new(),
                candidates: Vec::new(),
                commit: None,
                passthrough: true,
                page: PageInfo::default(),
            };
        }

        // Enter is committed as a newline through the input-method protocol.
        // Forwarding it through the virtual keyboard loses the key-up on
        // cosmic-edit, so the editor never inserts a line break.
        if keysym == 0xff0d {
            self.reset();
            return EngineOutput {
                mode: Mode::Wubi,
                preedit: String::new(),
                candidates: Vec::new(),
                commit: Some("\n".to_string()),
                passthrough: false,
                page: PageInfo::default(),
            };
        }

        if is_page_key(keysym) && !self.letters.is_empty() {
            self.feed(self.active(), keysym);
            let (preedit, candidates, page) = self.candidates_of(self.active());
            return EngineOutput {
                mode: self.mode,
                preedit,
                candidates,
                commit: None,
                passthrough: false,
                page,
            };
        }

        if should_passthrough(keysym, text, mods, !self.letters.is_empty()) {
            return EngineOutput {
                mode: self.mode,
                preedit: String::new(),
                candidates: Vec::new(),
                commit: None,
                passthrough: true,
                page: PageInfo::default(),
            };
        }

        if !text.is_empty() && text.chars().all(|c| c.is_ascii_alphabetic()) {
            self.feed(&self.wubi, keysym);
            self.feed_pinyin(keysym);
            self.letters.push_str(text);
            self.recur_mode();
            let (preedit, candidates, page) = self.candidates_of(self.active());
            return EngineOutput {
                mode: self.mode,
                preedit,
                candidates,
                commit: None,
                passthrough: false,
                page,
            };
        }

        if is_punctuation(keysym) {
            self.feed(self.active(), keysym);
            let commit = self.active().commit().map(|c| c.text().to_string());
            let passthrough = commit.is_none();
            if commit.is_some() {
                self.letters.clear();
                self.mode = Mode::Wubi;
            }
            return EngineOutput {
                mode: self.mode,
                preedit: String::new(),
                candidates: Vec::new(),
                commit,
                passthrough,
                page: PageInfo::default(),
            };
        }

        match keysym {
            0x20 | 0x31..=0x39 => {
                self.feed(self.active(), keysym);
                let commit = self.active().commit().map(|c| c.text().to_string());
                if commit.is_some() {
                    self.reset();
                }
                EngineOutput {
                    mode: self.mode,
                    preedit: String::new(),
                    candidates: Vec::new(),
                    commit,
                    passthrough: false,
                    page: PageInfo::default(),
                }
            }
            0xff08 => {
                self.feed(&self.wubi, keysym);
                self.feed_pinyin(keysym);
                self.letters.pop();
                if self.letters.is_empty() {
                    self.reset();
                    return EngineOutput {
                        mode: Mode::Wubi,
                        preedit: String::new(),
                        candidates: Vec::new(),
                        commit: None,
                        passthrough: false,
                        page: PageInfo::default(),
                    };
                }
                self.recur_mode();
                let (preedit, candidates, page) = self.candidates_of(self.active());
                EngineOutput {
                    mode: self.mode,
                    preedit,
                    candidates,
                    commit: None,
                    passthrough: false,
                    page,
                }
            }
            0xff1b => {
                self.reset();
                EngineOutput {
                    mode: Mode::Wubi,
                    preedit: String::new(),
                    candidates: Vec::new(),
                    commit: None,
                    passthrough: false,
                    page: PageInfo::default(),
                }
            }
            _ => EngineOutput {
                mode: self.mode,
                preedit: String::new(),
                candidates: Vec::new(),
                commit: None,
                passthrough: true,
                page: PageInfo::default(),
            },
        }
    }
}

impl Drop for HybridEngine {
    fn drop(&mut self) {
        let _ = self.wubi.close();
        if let Some(pinyin) = &mut self.pinyin {
            let _ = pinyin.close();
        }
        rime_api::finalize();
    }
}

impl InputEngine for HybridEngine {
    fn handle(&mut self, keysym: u32, text: &str, mods: u32) -> EngineOutput {
        HybridEngine::handle(self, keysym, text, mods)
    }

    fn cancel(&mut self) {
        HybridEngine::cancel(self);
    }

    fn toggle_ascii(&mut self) {
        HybridEngine::toggle_ascii(self);
    }

    fn status_label(&self) -> String {
        HybridEngine::status_label(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fallback_after_five_letters() {
        assert_eq!(choose_mode(4, 2), Mode::Wubi);
        assert_eq!(choose_mode(5, 2), Mode::Pinyin);
    }

    #[test]
    fn fallback_without_wubi_candidates() {
        assert_eq!(choose_mode(2, 0), Mode::Pinyin);
        assert_eq!(choose_mode(2, 1), Mode::Wubi);
    }
}

#[cfg(test)]
mod passthrough_tests {
    use super::*;

    #[test]
    fn ctrl_shortcuts_pass_through() {
        assert!(should_passthrough(0x63, "c", MOD_CTRL, false));
        assert!(should_passthrough(0x76, "v", MOD_CTRL, true));
    }

    #[test]
    fn plain_letters_are_handled() {
        assert!(!should_passthrough(0x77, "w", 0, false));
        assert!(!should_passthrough(0x57, "W", 0, true));
    }

    #[test]
    fn page_keys_are_page_keys() {
        assert!(is_page_key(0xff55));
        assert!(is_page_key(0xff56));
        assert!(!is_page_key(0xff53));
        assert!(!is_page_key(0x2e));
    }

    #[test]
    fn page_label_hides_single_page() {
        assert_eq!(PageInfo::default().label(), None);
        assert_eq!(
            PageInfo {
                page_no: 0,
                page_size: 5,
                is_last_page: false
            }
            .label()
            .as_deref(),
            Some("第 1 页")
        );
        assert_eq!(
            PageInfo {
                page_no: 1,
                page_size: 5,
                is_last_page: true
            }
            .label()
            .as_deref(),
            Some("第 2 页")
        );
    }

    #[test]
    fn navigation_keys_pass_through() {
        assert!(should_passthrough(0xff53, "", 0, true));
        assert!(should_passthrough(0xffbe, "", 0, false));
        // Enter must reach multi-line applications when not composing.
        assert!(should_passthrough(0xff0d, "\r", 0, false));
    }

    #[test]
    fn space_and_digits_depend_on_composition() {
        assert!(should_passthrough(0x20, " ", 0, false));
        assert!(!should_passthrough(0x20, " ", 0, true));
        assert!(should_passthrough(0x31, "1", 0, false));
        assert!(!should_passthrough(0x31, "1", 0, true));
    }
}

#[cfg(test)]
mod punctuation_tests {
    use super::*;

    #[test]
    fn punctuation_is_handled_unless_shortcut() {
        assert!(!should_passthrough(0x2e, ".", 0, false));
        assert!(!should_passthrough(0x3f, "?", 0, true));
        assert!(should_passthrough(0x2e, ".", MOD_CTRL, false));
    }

    #[test]
    fn punctuation_ranges() {
        assert!(is_punctuation(0x2e));
        assert!(is_punctuation(0x3f));
        assert!(!is_punctuation(0x20));
        assert!(!is_punctuation(0x61));
    }
}

#[cfg(test)]
mod toggle_tests {
    use super::*;

    #[test]
    fn ctrl_space_and_f2_toggle() {
        assert!(!should_toggle(0x20, MOD_CTRL));
        assert!(should_toggle(0xffbf, 0));
        assert!(!should_toggle(0x20, 0));
        assert!(!should_toggle(0x61, 0));
    }
}

#[cfg(test)]
mod fallback_tests {
    use super::*;

    #[test]
    fn disabled_fallback_keeps_wubi() {
        assert_eq!(choose_mode_with_fallback(5, 0, false), Mode::Wubi);
        assert_eq!(choose_mode_with_fallback(5, 0, true), Mode::Pinyin);
    }
}

#[cfg(test)]
mod status_tests {
    use super::*;

    #[test]
    fn labels_reflect_mode() {
        assert_eq!(status_label_for(false, Mode::Wubi), "中 五");
        assert_eq!(status_label_for(false, Mode::Pinyin), "中 拼");
        assert_eq!(status_label_for(true, Mode::Wubi), "英");
    }
}
