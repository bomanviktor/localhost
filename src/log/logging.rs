use chrono::Local;
use std::fmt::Write as FmtWrite;
use std::fs;
use std::fs::OpenOptions;
use std::io::Write;
use std::path::Path;

pub enum LogFileType {
    Server,
    Client,
}

const LOG_DIR: &str = "./src/log/log_files/";

impl LogFileType {
    fn file_path(&self) -> String {
        match self {
            LogFileType::Server => format!("{LOG_DIR}server.log"),
            LogFileType::Client => format!("{LOG_DIR}client.log"),
        }
    }
}
pub fn init_logs() {
    let _ = fs::create_dir(LOG_DIR);
    let files = [LogFileType::Server, LogFileType::Client];

    for file_type in &files {
        let file_path = file_type.file_path();
        let path = Path::new(&file_path);

        if path.exists() {
            let now = Local::now();
            let timestamp = now.format("%Y%m%d%H%M%S").to_string();
            let new_file_name = format!("{}_{}.log", file_path.trim_end_matches(".log"), timestamp);

            fs::rename(file_path, &new_file_name).expect("Unable to rename file");
        }
    }
}

pub fn log_with_file_line(
    file_type: LogFileType,
    log_message: String,
    file_source: &str,
    line_number: u32,
) {
    if log_message.is_empty() {
        return;
    }

    let log_file = file_type.file_path();

    let mut message = String::new();
    let now = Local::now();
    write!(message, "[{}]", now.format("%d/%m/%y %H:%M:%S")).unwrap();
    write!(message, "[{}:{}] ", file_source, line_number).unwrap();
    writeln!(message, "{}", log_message).unwrap();

    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(log_file)
        .expect("Unable to open file");
    file.write_all(message.as_bytes())
        .expect("Unable to write to file");
}

// Macro to simplify logging
// Usage example
// log!(LogFileType::Server, "This is a test log message");
// log!(LogFileType::Client, "This is a test log message");
#[macro_export]
macro_rules! log {
    ($file_type:expr, $log_message:expr) => {
        $crate::log::log_with_file_line($file_type, $log_message, file!(), line!())
    };
}
