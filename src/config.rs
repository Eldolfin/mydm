use std::path::Path;

use anyhow::Context;
use figment::{
    providers::{Format, Yaml},
    Figment,
};
use serde::Deserialize;

const CONFIG_PATH: &str = "/etc/mydm/config.yml";

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    pub wayland: Option<Wayland>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Wayland {
    /// command to run the compositor
    pub compositor: String,
}

pub fn load() -> anyhow::Result<Config> {
    load_from(CONFIG_PATH)
}

fn load_from(path: impl AsRef<Path>) -> Result<Config, anyhow::Error> {
    Figment::new()
        .merge(Yaml::file(path))
        .extract()
        .with_context(|| format!("Could not load config file `{CONFIG_PATH}`"))
}

#[cfg(test)]
mod tests {
    use tempfile::NamedTempFile;

    use super::load_from;
    use rstest::rstest;
    use std::io::Write;

    #[rstest]
    #[case(
        r#"
wayland:
    compositor: "weston --shell=kiosk"
"#
    )]
    #[case(r#""#)]
    fn test_config_load(#[case] content: &str) -> anyhow::Result<()> {
        let mut config_file = NamedTempFile::new().unwrap();
        write!(config_file, "{}", content)?;
        load_from(config_file.path())?;
        Ok(())
    }

    #[test]
    fn test_no_config() -> anyhow::Result<()> {
        load_from("/etc/nonexistant/config.yml")?;
        Ok(())
    }
}
