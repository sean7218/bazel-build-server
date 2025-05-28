use std::sync::OnceLock;
use std::{fmt::Debug, fs::OpenOptions};
use std::fs::File;
use std::io::Write;
use chrono::Local;

#[derive(Debug)]
pub struct Logger {
    pub file: File
}

static LOGGER: OnceLock<std::sync::Mutex<Logger>> = OnceLock::new();

pub fn get_logger() -> &'static std::sync::Mutex<Logger> {
    LOGGER.get_or_init(|| {
        let logger = Logger::new();
        std::sync::Mutex::new(logger)
    })
}

impl Logger {
    pub fn new() -> Self {
        let home_dir = dirs::home_dir()
            .expect("Failed to get home directory.");
        let server_dir = home_dir
            .join(".sourcekit-bsp");
        let log_file = server_dir
            .join("bsp.log");
        if !server_dir.exists() {
            std::fs::create_dir_all(&server_dir)
                .expect("Failed create log file directory");
        }

        Logger {
            file: OpenOptions::new()
                .create(true)
                .append(true)
                .open(log_file)
                .expect("Failed to create log file."),
        }
    }

    pub fn log(&mut self, message: &str) {
        let timestamp = Local::now().format("%Y-%m-%d %H:%M:%S");
        writeln!(self.file, "[{}] {}", timestamp, message).unwrap();
    }
}

#[macro_export]
macro_rules! log_str {
    ($msg:expr) => {
        $crate::utils::logger::get_logger().lock().unwrap().log($msg)
    };
    ($fmt:expr, $($arg:tt)*) => {
        $crate::utils::logger::get_logger().lock().unwrap().log(&format!($fmt, $($arg)*))
    };
}
