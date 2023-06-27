use std::path::PathBuf;

#[derive(Debug)]
pub enum Error {
    BuildInfoMissing,
    CommandlineInvalid(String),
    ConfigOpenError { path: PathBuf, message: String },
    ConfigParseError(String),
    ConfigInvalid { path: PathBuf, message: String },
    DaemonizationFailed(String),
    LogConsoleInit(String),
    LogFileInit { message: String, path: PathBuf },
    LogFilePath(PathBuf),
    SignalInit(String),
    UserRunFailed(String),
}

impl std::error::Error for Error {}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use Error::*;
        match self {
            BuildInfoMissing => write!(f, "Could not read build_info"),
            CommandlineInvalid(s) => write!(f, "Invalid commandline {s}"),
            ConfigOpenError { path, message } => write!(
                f,
                "Could not open config file {}: {}",
                path.display(),
                message
            ),
            ConfigParseError(s) => write!(f, "Config parse error {s}"),
            ConfigInvalid { path, message } => {
                write!(f, "Invalid config file {}: {}", path.display(), message)
            }
            DaemonizationFailed(s) => write!(f, "Could not daemonize: {s}"),
            LogConsoleInit(s) => write!(f, "Could not init console logging: {s}"),
            LogFileInit { message, path } => write!(
                f,
                "Could not init file logging to {}: {}",
                path.display(),
                message
            ),
            LogFilePath(path) => write!(f, "Could not get log file path of {}", path.display()),
            SignalInit(s) => write!(f, "Installing signal handler returned error: {s}"),
            UserRunFailed(s) => write!(f, "User run function returned error: {s}"),
        }
    }
}
