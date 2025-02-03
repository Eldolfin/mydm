use egui::{Align2, Color32, RichText, TextEdit, Widget};

fn main() -> eframe::Result {
    env_logger::init();
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default(),
        // .with_fullscreen(true),
        ..Default::default()
    };

    let users = vec!["oscar".to_owned(), "test".to_owned()];

    eframe::run_native(
        "MyDM",
        options,
        Box::new(|cc| {
            egui_extras::install_image_loaders(&cc.egui_ctx);

            Ok(Box::new(MyDm::new(users)))
        }),
    )
}

struct MyDm {
    users: Vec<String>,
    user_index: usize,
    password: String,
    show_password: bool,
}

impl MyDm {
    fn new(users: Vec<String>) -> Self {
        assert!(!users.is_empty(), "Can't show display manager with 0 users");
        Self {
            users,
            user_index: 0,
            password: String::new(),
            show_password: false,
        }
    }
}

impl eframe::App for MyDm {
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
                        TextEdit::singleline(&mut self.password)
                            .password(!self.show_password)
                            .ui(ui);
                        self.show_password = ui
                            .label(RichText::new("👁").color(Color32::GREEN).size(26.))
                            .hovered();
                    });
                });
            });
    }
}
