use serde::{Deserialize, Serialize};
use merlon::package::Package;

mod logotype;
mod support_link;

#[derive(Deserialize, Serialize, Default)]
#[serde(default)]
pub struct App {
    packages: Vec<Package>,
    logotype: logotype::Logotype,
    about_window: bool,
}

impl App {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        if let Some(storage) = cc.storage {
            return eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default();
        }

        let mut fonts = egui::FontDefinitions::default();
        egui_phosphor::add_to_fonts(&mut fonts);
        cc.egui_ctx.set_fonts(fonts);

        Default::default()
    }
}

impl eframe::App for App {
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }

    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.logotype.load_if_first_attempt(ctx);

        egui::TopBottomPanel::top("top_bar").show(ctx, |ui| {
            // Draw title bar on macOS
            #[cfg(target_os = "macos")]
            ui.vertical_centered(|ui| {
                ui.set_height(24.0);
                ui.allocate_space(egui::vec2(0.0, 2.0));
                ui.label("Merlon");
            });
        });

        egui::SidePanel::right("side_panel").resizable(false).show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                if ui.button("About").clicked() {
                    self.about_window = true;
                }
            });
        });

        egui::Window::new("About").resizable(false).open(&mut self.about_window).show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.spacing_mut().item_spacing.y = 8.0;

                ui.add_space(16.0);
                ui.add(&mut self.logotype);

                ui.label(format!("Version {}", env!("CARGO_PKG_VERSION")));
                egui::warn_if_debug_build(ui);

                ui.add_space(16.0);

                ui.label("© 2023 Alex Bates");

                ui.add_space(16.0);

                ui.add(support_link::SupportLink);
                ui.add(egui::Hyperlink::from_label_and_url("Documentation", "https://merlon.readthedocs.io/"));
                ui.add(egui::Hyperlink::from_label_and_url("Report an issue", "https://github.com/nanaian/merlon/issues"));
                ui.add(egui::Hyperlink::from_label_and_url("Source code", "https://github.com/nanaian/merlon"));
                ui.add(egui::Hyperlink::from_label_and_url("Paper Mario Modding Discord server", "https://discord.gg/paper-mario-modding"));

                ui.add_space(16.0);
            });
        });

        /*egui::TopBottomPanel::bottom("bottom_bar").show(ctx, |ui| {
        });*/

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.spacing_mut().item_spacing.y = 8.0;

                // TODO packages
            });
        });

        // Collect dropped files
        ctx.input(|i| {
            if !i.raw.dropped_files.is_empty() {
                // TODO: handle them, i.e. open .merlon files, open directories, etc.
            }
        });
    }
}
