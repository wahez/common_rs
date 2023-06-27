use fast_log::appender::{Command, FastLogRecord, RecordFormat};

use crate::Error;

pub fn init_stdout(config: &crate::CommonConfig) -> Result<(), Error> {
    let log_config = fast_log::Config::new()
        .chan_len(Some(config.log_channel_size))
        .level(config.log_level)
        .format(LOG_FORMATTER)
        .console();
    fast_log::init(log_config)
        .map_err(|e| Error::LogConsoleInit(format!("Could not init console logger: {e}")))?;
    Ok(())
}

pub(crate) struct LogFormatter {}

impl LogFormatter {
    fn level_to_str(level: log::Level) -> &'static str {
        use log::Level::*;
        match level {
            Error => "Error",
            Warn => "Warn",
            Info => "Info",
            Debug => "Debug",
            Trace => "Trace",
        }
    }
}

pub(crate) const LOG_FORMATTER: LogFormatter = LogFormatter {};

impl RecordFormat for LogFormatter {
    fn do_format(&self, arg: &mut FastLogRecord) {
        match &arg.command {
            Command::CommandRecord => {
                let now = chrono::DateTime::<chrono::Local>::from(arg.now).time();
                arg.formated = format!(
                    "{} [{:5}] [{}] {}\n",
                    &now,
                    Self::level_to_str(arg.level),
                    arg.module_path,
                    arg.args
                );
            }
            Command::CommandExit => {}
            Command::CommandFlush(_) => {}
        }
    }
}

#[macro_export]
macro_rules! alert_ae {
    // message of the alert is formatted immediately
    (code: $code:expr, $($arg:tt)+) => (log::log!(target: "ALERTS", log::Level::Error, "[{}] {}", $code, format!($($arg)+)));
}

#[macro_export]
macro_rules! alert_ae_panic {
    // message of the alert is formatted immediately
    (code: $code:expr, $($arg:tt)+) => ({
        $crate::alert_ae!(code: $code, $($arg)+);
        panic!("Fatal AE Alert: [{}] {}", $code, format!($($arg)+));
    }
    );
}

#[macro_export]
macro_rules! notify_dev {
    // message of the alert is formatted immediately
    (code: $code:expr, $($arg:tt)+) => (log::log!(target: "NOTIFICATIONS", log::Level::Error, "[{}] {}", $code, format!($($arg)+)));
}

#[macro_export]
macro_rules! metric {
    ($($arg:tt)+) => (log::log!(target: "METRICS", log::Level::Info, $($arg)+));
}
