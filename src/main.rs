use std::sync::{Arc, Mutex};

use anyhow::{anyhow, Context};
use egui::{Align2, Color32, RichText, TextEdit, Ui, Widget};
use log::debug;

fn main() -> anyhow::Result<()> {
    env_logger::init();

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

    MyDm::new(users, |login, password| {
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

struct MyDm<F>
where
    F: Fn(String, String) -> anyhow::Result<()>,
{
    users: Vec<String>,
    user_index: usize,
    password: String,
    show_password: bool,
    on_login: F,
    last_attemp_result: LoginResult,
}

impl<F> MyDm<F>
where
    F: Fn(String, String) -> anyhow::Result<()>,
{
    fn new(users: Vec<String>, on_login: F) -> Self {
        assert!(!users.is_empty(), "Can't show display manager with 0 users");
        Self {
            users,
            on_login,
            user_index: 0,
            password: String::new(),
            show_password: false,
            last_attemp_result: LoginResult::NoAttempt,
        }
    }

    fn run(self) -> Result<(), eframe::Error> {
        let options = eframe::NativeOptions {
            viewport: egui::ViewportBuilder::default().with_fullscreen(true),
            ..Default::default()
        };
        eframe::run_native(
            "MyDM",
            options,
            Box::new(|cc| {
                egui_extras::install_image_loaders(&cc.egui_ctx);

                Ok(Box::new(self))
            }),
        )
    }
}

impl<F> eframe::App for MyDm<F>
where
    F: Fn(String, String) -> anyhow::Result<()>,
{
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::Window::new("Login")
            .collapsible(false)
            .resizable(false)
            .anchor(Align2::CENTER_CENTER, [0., 0.])
            .show(ctx, |ui| {
                ui.vertical(|ui| {
                    ui.horizontal(|ui| {
                        egui::ComboBox::from_label("")
                            .selected_text(self.users[self.user_index].to_owned())
                            .show_ui(ui, |ui| {
                                for (i, login) in self.users.iter().enumerate() {
                                    ui.selectable_value(&mut self.user_index, i, login.to_owned());
                                }
                            })
                    });
                    ui.horizontal(|ui| {
                        ui.label("Password: ");
                        let password_field = TextEdit::singleline(&mut self.password)
                            .password(!self.show_password)
                            .background_color(self.last_attemp_result.color(ui))
                            .ui(ui);
                        self.show_password = ui
                            .label(RichText::new("👁").color(Color32::GREEN).size(26.))
                            .hovered();
                        if password_field
                            .ctx
                            .input(|i| i.key_pressed(egui::Key::Enter))
                        {
                            let res = (self.on_login)(
                                self.users[self.user_index].to_owned(),
                                self.password.to_owned(),
                            )
                            .inspect_err(|err| debug!("Error while logging in: {:#}", err));

                            self.last_attemp_result = if res.is_ok() {
                                LoginResult::Success
                            } else {
                                LoginResult::WrongPassword
                            };
                            debug!("login_result = {:?}", self.last_attemp_result);
                        }
                    });
                });
            });
    }
}

#[derive(Clone, Copy, Debug)]
enum LoginResult {
    NoAttempt,
    Success,
    WrongPassword,
}
impl LoginResult {
    fn color(&self, ui: &Ui) -> Color32 {
        match self {
            LoginResult::NoAttempt => ui.style().visuals.widgets.active.bg_fill,
            LoginResult::Success => Color32::DARK_GREEN,
            LoginResult::WrongPassword => Color32::DARK_RED,
        }
    }
}
