use serde::{Deserialize, Serialize};
use merlon::package::Package;

#[derive(Deserialize, Serialize, Default)]
#[serde(default)]
pub struct App {
    package_state: Option<PackageState>,
}

#[derive(Deserialize, Serialize)]
pub struct PackageState {
    package: Package,
}

impl App {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        if let Some(storage) = cc.storage {
            return eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default();
        }

        Default::default()
    }
}

impl eframe::App for App {
    /// Called by the frame work to save state before shutdown.
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }

    /// Called each time the UI needs repainting, which may be many times per second.
    /// Put your widgets into a `SidePanel`, `TopPanel`, `CentralPanel`, `Window` or `Area`.
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Examples of how to create different panels and windows.
        // Pick whichever suits you.
        // Tip: a good default choice is to just keep the `CentralPanel`.
        // For inspiration and more examples, go to https://emilk.github.io/egui

        egui::TopBottomPanel::top("top_bar").show(ctx, |ui| {
            // Draw title bar on macOS
            #[cfg(target_os = "macos")]
            ui.vertical_centered(|ui| {
                ui.set_height(24.0);
                ui.allocate_space(egui::vec2(0.0, 2.0));
                ui.label("Merlon");
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            
            ui.vertical_centered(|ui| {
                ui.spacing_mut().item_spacing.y = 8.0;

                ui.add_space(16.0);
                ui.heading("Welcome to Merlon"); // TODO: use logotype

                ui.allocate_ui_with_layout(egui::vec2(200.0, 20.0), egui::Layout::from_main_dir_and_cross_align(egui::Direction::LeftToRight, egui::Align::Center), |ui| {
                    ui.label(format!("Version {}", env!("CARGO_PKG_VERSION")));
                    egui::warn_if_debug_build(ui);
                });

                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing.x = 20.0;
                    ui.add(egui::Hyperlink::from_label_and_url("Documentation", "https://merlon.readthedocs.io/"));
                    ui.add(egui::Hyperlink::from_label_and_url("Issues", "https://github.com/nanaian/merlon/issues"));
                    ui.add(egui::Hyperlink::from_label_and_url("Source code", "https://github.com/nanaian/merlon"));
                });
                ui.add(egui::Hyperlink::from_label_and_url("Paper Mario Modding Discord server", "https://discord.gg/paper-mario-modding"));
            });
        });

        /*egui::TopBottomPanel::bottom("bottom_bar").show(ctx, |ui| {
            // TODO
        });*/

        // Collect dropped files
        ctx.input(|i| {
            if !i.raw.dropped_files.is_empty() {
                // TODO: handle them, i.e. open .merlon files, open directories, etc.
            }
        });
    }
}
