# common_rs

This is a common application library for rust.

# To use
## Add dependency
It can be used as any other dependency (artifactory, crates, git-url or local path in your Cargo.toml).
For the build info to work, you need to build your application like:
    __cargo auditable build__ and
    __cargo auditable run__.

## Define config
Easiest way to read a config file:
 - define your own config struct and add serde macros
 - use a macro to generate the config reader
 - call the main function.

```
#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "PascalCase")]
struct MyConfig {
    common_config: common_rust::CommonConfig,
    input_path: PathBuf,
    output_path: PathBuf,
}
common_rust::gen_config_reader!(serde_xml MyConfig);
```
If you need to add validation, you can implement the Config trait for your config struct instead of using the macro.

## Run the main function
```
fn my_main_function(config: MyConfig) -> Result<(), Box<dyn std::error::Error> {
    Err("This needs an implementation")?;
}

fn main() {
    common_rust::run_application(my_main_function);
}
```

# Examples
There are a few example applications in /examples/
- copy_file.rs is a fully featured application that copies a file to another path. Nothing special but it does support logging, config file reading, -h and AE-alerts.
- custom_args.rs is an application that just sleeps, but has some custom commandline arguments.

# Features

## build_info
This gives support for -v. The dependencies are embedded through cargo auditable. This means that it will support getting the dependencies out of the binary (without access to the repo). And it can do an online vulnerability check using 3rd party tools (cargo audit).

## logging
Applications and libraries can just use the log-crate and the commonly used macros info!(...) etc. Logfiles are rotated at midnight and on SIGUSR1.
There is also some other macros: alert_ae, alert_ae_panic, notify_dev, metric.
The current implementation uses fast_log and a logging thread.

## config reading
Config reading can be completely customized. But doing it with serde_xml is extremely easy. See copy_file.rs and the example above. For custom validation or different formats, you can implement the Config trait for your own struct in any way you like.

## custom commandline arguments
For custom commandline arguments, you can do the same but with clap (see example in custom_args.rs).