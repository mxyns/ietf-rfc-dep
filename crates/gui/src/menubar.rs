use eframe::egui;
use eframe::egui::{Context, Ui};
use if_chain::if_chain;
use rfc_dep_cache::{Cache, ResolveParams, ResolveTarget};
use crate::doc::{update_missing_dep_count, StatefulDoc};
use rfc_dep_ietf::{DocIdentifier};
use crate::gui::RFCDepApp;
use std::fs::File;
use egui_modal::Modal;

impl RFCDepApp {
    pub(crate) fn make_menu(&mut self, ui: &mut Ui, confirm_clear: Modal) {
        egui::menu::bar(ui, |ui| {
            ui.menu_button("File", |ui| {
                // Open Button
                if_chain! {
                        if ui.button("Open").clicked();
                        if let Some(path) = rfd::FileDialog::new()
                            .add_filter("json", &["json"])
                            .pick_file();
                        if let Ok(file) = File::open(path);
                        then {
                            self.update_cache(Some(
                                serde_json::from_reader(file).unwrap()
                            ));
                        }
                    }

                // Save Button
                if_chain! {
                        if ui.button("Save").clicked();
                        if let Some(path) = rfd::FileDialog::new()
                            .add_filter("json", &["json"])
                            .save_file();
                        if let Ok(file) = &File::create(path);
                        then {
                            serde_json::to_writer_pretty(file, &self.cache).unwrap();
                        }
                    }

                // Import Button
                if_chain! {
                        if ui.button("Import").clicked();
                        if let Some(path) = rfd::FileDialog::new()
                            .add_filter("json", &["json"])
                            .pick_file();
                        if let Ok(file) = File::open(path);
                        then {
                            let new_state: Cache<DocIdentifier, StatefulDoc> = serde_json::from_reader(file).unwrap();
                            println!("{:#?}", new_state);
                            self.merge_caches(new_state);
                            println!("{:#?}", self.cache);
                        }
                    }

                ui.separator();

                // Clear Button
                if ui.button("Clear").clicked() {
                    confirm_clear.open();
                }
            });

            let cache_size = self.cache.len();
            ui.add_enabled_ui(cache_size > 0, |ui| {
                ui.menu_button("Select", |ui| {
                    ui.add_enabled_ui(self.list_selected_count < cache_size, |ui| {
                        if ui.button("Select All").clicked() {
                            (&mut self.cache).into_iter().for_each(|(_, state)| {
                                state.is_selected = true;
                            });
                            self.list_selected_count = self.cache.len();
                        }
                    });

                    ui.add_enabled_ui(self.list_selected_count >= cache_size, |ui| {
                        if ui.button("Deselect All").clicked() {
                            (&mut self.cache).into_iter().for_each(|(_, state)| {
                                state.is_selected = false;
                            });
                            self.list_selected_count = 0;
                        }
                    });
                    ui.add_enabled_ui(self.list_selected_count > 0, |ui| {
                        if ui.button("Remove selected").clicked() {
                            self.cache.retain(|_, state| state.is_selected == false);
                            self.update_cache(None);
                            self.list_selected_count = 0;
                        }
                    });
                });
            });

            // update value since cache may have been updated
            let cache_size = self.cache.len();
            ui.add_enabled_ui(cache_size > 0, |ui| {
                ui.menu_button("Resolve", |ui| {
                    ui.add_enabled_ui(self.list_selected_count > 0, |ui| {
                        if ui.button("Resolve Selected").clicked() {
                            for (_id, doc) in (&mut self.cache).into_iter() {
                                if doc.is_selected {
                                    doc.to_resolve = true;
                                    self.cache_requires_update = true;
                                }
                            }
                        }
                    });

                    if ui.button("Resolve All").clicked() {
                        self.cache.resolve_dependencies(ResolveTarget::All,
                                                        ResolveParams {
                                                            print: true,
                                                            depth: self.max_depth.clone(),
                                                            query: true,
                                                        }, update_missing_dep_count);
                    }
                });
            });
        });
    }

    pub(crate) fn make_clear_confirm_dialog(&mut self, ctx: &Context) -> Modal {
        let modal = Modal::new(ctx, "confirm_clear")
            .with_close_on_outside_click(false);
        modal.show(|ui| {
            modal.title(ui, "Confirm Clear");
            modal.body_and_icon(ui, "Clearing the current state will result in loss of any unsaved change", egui_modal::Icon::Warning);
            modal.buttons(ui, |ui| {
                if modal.caution_button(ui, "cancel").clicked() { modal.close() }
                if modal.suggested_button(ui, "clear").clicked() {
                    self.reset();
                    println!("{:#?}", self.cache);
                };
            });
        });

        modal
    }
}