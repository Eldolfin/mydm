use anyhow::Context as _;
use log::debug;

use crate::ui::LoginRequest;

pub fn auth() -> impl Fn(LoginRequest) -> Result<(), anyhow::Error> {
    |LoginRequest {
         login,
         password,
         desktop,
     }| {
        debug!("{login} ({desktop:?}]): {}", "*".repeat(password.len()));
        let mut client = pam::Client::with_password("mydm").expect("Could not init PAM client!");
        client.close_on_drop = false;
        let user = uzers::get_user_by_name(&login).unwrap();
        client
            .conversation_mut()
            .set_credentials(login.clone(), password);
        client.authenticate().context("Authentication failed")?;
        client.open_session().context("Could not open a session")?;

        // let original_path = std::env::var("PATH").expect("PATH variable to be set");
        // let new_path = std::env::var("PATH").expect("PATH variable to be set");
        // let combined = format!("{new_path}:{original_path}");
        // std::env::set_var("PATH", &combined);

        desktop.run_as(user)?;
        // TODO: maybe we should exit here as the desktop is launched
        Ok(())
    }
}
