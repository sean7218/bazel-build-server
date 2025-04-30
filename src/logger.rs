use std::fmt::Debug;
use std::fs::File;
use std::io::Write;

pub struct Logger {
    pub file: File,
}

impl Logger {
    pub fn info(&mut self, message: &str) {
        writeln!(self.file, "buildserver | {:?}", message).unwrap();
    }

    pub fn debug<T: Debug>(&mut self, obj: &T) {
        writeln!(self.file, "buildserver | {:?}", obj).unwrap();
    }
}
