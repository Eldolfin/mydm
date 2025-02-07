use anyhow::{anyhow, Context};
use log::warn;
use std::os::unix::process::CommandExt;
use std::{ffi::OsStr, process::Command};
use walkdir::WalkDir;

pub fn list_desktops(session_dir: &str) -> anyhow::Result<Vec<DesktopEntry>> {
    Ok(WalkDir::new(session_dir)
        .follow_links(true)
        .into_iter()
        .filter_map(|res| res.ok())
        .map(|entry| entry.into_path())
        .filter(|path| path.is_file() && path.extension() == Some(OsStr::new("desktop")))
        .map(|path| path.display().to_string())
        .map(xdgkit::desktop_entry::DesktopEntry::new)
        .filter_map(|entry| {
            let name = entry.name.clone();
            entry
                .try_into()
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

#[derive(Clone, Debug)]
pub struct DesktopEntry {
    pub name: String,
    exec: String,
}

impl DesktopEntry {
    pub(crate) fn run_as(&self, user: uzers::User) -> anyhow::Result<()> {
        // TODO: not sure if it's ok to drop it here
        let mut child = Command::new("/bin/sh").arg("-c").arg(format!(
            "[ -f /etc/profile ] && . /etc/profile; [ -f $HOME/.profile ] && . $HOME/.profile; exec {}",
            &self.exec
        ))
            .env("XDG_RUNTIME_DIR", format!("/run/user/{}", user.uid()))
            .uid(user.uid())
            .gid(user.primary_group_id())
            .spawn()
            .with_context(|| format!("Failed to launch `{:?}`", self))?;
        child.wait()?;
        Ok(())
        // .exec();
        // anyhow!("Failed to launch `{:?}`: {err:#}", self)
    }
}

#[derive(Debug)]
pub struct MissingField(&'static str);

impl TryFrom<xdgkit::desktop_entry::DesktopEntry> for DesktopEntry {
    type Error = MissingField;

    fn try_from(value: xdgkit::desktop_entry::DesktopEntry) -> Result<Self, Self::Error> {
        Ok(Self {
            name: value.name.ok_or(MissingField("name"))?,
            exec: value.exec.ok_or(MissingField("exec"))?,
        })
    }
}
