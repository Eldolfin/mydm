use anyhow::Context;
use log::warn;
use std::fs::create_dir_all;
use std::os::unix::process::CommandExt;
use std::path::PathBuf;
use std::{ffi::OsStr, process::Command};
use uzers::os::unix::UserExt;
use walkdir::WalkDir;

pub fn list_desktops(session_dir: &str) -> anyhow::Result<Vec<DesktopEntry>> {
    Ok(WalkDir::new(session_dir)
        .follow_links(true)
        .into_iter()
        .filter_map(|res| res.ok())
        .map(|entry| entry.into_path())
        .filter(|path| path.is_file() && path.extension() == Some(OsStr::new("desktop")))
        .map(|path| {
            (
                path.clone(),
                xdgkit::desktop_entry::DesktopEntry::new(path.display().to_string()),
            )
        })
        .filter_map(|(path, entry)| {
            let name = entry.name.clone();
            let display_server = match path
                .parent()
                .unwrap()
                .file_name()
                .unwrap()
                .to_str()
                .unwrap()
            {
                "xsessions" => DisplayServer::X11,
                "wayland-sessions" => DisplayServer::Wayland,
                dir => todo!("Handle desktops with parent dir {dir}"),
            };
            DesktopEntry::try_from(display_server, entry)
                .inspect_err(|err| {
                    warn!(
                        "desktop entry {}: {:?}",
                        name.unwrap_or("noname".to_owned()),
                        err
                    )
                })
                .ok()
        })
        .collect())
}

#[derive(Clone, Copy, Debug)]
enum DisplayServer {
    Wayland,
    X11,
}

#[derive(Clone, Debug)]
pub struct DesktopEntry {
    pub name: String,
    exec: String,
    display_server: DisplayServer,
}

impl DisplayServer {
    fn xdg_session_value(&self) -> &'static str {
        match self {
            DisplayServer::Wayland => "wayland",
            DisplayServer::X11 => "x11",
        }
    }
}
impl DesktopEntry {
    pub fn run_as(&self, user: uzers::User) -> anyhow::Result<()> {
        let mut cmd = Command::new("/bin/sh");
        let mut cmd = cmd.arg("-c").arg(format!(
            "[ -f /etc/profile ] && . /etc/profile; [ -f $HOME/.profile ] && . $HOME/.profile; exec {}",
            &self.exec
        ));
        match self.display_server {
            DisplayServer::Wayland => (),
            DisplayServer::X11 => self.run_x11(&mut cmd, &user)?,
        };
        let mut child = cmd
            .uid(user.uid())
            .gid(user.primary_group_id())
            .current_dir(user.home_dir())
            .spawn()
            .with_context(|| format!("Failed to launch `{:?}`", self))?;
        child.wait()?;
        Ok(())
        // .exec();
        // anyhow!("Failed to launch `{:?}`: {err:#}", self)
    }

    fn run_x11(&self, child: &mut &mut Command, user: &uzers::User) -> anyhow::Result<()> {
        // xauth
        let xdg_runtime_dir = format!("/run/user/{}", user.uid());
        /* if let Ok(runtime_dir) = std::env::var("XDG_RUNTIME_DIR") {
            runtime_dir
        } else if let Ok(cfg_home) = std::env::var("XDG_CONFIG_HOME") {
            todo!("handle no XDG_RUNTIME_DIR")
            // format!("{cfg_home}/mydm")
        } else {
            todo!("handle no XDG_RUNTIME_DIR")
        }; */
        let xauth_path = PathBuf::from(&xdg_runtime_dir).join("mydmxauth");
        let xauth_parent = xauth_path.parent().unwrap(); // == xauth_dir but normalized
        create_dir_all(xauth_parent).with_context(|| {
            format!(
                "Could not create xauth parent dir (at {})",
                xauth_parent.display()
            )
        })?;

        // File::create(xauth_path).unwrap();
        // nix::unistd::chown(xauth_parent, Some(user.uid()), Some(user.gid()))
        //     .with_context(|| format!("Could not chown xauth_parent: {}", xauth_parent.display()))?;
        child
            .env("XDG_SESSION_TYPE", self.display_server.xdg_session_value())
            .env("XDG_SESSION_CLASS", "user")
            .env("XDG_SEAT", "seat0")
            .env("XDG_SESSION_ID", "1")
            .env("XDG_RUNTIME_DIR", xdg_runtime_dir)
            .env("DISPLAY", ":0") // FIXME: hardcode 🙄
            .env("XAUTHORITY", xauth_path.display().to_string());
        Ok(())
    }
}

#[derive(Debug)]
#[allow(dead_code)]
pub struct MissingField(&'static str);

impl DesktopEntry {
    fn try_from(
        display_server: DisplayServer,
        entry: xdgkit::desktop_entry::DesktopEntry,
    ) -> Result<Self, MissingField> {
        Ok(Self {
            name: entry.name.ok_or(MissingField("name"))?,
            exec: entry.exec.ok_or(MissingField("exec"))?,
            display_server,
        })
    }
}
