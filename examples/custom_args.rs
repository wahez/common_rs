use chrono::{Datelike, Days, NaiveDate, Weekday};
use clap::Parser;
use std::path::PathBuf;

#[derive(Debug, Parser)]
struct MyArgs {
    #[command(flatten)]
    common_args: common_rust::CommonArgs,
    #[arg(short, long)]
    date: Option<NaiveDate>,
}

fn previous_working_day() -> NaiveDate {
    let today = chrono::Local::now().date_naive();
    let days_back = match today.weekday() {
        Weekday::Sat | Weekday::Sun => {
            panic!("Did not expect to run during the weekend")
        }
        Weekday::Mon => 3,
        _ => 1,
    };
    today - Days::new(days_back)
}

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "PascalCase")]
struct MyConfig {
    common_config: common_rust::CommonConfig,
    input_path: PathBuf,
    output_path: PathBuf,
}

fn run_with_custom_args(args: MyArgs, config: MyConfig) -> Result<(), Box<dyn std::error::Error>> {
    log::info!("Running my application with args {args:?} and config {config:?}");
    log::debug!(
        "Working for date {:?}",
        args.date.unwrap_or_else(previous_working_day)
    );
    log::debug!("Opening input_path {:?}", config.input_path);
    log::debug!("Opening output_path {:?}", config.output_path);
    log::error!("Sleeping");
    log::logger().flush();
    unsafe { nix::libc::sleep(10) };
    log::error!("Waking up");
    Ok(())
}

common_rust::gen_args_reader!(clap MyArgs);
common_rust::gen_config_reader!(serde_xml MyConfig);

fn main() {
    common_rust::run_application_with_args(run_with_custom_args);
}
