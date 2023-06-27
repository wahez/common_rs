use std::{cell::RefCell, io::Write, path::PathBuf, sync::atomic::AtomicBool, time::SystemTime};

use chrono::{DurationRound, Local};
use fast_log::appender::Command;
use nix::sys::signal;

use crate::Error;

struct LogFile {
    target: String,
    file: std::fs::File,
    bytes_written: usize,
}

impl LogFile {
    fn write_all(&mut self, line: &str) -> Result<(), std::io::Error> {
        self.bytes_written += line.len();
        self.file.write_all(line.as_bytes())
    }
}

struct RotatingLogFiles {
    write_until: SystemTime,
    log_files: Vec<LogFile>,
}

impl RotatingLogFiles {
    fn flush(&mut self) {
        for f in &mut self.log_files {
            let _ = f.file.flush();
        }
    }
}

pub struct LogFileConfig {
    target: String,
    directory: PathBuf,
    postfix: String,
}

impl LogFileConfig {
    pub fn new(target: &str, directory: &std::path::Path, postfix: &str) -> LogFileConfig {
        LogFileConfig {
            target: target.to_string(),
            directory: directory.to_path_buf(),
            postfix: postfix.to_string(),
        }
    }
    fn get_new_file_path(&self, base_name: &str) -> PathBuf {
        let now = Local::now();
        let mut path = self.directory.to_path_buf();
        path.push(format!(
            "{}-{base_name}{}.log",
            now.format("%Y%m%d-%H%M"),
            self.postfix
        ));
        path
    }
}

pub struct LogRollerConfig {
    pub base_name: String,
    pub roll_size: usize,
    pub roll_interval: chrono::Duration,
    pub outputs: Vec<LogFileConfig>,
}

impl LogRollerConfig {
    fn calc_roll_time(&self) -> SystemTime {
        let now = Local::now();
        let roll_time = now.duration_trunc(self.roll_interval).unwrap() + self.roll_interval;
        roll_time.into()
    }

    fn open_file(&self, target: &str, log_output: &LogFileConfig) -> Result<LogFile, Error> {
        let path = log_output.get_new_file_path(&self.base_name);
        let file = std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&path)
            .map_err(|e| Error::LogFileInit {
                message: e.to_string(),
                path,
            })?;
        Ok(LogFile {
            target: target.to_string(),
            file,
            bytes_written: 0,
        })
    }

    fn create_log_writer(&self) -> Result<RotatingLogFiles, Error> {
        let log_files = self
            .outputs
            .iter()
            .map(|log_output| self.open_file(&log_output.target, log_output))
            .collect::<Result<_, _>>()?;
        Ok(RotatingLogFiles {
            write_until: self.calc_roll_time(),
            log_files,
        })
    }
}

struct LogRoller {
    config: LogRollerConfig,
    writer: RotatingLogFiles,
}

impl LogRoller {
    fn new(config: LogRollerConfig) -> Result<LogRoller, Error> {
        let writer = config.create_log_writer()?;
        Ok(LogRoller { config, writer })
    }
    fn check_log_roll(&mut self) {
        if ROLL_LOG_FLAG.load(std::sync::atomic::Ordering::Acquire) {
            ROLL_LOG_FLAG.store(false, std::sync::atomic::Ordering::Release);
            match self.config.create_log_writer() {
                Err(e) => {
                    log::error!("Could not roll log file after SIGUSR1: {e}");
                }
                Ok(w) => {
                    self.writer = w;
                }
            }
        }
        let now = SystemTime::now();
        if now > self.writer.write_until {
            match self.config.create_log_writer() {
                Err(e) => {
                    log::error!("Could not roll log file at roll time: {e}");
                    self.writer.write_until = self.config.calc_roll_time();
                }
                Ok(w) => self.writer = w,
            }
        }
    }
    fn write_line(&mut self, line: &str, target: &str) -> Result<(), std::io::Error> {
        let log_file = {
            if let Some(f) = self
                .writer
                .log_files
                .iter_mut()
                .find(|f| f.target == target)
            {
                f
            } else if let Some(f) = self.writer.log_files.iter_mut().last() {
                f
            } else {
                panic!("No log_files set on logger");
            }
        };
        if log_file.bytes_written + line.len() <= self.config.roll_size {
            log_file.write_all(line)
        } else {
            match self.config.create_log_writer() {
                Err(e) => {
                    log::error!("Could not roll log file due to size: {e}");
                    log_file.bytes_written /= 2; // not true but prevents the next log roll for a while
                }
                Ok(w) => self.writer = w,
            }
            if let Some(log_file) = self
                .writer
                .log_files
                .iter_mut()
                .find(|f| f.target == target)
            {
                log_file.write_all(line)
            } else if let Some(log_file) = self.writer.log_files.iter_mut().last() {
                log_file.write_all(line)
            } else {
                panic!("No log_files set on logger");
            }
        }
    }
}

pub struct LogRollingAppender {
    cell: RefCell<LogRoller>,
}

impl LogRollingAppender {
    pub fn new(config: LogRollerConfig) -> Result<LogRollingAppender, Error> {
        Ok(Self {
            cell: RefCell::new(LogRoller::new(config)?),
        })
    }
}

impl fast_log::appender::LogAppender for LogRollingAppender {
    fn do_logs(&mut self, records: &[fast_log::appender::FastLogRecord]) {
        let mut roller = self.cell.borrow_mut();
        roller.check_log_roll();
        for x in records {
            match x.command {
                Command::CommandRecord => {
                    roller
                        .write_line(&x.formated, &x.target)
                        .expect("Could not write to logfile");
                }
                Command::CommandExit | Command::CommandFlush(_) => {
                    roller.writer.flush();
                }
            }
        }
    }
}

static ROLL_LOG_FLAG: AtomicBool = AtomicBool::new(false);

extern "C" fn set_roll_log(
    _: nix::libc::c_int,
    _: *mut nix::libc::siginfo_t,
    _: *mut nix::libc::c_void,
) {
    ROLL_LOG_FLAG.store(true, std::sync::atomic::Ordering::Relaxed);
}

pub fn init_file(config: &crate::CommonConfig, instance: &str) -> Result<(), Error> {
    unsafe {
        let action = signal::SigAction::new(
            signal::SigHandler::SigAction(set_roll_log),
            signal::SaFlags::empty(),
            signal::SigSet::empty(),
        );
        signal::sigaction(signal::SIGUSR1, &action)
    }
    .map_err(|e| Error::SignalInit(e.to_string()))?;
    let roller_config = LogRollerConfig {
        base_name: instance.to_string(),
        roll_size: 2_000_000_000,
        roll_interval: chrono::Duration::days(1),
        outputs: vec![
            LogFileConfig::new("METRICS", &config.log_path, "_metrics"),
            LogFileConfig::new("ALERTS", &config.alerts_path, "_alerts"),
            LogFileConfig::new("NOTIFICATIONS", &config.alerts_path, "_devnotification"),
            LogFileConfig::new("", &config.log_path, ""),
        ],
    };
    let log_config = fast_log::Config::new()
        .chan_len(Some(config.log_channel_size))
        .level(config.log_level)
        .format(crate::logging::LOG_FORMATTER)
        .custom(LogRollingAppender::new(roller_config)?);
    fast_log::init(log_config).map_err(|e| Error::LogFileInit {
        message: e.to_string(),
        path: config.log_path.clone(),
    })?;
    Ok(())
}
