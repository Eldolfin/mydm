use egui::{Align2, Color32, RichText, TextEdit, Ui, Widget};
use log::debug;

use crate::desktops::DesktopEntry;

pub struct MyDm<F>
where
    F: Fn(LoginRequest) -> anyhow::Result<()>,
{
    data: MyDmData<F>,
    state: MyDmState,
}

// config data that is never changed
pub struct MyDmData<F>
where
    F: Fn(LoginRequest) -> anyhow::Result<()>,
{
    pub users: Vec<String>,
    pub on_login: F,
    pub desktops: Vec<crate::desktops::DesktopEntry>,
}

#[derive(Clone, Debug)]
pub struct LoginRequest {
    pub login: String,
    pub password: String,
    pub desktop: DesktopEntry,
}

// mutable state
#[derive(Default)]
pub struct MyDmState {
    pub password: String,
    pub user_index: usize,
    pub desktop_index: usize,
    pub show_password: bool,
    pub last_attemp_result: LoginResult,
}

impl<F> MyDm<F>
where
    F: Fn(LoginRequest) -> anyhow::Result<()>,
{
    pub fn new(data: MyDmData<F>) -> Self {
        assert!(
            !data.users.is_empty(),
            "Can't show display manager with 0 users"
        );
        assert!(
            !data.desktops.is_empty(),
            "Can't show display manager with 0 desktops"
        );
        Self {
            data,
            state: MyDmState::default(),
        }
    }

    pub fn run(self) -> Result<(), eframe::Error> {
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
    F: Fn(LoginRequest) -> anyhow::Result<()>,
{
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::Window::new("Login")
            .collapsible(false)
            .resizable(false)
            .anchor(Align2::CENTER_CENTER, [0., 0.])
            .show(ctx, |ui| {
                ui.vertical(|ui| {
                    ui.push_id("desktop select", |ui| {
                        egui::ComboBox::from_label("")
                            .selected_text(
                                self.data.desktops[self.state.desktop_index].name.to_owned(),
                            )
                            .show_ui(ui, |ui| {
                                for (i, login) in self.data.desktops.iter().enumerate() {
                                    ui.selectable_value(
                                        &mut self.state.desktop_index,
                                        i,
                                        login.name.to_owned(),
                                    );
                                }
                            });
                    });
                    ui.push_id("user select", |ui| {
                        egui::ComboBox::from_label("")
                            .selected_text(self.data.users[self.state.user_index].to_owned())
                            .show_ui(ui, |ui| {
                                for (i, login) in self.data.users.iter().enumerate() {
                                    ui.selectable_value(
                                        &mut self.state.user_index,
                                        i,
                                        login.to_owned(),
                                    );
                                }
                            });
                    });
                    ui.horizontal(|ui| {
                        let password_field = TextEdit::singleline(&mut self.state.password)
                            .password(!self.state.show_password)
                            .background_color(self.state.last_attemp_result.color(ui))
                            .hint_text("Password...")
                            .ui(ui);
                        self.state.show_password = ui
                            .label(RichText::new("👁").color(Color32::GREEN).size(26.))
                            .hovered();
                        if password_field
                            .ctx
                            .input(|i| i.key_pressed(egui::Key::Enter))
                        {
                            let login = self.data.users[self.state.user_index].to_owned();
                            let desktop = self.data.desktops[self.state.desktop_index].to_owned();
                            let password = self.state.password.to_owned();
                            let res = (self.data.on_login)(LoginRequest {
                                login,
                                password,
                                desktop,
                            })
                            .inspect_err(|err| debug!("Error while logging in: {:#}", err));

                            self.state.last_attemp_result = if res.is_ok() {
                                LoginResult::Success
                            } else {
                                LoginResult::WrongPassword
                            };
                            debug!("login_result = {:?}", self.state.last_attemp_result);
                        }
                    });
                });
            });
    }
}

#[derive(Default, Clone, Copy, Debug)]
pub enum LoginResult {
    #[default]
    NoAttempt,
    Success,
    WrongPassword,
}

impl LoginResult {
    pub fn color(&self, ui: &Ui) -> Color32 {
        match self {
            LoginResult::NoAttempt => ui.style().visuals.widgets.active.bg_fill,
            LoginResult::Success => Color32::DARK_GREEN,
            LoginResult::WrongPassword => Color32::DARK_RED,
        }
    }
}
