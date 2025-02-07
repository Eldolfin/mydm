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
    // TODO: put a default value?
    pub session_dir: String,
    pub wayland: Wayland,
    pub x11: X11,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Wayland {
    /// command to run the compositor
    pub compositor: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct X11 {
    pub display_command: String,
    pub displaystop_command: String,
    pub server_path: String,
    pub session_command: String,
    pub session_dir: String,
    pub xauth_path: String,
    pub xephyr_path: String,
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
    // real config
    #[case(
        r#"
    session_dir: /nix/store/v7w6y4g5ah4416p43x8cj7x3abh7vc9q-desktops/share
    wayland:
      compositor: /nix/store/ib1nnrvy1c6jzb7c81423002rpad2wfa-weston-14.0.1/bin/weston
        --shell=kiosk -c /nix/store/kqq442qlxaldjvvdr2zmin3s0d9j5vz9-weston.ini
    x11:
      display_command: /nix/store/6sxgfy8ril126hkayldrljzwmqfs5c9h-Xsetup
      displaystop_command: /nix/store/iwzpzrx9kd3xn7rj325nizfmqp6k96g4-Xstop
      server_path: /nix/store/ngxp14rv069npg9g12v57kln5ksyf4j3-xserver-wrapper
      session_command: /nix/store/qw5xv599fi8d89sl4ndjhk2hpfbdcbss-xsession-wrapper
      session_dir: /nix/store/v7w6y4g5ah4416p43x8cj7x3abh7vc9q-desktops/share/xsessions
      xauth_path: /nix/store/rkxfjs4mxpmvxp8j6d3hgcsj6y152q9s-xauth-1.1.3/bin/xauth
      xephyr_path: /nix/store/4vschn3vflf0fq6x38x2n3fgdzbv2w8x-xorg-server-21.1.15/bin/Xephyr
    "#
    )]
    fn test_config_load(#[case] content: &str) -> anyhow::Result<()> {
        let mut config_file = NamedTempFile::new().unwrap();
        write!(config_file, "{}", content)?;
        load_from(config_file.path())?;
        Ok(())
    }

    // TODO: default session_dir
    // #[test]
    // fn test_no_config() -> anyhow::Result<()> {
    //     load_from("/etc/nonexistant/config.yml")?;
    //     Ok(())
    // }
}
