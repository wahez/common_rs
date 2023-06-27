use std::{env, path::Path};

use auditable_serde::Package;

use crate::Error;

fn get_packages() -> Result<Vec<Package>, Error> {
    let info = auditable_info::audit_info_from_file(
        Path::new(&env::args().next().unwrap()),
        Default::default(),
    );
    let info = info.map_err(|_| Error::BuildInfoMissing)?;
    Ok(info.packages)
}

pub fn print_build_info() -> Result<(), Error> {
    println!("Build info:");
    println!("Build user: {}", env!("USER"));
    println!(
        "Build host: {}",
        option_env!("HOSTNAME").unwrap_or("Unknown hostname")
    );
    println!("Dependencies: ");
    for package in get_packages()? {
        println!("    {} {}", package.name, package.version);
    }
    Ok(())
}

pub fn log_build_info() -> Result<(), Error> {
    log::info!("Build info:");
    log::info!("Build user: {}", env!("USER"));
    log::info!(
        "Build host: {}",
        option_env!("HOSTNAME").unwrap_or("Unknown hostname")
    );
    log::info!("Dependencies: ");
    for package in get_packages()? {
        log::info!("{} {}", package.name, package.version,);
    }
    Ok(())
}
