use std::fs;
use std::io::Write;
use std::path::PathBuf;
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::paths;
use crate::service;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UserEntry {
    pub text: String,
    pub code: String,
    pub commits: i64,
}

pub trait UserDictionaryBackend {
    fn list_dicts(&mut self) -> Result<Vec<String>, String>;
    fn load(&mut self, dict: &str) -> Result<Vec<UserEntry>, String>;
}

#[derive(Default)]
pub struct RealUserDictionaryBackend;

impl UserDictionaryBackend for RealUserDictionaryBackend {
    fn list_dicts(&mut self) -> Result<Vec<String>, String> {
        let output = Command::new("rime_dict_manager")
            .current_dir(paths::rime_user_dir())
            .arg("--list")
            .output()
            .map_err(|err| format!("无法运行 rime_dict_manager: {err}"))?;
        if !output.status.success() {
            return Err(stderr(&output));
        }
        Ok(String::from_utf8_lossy(&output.stdout)
            .lines()
            .map(str::trim)
            .filter(|line| !line.is_empty())
            .map(str::to_string)
            .collect())
    }

    fn load(&mut self, dict: &str) -> Result<Vec<UserEntry>, String> {
        let dir = paths::rime_user_dir();
        let export = std::env::temp_dir().join(format!(
            "rpci-userdict-{}-{}.tsv",
            std::process::id(),
            now_millis()
        ));

        service::stop_input_method().map_err(|err| format!("停止输入法失败: {err}"))?;
        let result = export_dict(&dir, dict, &export);
        let start_result = service::start_input_method();
        let _ = fs::remove_file(&export);

        let entries = result?;
        if let Err(err) = start_result {
            return Err(format!("导出完成，但启动输入法失败: {err}"));
        }
        Ok(entries)
    }
}

fn export_dict(
    dir: &std::path::Path,
    dict: &str,
    export: &std::path::Path,
) -> Result<Vec<UserEntry>, String> {
    let output = Command::new("rime_dict_manager")
        .current_dir(dir)
        .args(["--export", dict])
        .arg(export)
        .output()
        .map_err(|err| format!("无法运行 rime_dict_manager: {err}"))?;
    if !output.status.success() {
        return Err(stderr(&output));
    }
    let text = fs::read_to_string(export).map_err(|err| format!("读取导出文件失败: {err}"))?;
    Ok(parse_export(&text))
}

fn stderr(output: &std::process::Output) -> String {
    let text = String::from_utf8_lossy(&output.stderr).trim().to_string();
    if text.is_empty() {
        format!("rime_dict_manager 退出码 {}", output.status)
    } else {
        text
    }
}

fn now_millis() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis())
        .unwrap_or(0)
}

pub fn parse_export(text: &str) -> Vec<UserEntry> {
    text.lines()
        .filter_map(|line| {
            let line = line.trim_end_matches('\r');
            if line.is_empty() || line.starts_with('#') {
                return None;
            }
            let mut fields = line.splitn(3, '\t');
            let text = fields.next()?.to_string();
            let code = fields.next()?.to_string();
            let commits = fields
                .next()
                .and_then(|value| value.parse::<i64>().ok())
                .unwrap_or(0);
            Some(UserEntry {
                text,
                code,
                commits,
            })
        })
        .collect()
}

pub struct UserDictionary {
    backend: Box<dyn UserDictionaryBackend>,
    dicts: Vec<String>,
    selected: Option<String>,
    entries: Vec<UserEntry>,
    status: String,
}

impl UserDictionary {
    pub fn new(mut backend: Box<dyn UserDictionaryBackend>) -> Self {
        match backend.list_dicts() {
            Ok(mut dicts) => {
                dicts.sort();
                dicts.dedup();
                let selected = dicts.first().cloned();
                let status = if dicts.is_empty() {
                    "未找到用户词典".to_string()
                } else {
                    "就绪".to_string()
                };
                Self {
                    backend,
                    dicts,
                    selected,
                    entries: Vec::new(),
                    status,
                }
            }
            Err(err) => Self {
                backend,
                dicts: Vec::new(),
                selected: None,
                entries: Vec::new(),
                status: format!("读取用户词典失败: {err}"),
            },
        }
    }

    pub fn dicts(&self) -> &[String] {
        &self.dicts
    }

    pub fn selected(&self) -> Option<&str> {
        self.selected.as_deref()
    }

    pub fn entries(&self) -> &[UserEntry] {
        &self.entries
    }

    pub fn status(&self) -> &str {
        &self.status
    }

    pub fn set_status(&mut self, status: String) {
        self.status = status;
    }

    pub fn select(&mut self, dict: String) {
        if self.dicts.contains(&dict) {
            self.selected = Some(dict);
            self.entries.clear();
            self.status = "就绪".to_string();
        }
    }

    pub fn load(&mut self) {
        let Some(dict) = self.selected.clone() else {
            self.status = "没有可加载的用户词典".to_string();
            return;
        };
        match self.backend.load(&dict) {
            Ok(entries) => {
                self.status = format!("已加载 {} 条", entries.len());
                self.entries = entries;
            }
            Err(err) => {
                self.entries.clear();
                self.status = format!("加载失败: {err}");
            }
        }
    }

    pub fn export(&mut self) -> Result<PathBuf, String> {
        if self.entries.is_empty() {
            return Err("请先加载用户词典".to_string());
        }
        let dir = paths::exports_dir().ok_or_else(|| "无法确定导出目录".to_string())?;
        fs::create_dir_all(&dir).map_err(|err| format!("创建导出目录失败: {err}"))?;
        let dict = self.selected.as_deref().unwrap_or("userdict");
        let path = dir.join(format!("{dict}-userdict.tsv"));
        let mut file = fs::File::create(&path).map_err(|err| format!("创建导出文件失败: {err}"))?;
        writeln!(file, "# RPCI user dictionary export")
            .map_err(|err| format!("写入导出文件失败: {err}"))?;
        for entry in &self.entries {
            writeln!(file, "{}\t{}\t{}", entry.text, entry.code, entry.commits)
                .map_err(|err| format!("写入导出文件失败: {err}"))?;
        }
        self.status = format!("已导出到 {}", path.display());
        Ok(path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct FakeBackend {
        dicts: Vec<String>,
        entries: Vec<UserEntry>,
        loaded: usize,
    }

    impl UserDictionaryBackend for FakeBackend {
        fn list_dicts(&mut self) -> Result<Vec<String>, String> {
            Ok(self.dicts.clone())
        }

        fn load(&mut self, _: &str) -> Result<Vec<UserEntry>, String> {
            self.loaded += 1;
            Ok(self.entries.clone())
        }
    }

    fn backend() -> FakeBackend {
        FakeBackend {
            dicts: vec!["wubi86".to_string(), "pinyin_simp".to_string()],
            entries: vec![UserEntry {
                text: "工".to_string(),
                code: "a".to_string(),
                commits: 1,
            }],
            loaded: 0,
        }
    }

    #[test]
    fn parses_export_and_skips_comments() {
        let text = "# header\n工\ta\t1\r\n世界\tanlw\t2\nbad\n";
        let entries = parse_export(text);
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].text, "工");
        assert_eq!(entries[1].commits, 2);
    }

    #[test]
    fn lists_and_selects_first_dict() {
        let dictionary = UserDictionary::new(Box::new(backend()));
        assert_eq!(dictionary.dicts(), &["pinyin_simp", "wubi86"]);
        assert_eq!(dictionary.selected(), Some("pinyin_simp"));
        assert_eq!(dictionary.status(), "就绪");
    }

    #[test]
    fn load_moves_entries_into_view() {
        let mut dictionary = UserDictionary::new(Box::new(backend()));
        dictionary.load();
        assert_eq!(dictionary.entries().len(), 1);
        assert_eq!(dictionary.status(), "已加载 1 条");
    }

    #[test]
    fn empty_dictionary_is_explicit() {
        let mut backend = backend();
        backend.dicts.clear();
        let dictionary = UserDictionary::new(Box::new(backend));
        assert_eq!(dictionary.status(), "未找到用户词典");
    }

    #[test]
    fn export_requires_loaded_entries() {
        let mut dictionary = UserDictionary::new(Box::new(backend()));
        assert!(dictionary.export().is_err());
    }
}
