use std::{fs::File, io::Write};
use chrono::{Datelike, Timelike, Utc};

pub struct Log {
    file: File,
}

impl Log {
    pub fn new() -> Self {
        let now = Utc::now();
        let name = format!("twm_log_{}_{}_{}_{}_{}_{}",
            now.year_ce().1, now.month(), now.day(),
            now.hour(), now.minute(), now.second()
        );
        let file = std::fs::File::create(name).expect("create failed");
        Log { file }
    }

    pub fn write(&mut self, buf: &str) {
        self.file.write_all(format!("{}\n", buf).as_bytes()).unwrap();
    }
}