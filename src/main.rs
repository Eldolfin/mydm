mod config;
mod desktops;
mod ui;

use anyhow::{anyhow, Context};
use base64::{prelude::BASE64_STANDARD, Engine as _};
use config::Config;
use desktops::list_desktops;
use log::debug;
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    env::{self, VarError},
    process::Command,
    sync::{Arc, Mutex},
};
use ui::LoginRequest;

#[derive(Debug, Serialize, Deserialize)]

struct EncodableEnv(HashMap<String, String>);

fn env_encoded() -> String {
    let env: HashMap<String, String> = HashMap::from_iter(
        ["DISPLAY", "PATH"]
            .iter()
            .filter_map(|var| Some((var.to_string(), env::var(var).ok()?))),
    );

    let json = serde_json::to_string(&EncodableEnv(env)).unwrap();

    BASE64_STANDARD.encode(json)
}

fn apply_env(encoded: &str) -> anyhow::Result<()> {
    let json = BASE64_STANDARD.decode(encoded)?;
    let json = String::from_utf8(json)?;
    let env: EncodableEnv = serde_json::from_str(&json)?;
    debug!("applying original env: {env:#?}");
    for (key, value) in env.0.iter() {
        std::env::set_var(key, value);
    }
    Ok(())
}

fn main() -> anyhow::Result<()> {
    debug!("env: {:#?}", env::vars().collect::<Vec<_>>());
    init_logger()?;
    let config = config::load()?;
    debug!("config: {config:#?}");

    let in_compositor = ["WAYLAND_DISPLAY", "WAYLAND_SOCKET", "DISPLAY"]
        .iter()
        .any(|res| env::var(res) != Err(VarError::NotPresent));

    if in_compositor {
        if let Some(arg) = std::env::args().nth(1) {
            apply_env(&arg).with_context(|| {
                format!(
                    "Could not apply encoded environment from argv[1] = `{}`",
                    arg
                )
            })?;
        }
        main_unwrapped(config)
    } else {
        let wrapped_cmd = format!(
            "{} -- {}",
            config.wayland.compositor,
            env::args()
                .chain([env_encoded()])
                .map(|arg| arg.to_string())
                .collect::<Vec<_>>()
                .join(" ")
        );
        let mut child = Command::new("/bin/sh")
            .arg("-c")
            .arg(wrapped_cmd)
            .spawn()
            .context("Could not spawn weston process")?;
        child.wait().context("Weston process failed")?;
        Ok(())
    }
}

/// runs the program, needs to be wrapped in a compositor if
fn main_unwrapped(config: Config) -> anyhow::Result<()> {
    let users = unsafe { uzers::all_users() };
    let users = users
        .filter(|user| {
            user.groups()
                .is_some_and(|groups| groups.iter().any(|g| g.name().to_str() == Some("users")))
        })
        .map(|user| user.name().to_str().unwrap().to_owned())
        .collect::<Vec<_>>();
    let client = Arc::new(Mutex::new(
        pam::Client::with_password("mydm").context("Could not init PAM client!")?,
    ));
    client.lock().unwrap().close_on_drop = false;
    let original_path = std::env::var("PATH").expect("PATH variable to be set");
    let desktops = list_desktops(&config.session_dir)?;
    let on_login = |LoginRequest {
                        login,
                        password,
                        desktop,
                    }| {
        debug!("{login} ({desktop:?}]): {}", "*".repeat(password.len()));
        let mut client = client.lock().unwrap();
        let user = uzers::get_user_by_name(&login).unwrap();
        client
            .conversation_mut()
            .set_credentials(login.clone(), password);
        client.authenticate().context("Authentication failed")?;
        client.open_session().context("Could not open a session")?;

        let new_path = std::env::var("PATH").expect("PATH variable to be set");
        let combined = format!("{new_path}:{original_path}");
        std::env::set_var("PATH", &combined);

        desktop.run_as(user)?;
        // TODO: maybe we should exit here as the desktop is launched
        Ok(())
    };

    ui::MyDm::new(ui::MyDmData {
        users,
        on_login,
        desktops,
    })
    .run()
    .map_err(|err| anyhow!("ui errored: {:?}", err))
}

fn init_logger() -> anyhow::Result<()> {
    // let target = Box::new(File::create("/tmp/mydm.log").context("Can't create log file")?);

    // env_logger::Builder::new()
    //     .target(env_logger::Target::Pipe(target))
    //     .filter(None, LevelFilter::Debug)
    //     .init();

    env_logger::init();
    Ok(())
}
