use super::ReqScopedState;
use serde_json::{json, Value};

pub trait LoggerInterface {
    fn info(&self, item: &str);
    fn warning(&self, item: &str);
    fn danger(&self, item: &str);
    fn debug(&self, item: &str);
}

#[derive(Clone, Debug)]

pub struct Logger<'a>(pub &'a ReqScopedState);

impl<'a> LoggerInterface for Logger<'a> {
    fn info(&self, item: &str) {
        log(&self.0, LogLevel::Info, item)
    }
    fn warning(&self, item: &str) {
        log(&self.0, LogLevel::Warning, item)
    }

    fn danger(&self, item: &str) {
        log(&self.0, LogLevel::Danger, item)
    }

    fn debug(&self, item: &str) {
        log(&self.0, LogLevel::Debug, item)
    }
}

#[derive(Clone, Debug)]
enum LogLevel {
    Info,
    Warning,
    Danger,
    Debug,
}

impl std::fmt::Display for LogLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let item = match self {
            LogLevel::Info => "info",
            LogLevel::Warning => "warning",
            LogLevel::Danger => "danger",
            LogLevel::Debug => "debug",
        };
        write!(f, "{}", item)
    }
}

fn log(ctx: &ReqScopedState, level: LogLevel, item: &str) {
    let mut map = ctx.log_member.clone();

    map.insert("log_level".to_string(), json!(level.to_string()));
    map.insert("message".to_string(), json!(item.to_string()));

    let tmp: Value = map.into();
    println!("{tmp}")
}
