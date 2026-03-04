use std::fs::{create_dir_all, File, OpenOptions};
use std::io::Write;
use std::path::PathBuf;
use std::sync::{Mutex, OnceLock};

use chrono::Local;

static LOG_FILE: OnceLock<Option<Mutex<File>>> = OnceLock::new();

pub fn error(message: &str) {
    write_line("ERROR", message);
}

fn write_line(level: &str, message: &str) {
    let timestamp = Local::now().format("%Y-%m-%d %H:%M:%S");
    let line = format!("{} [{}] {}\n", timestamp, level, message);

    let Some(file) = LOG_FILE.get_or_init(open_log_file).as_ref() else {
        eprintln!("{}", line.trim_end());
        return;
    };

    if let Ok(mut guard) = file.lock() {
        if guard.write_all(line.as_bytes()).is_ok() {
            let _ = guard.flush();
            return;
        }
    }

    eprintln!("{}", line.trim_end());
}

fn open_log_file() -> Option<Mutex<File>> {
    let path = log_file_path();

    if let Some(parent) = path.parent() {
        if create_dir_all(parent).is_err() {
            return None;
        }
    }

    OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .ok()
        .map(Mutex::new)
}

#[cfg(target_os = "macos")]
fn log_file_path() -> PathBuf {
    if let Some(home) = std::env::var_os("HOME") {
        return PathBuf::from(home).join("Library/Logs/mybar.log");
    }

    PathBuf::from("/tmp/mybar.log")
}

#[cfg(target_os = "linux")]
fn log_file_path() -> PathBuf {
    if let Some(xdg_state_home) = std::env::var_os("XDG_STATE_HOME") {
        return PathBuf::from(xdg_state_home).join("mybar/mybar.log");
    }

    if let Some(home) = std::env::var_os("HOME") {
        return PathBuf::from(home).join(".local/state/mybar/mybar.log");
    }

    PathBuf::from("/tmp/mybar.log")
}
