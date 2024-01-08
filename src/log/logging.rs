use chrono::Local;
use std::fmt::Write as FmtWrite;
use std::fs;
use std::fs::OpenOptions;
use std::io::Write;
use std::path::Path;

pub fn rename_server_and_client_logs() {
    let files = [
        "src/log/log_files/server.log",
        "src/log/log_files/client.log",
    ];

    for file_path in &files {
        let path = Path::new(file_path);

        if path.exists() {
            let now = Local::now();
            let timestamp = now.format("%Y%m%d%H%M%S").to_string();
            let new_file_name = format!("{}_{}.log", file_path.trim_end_matches(".log"), timestamp);

            fs::rename(file_path, &new_file_name).expect("Unable to rename file");
        }
    }
}

// Usage example
// log("server", "This is a test log message");
// log("client", "This is a test log message");
pub fn log(file: &str, log_message: String) {
    if file.is_empty() || log_message.is_empty() {
        return;
    }

    let log_file = match file {
        "server" => "src/log/log_files/server.log",
        "client" => "src/log/log_files/client.log",
        _ => "src/log/log_files/server.log",
    };

    let mut message = String::new();

    let line = line!();
    let file_source = file!();

    let now = Local::now();
    write!(message, "[{}]", now.format("%d/%m/%y %H:%M:%S")).unwrap();

    write!(message, "[{}:{}] ", file_source, line).unwrap();

    writeln!(message, "{}", log_message).unwrap();

    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(log_file)
        .expect("Unable to open file");
    file.write_all(message.as_bytes())
        .expect("Unable to write to file");
}
