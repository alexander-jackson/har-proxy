use std::path::PathBuf;

use anyhow::Result;

pub struct Args {
    pub archive: PathBuf,
}

impl Args {
    pub fn parse() -> Result<Self> {
        let mut args = pico_args::Arguments::from_env();

        Ok(Args {
            archive: args.value_from_str("--archive")?,
        })
    }
}
