use std::{
    fs::File,
    panic::{self, UnwindSafe},
    path::PathBuf,
};

use crate::{
    build_info::{log_build_info, print_build_info},
    CommandlineArgs, CommonArgs, Config, Error,
};

pub fn get_instance_name() -> Option<String> {
    let binary_path = PathBuf::from(std::env::args().next()?);
    Some(binary_path.file_name()?.to_str()?.to_string())
}

pub fn try_run<Args, Conf, Run>(run: Run) -> Result<(), Error>
where
    Args: CommandlineArgs + UnwindSafe,
    Conf: Config + UnwindSafe,
    Run: FnOnce(Args, Conf) -> Result<(), Box<dyn std::error::Error>> + UnwindSafe,
{
    let args = Args::parse_args().map_err(|e| Error::CommandlineInvalid(e.to_string()))?;
    let common_args = args.get_common_args();
    if common_args.help {
        Args::print_help();
    } else if common_args.version {
        print_build_info()?;
    } else {
        let instance = get_instance_name().expect("Could not find instance name");
        let default_config_file_path = PathBuf::from(format!("{instance}.xml"));
        let config_file_path = common_args
            .config
            .as_ref()
            .unwrap_or(&default_config_file_path);
        let config_file = File::open(config_file_path).map_err(|e| Error::ConfigOpenError {
            path: config_file_path.clone(),
            message: e.to_string(),
        })?;
        let config = Conf::parse(config_file).map_err(|e| Error::ConfigInvalid {
            path: config_file_path.clone(),
            message: e.to_string(),
        })?;
        if common_args.validate {
            config.is_valid(false).map_err(|e| Error::ConfigInvalid {
                path: config_file_path.clone(),
                message: e.to_string(),
            })?;
        } else {
            config.is_valid(true).map_err(|e| Error::ConfigInvalid {
                path: config_file_path.clone(),
                message: e.to_string(),
            })?;
            if !common_args.stdout && !common_args.foreground {
                nix::unistd::daemon(true, true)
                    .map_err(|e| Error::DaemonizationFailed(e.to_string()))?;
            }
            let common_config = &config.get_common_config();
            if common_args.stdout {
                crate::log_init_stdout(common_config)?;
                log::info!("Logging to stdout");
            } else {
                crate::log_init_file(common_config, &instance)?;
                log::info!("Opened logfile");
            }
            log::info!("Started with arguments: {args:?}");
            log::info!("Configuration: {config:?}");
            log_build_info()?;
            log::info!("Starting application");

            log::logger().flush();
            match panic::catch_unwind(move || run(args, config)) {
                Err(panic_err) => {
                    if let Some(msg) = panic_err.downcast_ref::<&str>() {
                        log::error!("Detected panic, sleeping 1s. Error: {msg}");
                    } else {
                        log::error!("Detected panic, sleeping 1s.");
                    }
                    log::logger().flush();
                    std::thread::sleep(std::time::Duration::from_secs(1));
                    panic::resume_unwind(panic_err);
                }
                Ok(Err(user_err)) => Err(Error::UserRunFailed(user_err.to_string()))?,
                Ok(Ok(())) => {}
            };
            log::info!("Finished application");
            log::logger().flush();
        }
    }
    Ok(())
}

pub fn run_application_with_args<Args, Conf, Run>(run: Run)
where
    Args: CommandlineArgs + UnwindSafe,
    Conf: Config + UnwindSafe,
    Run: FnOnce(Args, Conf) -> Result<(), Box<dyn std::error::Error>> + UnwindSafe,
{
    if let Err(err) = try_run(run) {
        eprintln!("Error executing application: {}", err);
        log::error!("Error executing application: {}", err);
        log::logger().flush();
        std::process::exit(1);
    } else {
        log::logger().flush();
    }
}

pub fn run_application<Conf, Run>(run: Run)
where
    Conf: Config + UnwindSafe,
    Run: FnOnce(Conf) -> Result<(), Box<dyn std::error::Error>> + UnwindSafe,
{
    run_application_with_args(|_: CommonArgs, config: Conf| run(config));
}
