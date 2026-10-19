use std::path::Path;
use std::process::ExitCode;

use ime_probe::backup::{self, KEEP_BACKUPS, default_paths};
use ime_probe::config::{Config, config_path};
use ime_probe::service;

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match args.as_slice() {
        [] => show(),
        [cmd] if cmd == "show" => show(),
        [cmd] if cmd == "path" => {
            match config_path() {
                Some(path) => println!("{}", path.display()),
                None => {
                    eprintln!("no XDG_CONFIG_HOME or HOME");
                    return ExitCode::FAILURE;
                }
            }
            ExitCode::SUCCESS
        }
        [cmd] if cmd == "backup" => {
            let paths = default_paths();
            eprintln!("tip: stop the input method before backup for a consistent userdb");
            match backup::backup_now(&paths, KEEP_BACKUPS) {
                Ok(path) => {
                    println!("{}", path.display());
                    ExitCode::SUCCESS
                }
                Err(err) => fail(err),
            }
        }
        [cmd] if cmd == "backups" => match backup::list_backups(&default_paths()) {
            Ok(backups) => {
                for path in backups {
                    println!("{}", path.display());
                }
                ExitCode::SUCCESS
            }
            Err(err) => fail(err),
        },
        [cmd, dir] if cmd == "export" => {
            let paths = default_paths();
            eprintln!("tip: stop the input method before export for a consistent userdb");
            match backup::export_to(&paths, Path::new(dir)) {
                Ok(()) => {
                    println!("exported to {dir}");
                    ExitCode::SUCCESS
                }
                Err(err) => fail(err),
            }
        }
        [cmd, dir] if cmd == "import" => {
            let paths = default_paths();
            match backup::import_from(&paths, Path::new(dir)) {
                Ok(()) => {
                    println!("imported from {dir}");
                    ExitCode::SUCCESS
                }
                Err(err) => fail(err),
            }
        }
        [cmd] if cmd == "apply" => {
            let config = Config::load();
            match config.save() {
                Ok(path) => {
                    println!("saved {}", path.display());
                    match service::restart_input_method() {
                        Ok(()) => {
                            println!("restarted {}", service::UNIT);
                            ExitCode::SUCCESS
                        }
                        Err(err) => {
                            eprintln!("saved, but restart failed: {err}");
                            ExitCode::FAILURE
                        }
                    }
                }
                Err(err) => fail(err),
            }
        }
        [cmd] if cmd == "reset" => {
            let config = Config::default();
            match config.save() {
                Ok(path) => {
                    println!("reset {}", path.display());
                    ExitCode::SUCCESS
                }
                Err(err) => fail(err),
            }
        }
        [cmd, key] if cmd == "get" => {
            let config = Config::load();
            match config.get(key) {
                Some(value) => {
                    println!("{value}");
                    ExitCode::SUCCESS
                }
                None => {
                    eprintln!("unknown key: {key}");
                    ExitCode::FAILURE
                }
            }
        }
        [cmd, key, value] if cmd == "set" => {
            let mut config = Config::load();
            if let Err(err) = config.apply(key, value) {
                eprintln!("{err}");
                return ExitCode::FAILURE;
            }
            match config.save() {
                Ok(path) => {
                    println!("updated {}: {key} = {value}", path.display());
                    ExitCode::SUCCESS
                }
                Err(err) => fail(err),
            }
        }
        _ => {
            eprintln!(
                "usage: ime-settings [show|path|reset|apply|get <key>|set <key> <value>]\n\
                 ime-settings [backup|backups|export <dir>|import <dir>]\n\
                 keys: ascii_mode_default, pinyin_fallback, candidate_count, theme"
            );
            ExitCode::FAILURE
        }
    }
}

fn show() -> ExitCode {
    let config = Config::load();
    match config_path() {
        Some(path) => println!("# {}", path.display()),
        None => println!("# no config path"),
    }
    match toml::to_string_pretty(&config) {
        Ok(text) => {
            print!("{text}");
            ExitCode::SUCCESS
        }
        Err(err) => fail(err),
    }
}

fn fail(err: impl std::fmt::Display) -> ExitCode {
    eprintln!("{err}");
    ExitCode::FAILURE
}
