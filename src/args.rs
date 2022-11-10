use std::path::PathBuf;

use anyhow::Result;

pub struct Args {
    pub proxy_from: PathBuf,
    pub port: u16,
    pub prefix: String,
}

impl Args {
    pub fn parse() -> Result<Self> {
        let mut args = pico_args::Arguments::from_env();

        Ok(Args {
            proxy_from: args.value_from_str(["-f", "--from"])?,
            port: args.opt_value_from_str(["-p", "--port"])?.unwrap_or(10320),
            prefix: args.opt_value_from_str("--prefix")?.unwrap_or_default(),
        })
    }
}
