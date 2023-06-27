use std::{io::Read, path::PathBuf};

use serde::Deserialize;
use serde_with::serde_as;

// this can be read from a file like this:
//   use serde_xml_rs::from_reader;
//   let file = File::open(common_args.get_config_file())?;
//   let mut reader = BufReader::new(file);
//   let config = from_reader(reader)?;
#[serde_as]
#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct CommonConfig {
    pub log_channel_size: usize,
    #[serde_as(as = "serde_with::DisplayFromStr")]
    pub log_level: log::LevelFilter,
    pub log_path: PathBuf,
    pub alerts_path: PathBuf,
}

pub trait Config: std::fmt::Debug + Sized {
    type Err: std::fmt::Debug + std::fmt::Display;

    // parse and validate custom commandline parameters here
    // don't print or log if everything is ok
    fn parse<R: Read>(reader: R) -> Result<Self, Self::Err>;
    // if you need to do extra validation
    // if prod_run is false, don't check for presence of other files etc
    fn is_valid(&self, _prod_run: bool) -> Result<(), String> {
        Ok(())
    }
    fn get_common_config(&self) -> &CommonConfig;
}

#[macro_export]
macro_rules! gen_config_reader {
    (serde_xml $Config:ty) => {
        impl $crate::Config for $Config {
            type Err = serde_xml_rs::Error;
            fn parse<R: std::io::Read>(reader: R) -> Result<Self, Self::Err> {
                serde_xml_rs::from_reader(reader)
            }
            fn is_valid(&self, _prod_run: bool) -> Result<(), String> {
                Ok(())
            }
            fn get_common_config(&self) -> &$crate::CommonConfig {
                &self.common_config
            }
        }
    };
}
