use clap::Parser;
use std::path::PathBuf;

#[derive(Debug, Parser)]
#[clap(disable_help_flag = true)]
pub struct CommonArgs {
    #[arg(short, long)]
    pub help: bool,
    #[arg(short, long)]
    pub config: Option<PathBuf>,
    #[arg(short, long)]
    pub foreground: bool,
    #[arg(short, long)]
    pub stdout: bool,
    #[arg(short = 'a', long)]
    pub validate: bool,
    #[arg(short, long)]
    pub version: bool,
}

// implement this if you need custom commandline arguments
pub trait CommandlineArgs: std::fmt::Debug + Sized {
    type Err: std::fmt::Debug + std::fmt::Display;

    // parse and validate custom commandline parameters here
    // don't print or log if everything is ok
    fn parse_args() -> Result<Self, Self::Err>;
    fn get_common_args(&self) -> &CommonArgs;
    fn print_help();
}

impl CommandlineArgs for CommonArgs {
    type Err = clap::error::Error;
    fn parse_args() -> Result<Self, Self::Err> {
        CommonArgs::try_parse()
    }
    fn get_common_args(&self) -> &CommonArgs {
        self
    }
    fn print_help() {
        <Self as clap::CommandFactory>::command()
            .print_help()
            .unwrap();
    }
}

#[macro_export]
macro_rules! gen_args_reader {
    (clap $Args:ty) => {
        impl $crate::CommandlineArgs for $Args {
            type Err = clap::error::Error;
            fn parse_args() -> Result<Self, Self::Err> {
                Self::try_parse()
            }
            fn get_common_args(&self) -> &$crate::CommonArgs {
                &self.common_args
            }
            fn print_help() {
                <Self as clap::CommandFactory>::command()
                    .print_help()
                    .unwrap();
            }
        }
    };
}
