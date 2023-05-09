use std::{path::PathBuf, collections::VecDeque};

use serde::{Deserialize, Serialize};
use anyhow::{Result, bail, anyhow};
use merlon::package::Package;
use merlon::package::distribute::{Distributable, OpenOptions};
use merlon::rom::Rom;

mod logotype;
mod support_link;

#[derive(Deserialize, Serialize, Default, Debug)]
#[serde(default)]
pub struct App {
    packages: VecDeque<Package>,
    logotype: logotype::Logotype,
    about_window: bool,
    baserom: Option<Rom>,
}

impl App {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let mut fonts = egui::FontDefinitions::default();
        egui_phosphor::add_to_fonts(&mut fonts);
        cc.egui_ctx.set_fonts(fonts);

        if let Some(storage) = cc.storage {
            log::info!("Loading app state from storage");
            // FIXME: ron parse error, maybe use toml?
            /* 
            let storage_string = &storage.get_string(eframe::APP_KEY).unwrap();
            let app: Result<Self, _> = ron::from_str(&storage_string);
            log::debug!("App storage string: {}", storage_string);
            log::debug!("Parsed app storage: {:?}", app);
            */
            match eframe::get_value(storage, eframe::APP_KEY) {
                Some(app) => return app,
                None => log::warn!("No app state found in storage, or error parsing"),
            }
        }

        Default::default()
    }

    fn open_package_or_distributable(&mut self, path: PathBuf) -> Result<()> {
        let package = if let Ok(package) = Package::try_from(path.clone()) {
            package
        } else if let Ok(distributable) = Distributable::try_from(path.clone()) {
            if let Some(baserom) = self.baserom.as_ref() {
                distributable.open_to_dir(OpenOptions {
                    baserom: baserom.path().to_owned(),
                    ..Default::default()
                })?
            } else {
                bail!("No baserom selected");
            }
        } else {
            bail!("Not a package directory or distributable file: {}", path.display());
        };

        if let Some(existing) = self.packages.iter().find(|p| p.path() == package.path()) {
            bail!("Package already open: {}", existing.path().display());
        }
        self.packages.push_front(package);
        Ok(())
    }

    fn handle_result<T>(&mut self, result: Result<T>) {
        if let Err(err) = result {
            log::error!("{}", err);
            // TODO: show modal error dialog e.g. with egui_modal
        }
    }
}

impl eframe::App for App {
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        if let Some(rom) = self.baserom.as_ref() {
            if !rom.path().is_file() {
                log::warn!("baserom file no longer exists, clearing baserom");
                self.baserom = None;
            }
        }

        log::info!("Saving app state to storage");
        eframe::set_value(storage, eframe::APP_KEY, self);
    }

    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        ctx.enable_accesskit();
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
            ui.set_width(200.0);
            ui.with_layout(egui::Layout::top_down_justified(egui::Align::Center), |ui| {
                ui.spacing_mut().item_spacing.y = 4.0;
                ui.add_space(8.0);

                if ui.button("New package…")
                    .on_hover_text("Create a new Merlon package")
                    .clicked()
                {
                    // TODO
                }

                if ui.button("Import package folder…")
                    .on_hover_text("Import a Merlon package folder")
                    .clicked()
                {
                    if let Some(path) = rfd::FileDialog::new()
                        .add_filter("Merlon distributables", &["merlon"])
                        .pick_folder()
                    {
                        let result = self.open_package_or_distributable(path);
                        self.handle_result(result);
                    }
                }

                ui.scope(|ui| {
                    ui.set_enabled(self.baserom.is_some());
                    if ui.button("Import distributable…")
                        .on_hover_text("Import a Merlon distributable file (.merlon)")
                        .on_disabled_hover_text("Select a base ROM first")
                        .clicked()
                    {
                        if let Some(path) = rfd::FileDialog::new()
                            .add_filter("Merlon distributables", &["merlon"])
                            .pick_file()
                        {
                            let result = self.open_package_or_distributable(path);
                            self.handle_result(result);
                        }
                    }
                });

                if ui.button("Set base ROM…").clicked() {
                    if let Some(path) = rfd::FileDialog::new()
                        .add_filter("N64 ROMs", &["z64"])
                        .pick_file()
                    {
                        let rom = Rom::from(path);
                        let sha1 = rom.sha1_string();
                        log::info!("Base ROM SHA1: {:?}", sha1);
                        if sha1.unwrap_or_else(|_| "".to_string()) != merlon::rom::PAPERMARIO_US_SHA1 {
                            self.handle_result::<()>(Err(anyhow!("Not an unmodified Paper Mario (US) ROM")));
                        } else {
                            self.baserom = Some(rom);

                        }
                    }
                }

                ui.add_space(16.0);
                ui.separator();
                ui.add_space(16.0);

                ui.add(egui::Hyperlink::from_label_and_url("Documentation", "https://merlon.readthedocs.io/"));
                ui.add(support_link::SupportLink);
                if ui.button("About…").clicked() {
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

                ui.label("© 2023 Alex Bates (nanaian)");
                ui.label("The author is not affiliated with Nintendo Co., Ltd. in any way.");
                ui.label("This Executable Form is subject to the terms of the Mozilla Public License, v. 2.0. If a copy of the MPL was not distributed with this program, You can obtain one at https://mozilla.org/MPL/2.0/.");

                ui.add_space(16.0);

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

                update_packages_list(ui, &self.packages);
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

pub fn update_packages_list(ui: &mut egui::Ui, packages: &VecDeque<Package>) {
    egui::ScrollArea::vertical().show(ui, |ui| {
        if packages.is_empty() {
            ui.centered_and_justified(|ui| {
                ui.label("No packages found.");
            });
        } else {
            for package in packages {
                let result: Result<()> = ui.group(|ui| {
                    let manifest = package.manifest()?; // TODO: cache this
                    let metadata = manifest.metadata();

                    ui.vertical(|ui| {
                        let name_str: &str = metadata.name().into();
                        ui.heading(name_str);
                        ui.label(metadata.authors().join(", "));
                        ui.label(format!("Version {}", metadata.version()));
                        ui.label(metadata.description());
                        ui.small(package.path().to_string_lossy());
                    });

                    Ok(())
                }).inner;
                if let Err(e) = result {
                    ui.label(format!("Error: {}", e));
                }
            }
        }
    });
}
