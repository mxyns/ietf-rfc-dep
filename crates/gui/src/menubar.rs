use eframe::egui;
use eframe::egui::{Context, Ui};
use egui_modal::Modal;
use if_chain::if_chain;
use std::fs::File;
use std::time::Duration;

use rfc_dep_cache::{ResolveParams, ResolveTarget};
use rfc_dep_ietf::IetfDoc;

use crate::app::RFCDepApp;
use crate::cache::DocCache;
use crate::doc::StatefulDoc;

impl RFCDepApp {
    pub(crate) fn make_menu(&mut self, ui: &mut Ui, confirm_clear: Modal, import_name: Modal) {
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
                        let new_state: DocCache = serde_json::from_reader(file).unwrap();
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

            ui.menu_button("Document", |ui| {
                if ui.button("From name").clicked() {
                    import_name.open()
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
                            self.cache.retain(|_, state| !state.is_selected);
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
                        self.task_resolve_dependencies(
                            ResolveTarget::All,
                            ResolveParams {
                                print: true,
                                depth: self.settings.max_depth,
                                query: true,
                            },
                        );
                    }
                });
            });
        });
    }

    pub(crate) fn make_clear_confirm_dialog(&mut self, ctx: &Context) -> Modal {
        let modal = Modal::new(ctx, "confirm_clear").with_close_on_outside_click(false);
        modal.show(|ui| {
            modal.title(ui, "Confirm Clear");
            modal.body_and_icon(
                ui,
                "Clearing the current state will result in loss of any unsaved change",
                egui_modal::Icon::Warning,
            );
            modal.buttons(ui, |ui| {
                if modal.caution_button(ui, "cancel").clicked() {
                    modal.close()
                }
                if modal.suggested_button(ui, "clear").clicked() {
                    self.reset();
                    println!("{:#?}", self.cache);
                };
            });
        });

        modal
    }

    pub(crate) fn make_import_name_modal(&mut self, ctx: &Context) -> Modal {
        let modal = Modal::new(ctx, "import_name").with_close_on_outside_click(false);

        modal.show(|ui| {
            modal.title(ui, "Import from name");
            modal.body_and_icon(
                ui,
                "Provide the exact name of the document",
                egui_modal::Icon::Info,
            );

            ui.separator();

            ui.add(
                egui::widgets::TextEdit::singleline(&mut self.direct_import_name)
                    .hint_text("rfcXXXX / draft-abcdef"),
            );

            modal.buttons(ui, |ui| {
                if modal.caution_button(ui, "cancel").clicked() {
                    modal.close()
                }
                if modal.suggested_button(ui, "import").clicked() {
                    if self.direct_import_name.is_empty() {
                        return;
                    }
                    let doc = IetfDoc::from_name(&self.direct_import_name);
                    if let Ok(doc) = doc {
                        self.cache
                            .cache(doc.summary.id.clone(), StatefulDoc::new(doc));
                    } else {
                        self.toasts
                            .error(format!(
                                "Could not import {}: {}",
                                &self.direct_import_name,
                                doc.err().unwrap()
                            ))
                            .set_closable(true)
                            .set_duration(Some(Duration::from_secs(10)));
                    }
                };
            });
        });

        modal
    }
}
