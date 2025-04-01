use anyhow::{anyhow, Result};
use std::{env, fs, process};
use std::process::Command;
use sha1::{Digest, Sha1};
use log::info;
use native_dialog::{MessageType, MessageDialog};
use iced::Task;
use crate::Message;
use env_logger::Target;

pub fn init_logger() -> Result<()> {
    let logfile = fs::OpenOptions::new()
        .create(true)
        .truncate(true)
        .write(true)
        .open("update.log")?;
    pretty_env_logger::formatted_timed_builder()
        .parse_filters("uma_autoupdate=info")
        .target(Target::Pipe(Box::new(logfile)))
        .init();
    Ok(())
}

/// 返回为iced的异步任务
pub fn msgbox(text: &str) -> Task<Message> {
    let t = text.to_string();
    Task::perform(async move {
        info!("{t}");
        MessageDialog::new()
            .set_type(MessageType::Info)
            .set_title("提示")
            .set_text(&t)
            .show_alert()
        },
        |_| { Message::text("msgbox return") }
    )
}
pub fn to_hex(bytes: &[u8]) -> String {
    bytes.iter()
        .fold(String::new(), |s, byte| s + &format!("{:02x}", byte))
}
pub fn get_exe_name() -> Result<String> {
    let exe_path = env::current_exe()?;
    // 把exe_path转为相对路径。中文路径可能有问题
    let cwd = env::current_dir()?;
    Ok(exe_path
        .to_string_lossy()
        .replace(&cwd.to_string_lossy().to_string(), ".")
    )
}
pub fn replace_self() -> Result<()> {
    let exe_name = get_exe_name()?;
    info!("Replacing {exe_name}");
    let old_name = format!("{exe_name}.old");
    fs::rename(&exe_name, &old_name)?;
    fs::rename("uma-autoupdate.exe.autoupdate", &exe_name)?;
    let _ = Command::new("cmd")
        .args(["/C", "start", &exe_name])
        .spawn()?;
    process::exit(0);
}

pub fn remove_old() -> Result<()> {
    let exe_path = env::current_exe()?;
    // 把exe_path转为相对路径。中文路径可能有问题
    let cwd = env::current_dir()?;
    let exe_name = exe_path
        .to_string_lossy()
        .replace(&cwd.to_string_lossy().to_string(), ".");
    let old_name = format!("{exe_name}.old");
    if fs::exists(&old_name)? {
        fs::remove_file(&old_name)?;
        info!("Removing {old_name}");
    }
    Ok(())
}

pub fn get_file_sha1(filename: &str) -> Result<String> {
    let contents = std::fs::read(&filename)?;
    let mut hasher = Sha1::new();
    hasher.update(&contents);
    let result = hasher.finalize();
    let result_text = to_hex(&result);
    info!("File: {filename}, SHA1: {result_text}");
    Ok(result_text)
}