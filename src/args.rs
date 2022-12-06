use std::path::PathBuf;

use anyhow::Result;

pub struct Args {
    pub proxy_from: PathBuf,
    pub port: u16,
    pub prefixes: Vec<String>,
}

impl Args {
    pub fn parse() -> Result<Self> {
        let mut args = pico_args::Arguments::from_env();

        Ok(Args {
            proxy_from: args.value_from_str(["-f", "--from"])?,
            port: args.opt_value_from_str(["-p", "--port"])?.unwrap_or(10320),
            prefixes: args
                .opt_value_from_str::<_, String>("--prefixes")?
                .unwrap_or_default()
                .split(',')
                .map(|s| s.to_string())
                .collect(),
        })
    }
}
