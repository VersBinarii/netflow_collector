use super::syslog::{self, Logger, Facility, Severity};

pub struct Log(Logger);

impl Log {
    pub fn new() -> Self {
        let writer = match syslog::unix(Facility::LOG_USER) {
            Ok(writer) => writer,
            Err(e) => {
                panic!("Failed to connect to syslog. Err: {}", e)
            },
        };
        
        Log(*writer)
    }

    pub fn info(&self, message: &str) {
        self.0.send(Severity::LOG_INFO, message).map_err(|_| println!("Failed to write to syslog.")).ok();
    }
}
