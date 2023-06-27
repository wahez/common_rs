use std::{fs::copy, path::PathBuf};

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "PascalCase")]
struct MyConfig {
    common_config: common_rust::CommonConfig,
    input_path: PathBuf,
    output_path: PathBuf,
}

common_rust::gen_config_reader!(serde_xml MyConfig);

fn copy_file(config: MyConfig) -> Result<(), Box<dyn std::error::Error>> {
    log::info!("Running my application with config {config:?}");
    log::debug!(
        "Copying from {} to {}",
        config.input_path.display(),
        config.output_path.display()
    );
    assert_ne!(config.input_path, config.output_path);
    if let Err(e) = copy(config.input_path, config.output_path) {
        common_rust::alert_ae!(code: "TEST-123", "Could not copy: {e}");
    }
    log::error!("Sleeping");
    log::logger().flush();
    std::thread::sleep(std::time::Duration::from_secs(2));
    log::error!("Waking up");
    panic!("Testing string panic {}", 1);
}

fn main() {
    common_rust::run_application(copy_file);
}
