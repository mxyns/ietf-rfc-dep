use std::collections::HashSet;
use std::ops::Deref;
use std::time::Duration;
use crate::app::RFCDepApp;
use crate::doc::{DocReference, StatefulDoc};
use eframe::egui::{Id, popup, Response, Ui};
use egui_extras::{Column, TableBuilder};
use rfc_dep_cache::CacheReference;
use rfc_dep_ietf::{DocIdentifier, IetfDoc, Meta};

fn name_to_href(ui: &mut Ui, s: &String) -> Response {
    ui.hyperlink_to(s, IetfDoc::<DocReference>::id_to_url(s).unwrap().html())
}

fn list_meta_links(ui: &mut Ui, list: &HashSet<DocReference>) {
    for DocReference(meta) in list {
        match meta {
            CacheReference::Unknown(id) => {
                name_to_href(ui, id);
            }
            CacheReference::Cached(id) => {
                name_to_href(ui, id);
            }
        }
    }
}

impl RFCDepApp {
    pub(crate) fn make_table_view(&mut self, ui: &mut Ui) {
        let action_popup = ui.make_persistent_id("table_item_actions");

        TableBuilder::new(ui)
            .striped(true)
            .vscroll(true)
            .column(Column::initial(20.0).clip(false).resizable(true)) //
            .column(Column::initial(60.0).clip(false).resizable(true)) // Actions
            .column(Column::initial(30.0).clip(false).resizable(true)) // Read
            .column(Column::initial(50.0).clip(true).resizable(true)) // Name
            .column(Column::initial(160.0).clip(true).resizable(true)) // Title
            .column(Column::initial(50.0).clip(true).resizable(true)) // Relations
            .column(Column::initial(30.0).clip(true).resizable(true)) // AKA
            .columns(Column::initial(75.0).clip(true).resizable(true), 6) // Was
            // Replaces
            // Updates
            // Obsoletes
            // Updated By
            // Obsoleted By
            .header(10.0, |mut header| {
                vec![
                    "",
                    "Action",
                    "Read",
                    "Name",
                    "Title",
                    "Relations",
                    "AKA",
                    "Was",
                    "Replaces",
                    "Updates",
                    "Obsoletes",
                    "Updated By",
                    "Obsoleted By",
                ]
                    .drain(..)
                    .for_each(|x| {
                        header.col(|ui| {
                            ui.label(x);
                        });
                    });
            })
            .body(|mut body| {
                for (id, state) in (&mut self.cache).into_iter() {
                    body.row(30.0, |mut row| {
                        row.col(|ui| {
                            if ui.checkbox(&mut state.is_selected, "").clicked() {
                                if state.is_selected {
                                    self.list_selected_count += 1
                                } else {
                                    self.list_selected_count -= 1
                                }
                            }
                        });
                        row.col(|ui| {
                            ui.vertical_centered_justified(|ui| {
                                let button = ui.button("...");
                                if button.clicked() {
                                    ui.memory_mut(|mem| {
                                        mem.data.insert_temp(action_popup, (id.clone(), button.clone()));
                                        mem.toggle_popup(action_popup);
                                    });
                                }
                            });
                        });

                        let doc = &state.content;
                        row.col(|ui| {
                            ui.horizontal_centered(|ui| ui.checkbox(&mut state.is_read, ""));
                        });
                        row.col(|ui| {
                            name_to_href(ui, id);
                        });
                        row.col(|ui| {
                            ui.label(doc.summary.title.clone());
                        });
                        row.col(|ui| {
                            ui.label(doc.meta.count().to_string());
                        });
                        row.col(|ui| {
                            ui.horizontal(|ui| {
                                for (_, meta) in doc.meta.deref() {
                                    if let Meta::AlsoKnownAs(id) = meta {
                                        name_to_href(ui, id);
                                    }
                                }
                            });
                        });
                        row.col(|ui| {
                            ui.horizontal(|ui| {
                                for (_, meta) in doc.meta.deref() {
                                    if let Meta::Was(id) = meta {
                                        name_to_href(ui, id);
                                    }
                                }
                            });
                        });
                        row.col(|ui| {
                            ui.horizontal(|ui| {
                                for (_, meta) in doc.meta.deref() {
                                    if let Meta::Replaces(id) = meta {
                                        name_to_href(ui, id);
                                    }
                                }
                            });
                        });
                        row.col(|ui| {
                            ui.horizontal(|ui| {
                                for (_, meta) in doc.meta.deref() {
                                    if let Meta::Updates(list) = meta {
                                        list_meta_links(ui, list);
                                    }
                                }
                            });
                        });
                        row.col(|ui| {
                            ui.horizontal(|ui| {
                                for (_, meta) in doc.meta.deref() {
                                    if let Meta::Obsoletes(list) = meta {
                                        list_meta_links(ui, list);
                                    }
                                }
                            });
                        });
                        row.col(|ui| {
                            ui.horizontal(|ui| {
                                for (_, meta) in doc.meta.deref() {
                                    if let Meta::UpdatedBy(list) = meta {
                                        list_meta_links(ui, list);
                                    }
                                }
                            });
                        });
                        row.col(|ui| {
                            ui.horizontal(|ui| {
                                for (_, meta) in doc.meta.deref() {
                                    if let Meta::ObsoletedBy(list) = meta {
                                        list_meta_links(ui, list);
                                    }
                                }
                            });
                        });
                    });
                }
            });

        let id: Option<(DocIdentifier, Response)> = ui.memory(|mem| {
           mem.data.get_temp(action_popup)
        });

        // all of this is atrocious
        if let Some((ref id, ref button)) = id {
            if let Some(result) = self.make_actions_ui(action_popup, id, button, ui) {
                if let Some(state) = self.cache.get_mut(id) {
                    *state = result
                }
            }
        }
    }

    fn make_actions_ui(&mut self, popup_id: Id, id: &DocIdentifier, button: &Response, ui: &mut Ui) -> Option<StatefulDoc> {
        let mut state = if let Some(last_state) = self.cache.get_mut(id) {
            last_state.clone()
        } else {
            return None;
        };

        let was_open = popup::popup_below_widget(ui, popup_id, button, |ui| {
            ui.vertical(|ui| {
                ui.horizontal(|ui| {
                    ui.label("resolve");
                    if state.missing_dep_count > 0
                        && ui.small_button(format!("+ {}", state.missing_dep_count)).clicked()
                    {
                        state.to_resolve = true;
                        self.cache_requires_update = true;
                    };
                });

                ui.horizontal(|ui| {
                    ui.label("offline");
                    if state.offline.is_none() {
                        if ui.small_button("Save").clicked() {
                            if let Err(err) = state.download() {
                                self.toasts.error(err).set_duration(Some(Duration::from_secs(5)));
                            }
                        }
                    } else {
                        if ui.small_button("View").clicked() {
                            self.open_viewer(id.clone());
                            println!("{}", state.offline.as_ref().unwrap());
                        }

                        if ui.small_button("Forget").clicked() {
                            state.offline = None;
                        }
                    }
                });
            });

            state
        });

        if was_open.is_some() && button.clicked_elsewhere() {
            ui.memory_mut(|mem| mem.open_popup(popup_id));
        }

        was_open
    }
}
