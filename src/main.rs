mod config;
mod ui;

use anyhow::{anyhow, Context};
use log::debug;
use std::{
    env::{self, VarError},
    process::Command,
    sync::{Arc, Mutex},
};

fn main() -> anyhow::Result<()> {
    debug!("env: {:#?}", env::vars().collect::<Vec<_>>());
    init_logger()?;
    let config = config::load()?;
    debug!("config: {config:#?}");

    let in_compositor = ["WAYLAND_DISPLAY", "WAYLAND_SOCKET", "DISPLAY"]
        .iter()
        .any(|res| env::var(res) != Err(VarError::NotPresent));

    if in_compositor || config.wayland.is_none() {
        main_unwrapped()
    } else {
        let mut child = Command::new("/bin/sh")
            .arg("-c")
            .arg(format!(
                "{} -- {}",
                config.wayland.unwrap().compositor,
                env::args()
                    .map(|arg| arg.to_string())
                    .collect::<Vec<_>>()
                    .join(" ")
            ))
            .spawn()
            .context("Could not spawn weston process")?;
        child.wait().context("Weston process failed")?;
        Ok(())
    }
}

/// runs the program, needs to be wrapped in a compositor if
fn main_unwrapped() -> anyhow::Result<()> {
    let users = unsafe { uzers::all_users() };
    let users = users
        .filter(|user| {
            user.groups()
                .is_some_and(|groups| groups.iter().any(|g| g.name().to_str() == Some("users")))
        })
        .map(|user| user.name().to_str().unwrap().to_owned())
        .collect::<Vec<_>>();
    let client = Arc::new(Mutex::new(
        pam::Client::with_password("system-auth").context("Could not init PAM client!")?,
    ));

    ui::MyDm::new(users, |login, password| {
        debug!("{login}: {}", "*".repeat(password.len()),);
        let mut client = client.lock().unwrap();
        client.conversation_mut().set_credentials(login, password);
        client.authenticate().context("Authentication failed")?;
        client.open_session().context("Could not open a session")?;

        // let user = get_user_by_name(&login).unwrap();
        // let error = Command::new("/bin/bash")
        //     .uid(user.uid())
        //     .gid(user.primary_group_id())
        //     .exec();
        Ok(())
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
