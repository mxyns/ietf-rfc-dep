use std::collections::HashSet;
use crate::app::RFCDepApp;
use crate::doc::DocReference;
use eframe::egui::{Response, Ui};
use egui_extras::{Column, TableBuilder};
use rfc_dep_cache::CacheReference;
use rfc_dep_ietf::{IetfDoc, Meta};

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
        TableBuilder::new(ui)
            .striped(true)
            .vscroll(true)
            .column(Column::initial(20.0).clip(true).resizable(true))
            .column(Column::initial(30.0).clip(true).resizable(true))
            .column(Column::initial(30.0).clip(true).resizable(true))
            .column(Column::initial(50.0).clip(true).resizable(true))
            .column(Column::initial(160.0).clip(true).resizable(true))
            .column(Column::initial(50.0).clip(true).resizable(true))
            .column(Column::initial(30.0).clip(true).resizable(true))
            .column(Column::initial(75.0).clip(true).resizable(true))
            .column(Column::initial(75.0).clip(true).resizable(true))
            .column(Column::initial(75.0).clip(true).resizable(true))
            .column(Column::initial(75.0).clip(true).resizable(true))
            .column(Column::initial(75.0).clip(true).resizable(true))
            .column(Column::remainder())
            .header(10.0, |mut header| {
                vec![
                    "",
                    "Missing",
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
                    body.row(20.0, |mut row| {
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
                            ui.horizontal_centered(|ui| {
                                let missing = &state.missing_dep_count;
                                if missing > &0
                                    && ui.small_button(format!("+ {}", missing)).clicked()
                                {
                                    state.to_resolve = true;
                                    self.cache_requires_update = true;
                                };
                            });
                        });

                        let doc = &state.content;
                        row.col(|ui| {
                            ui.checkbox(&mut state.is_read, "");
                        });
                        row.col(|ui| {
                            name_to_href(ui, id);
                        });
                        row.col(|ui| {
                            ui.label(doc.summary.title.clone());
                        });
                        row.col(|ui| {
                            ui.label(doc.meta_count().to_string());
                        });
                        row.col(|ui| {
                            ui.horizontal(|ui| {
                                for meta in &doc.meta {
                                    if let Meta::AlsoKnownAs(id) = meta {
                                        name_to_href(ui, id);
                                    }
                                }
                            });
                        });
                        row.col(|ui| {
                            ui.horizontal(|ui| {
                                for meta in &doc.meta {
                                    if let Meta::Was(id) = meta {
                                        name_to_href(ui, id);
                                    }
                                }
                            });
                        });
                        row.col(|ui| {
                            ui.horizontal(|ui| {
                                for meta in &doc.meta {
                                    if let Meta::Replaces(id) = meta {
                                        name_to_href(ui, id);
                                    }
                                }
                            });
                        });
                        row.col(|ui| {
                            ui.horizontal(|ui| {
                                for meta in &doc.meta {
                                    if let Meta::Updates(list) = meta {
                                        list_meta_links(ui, list);
                                    }
                                }
                            });
                        });
                        row.col(|ui| {
                            ui.horizontal(|ui| {
                                for meta in &doc.meta {
                                    if let Meta::Obsoletes(list) = meta {
                                        list_meta_links(ui, list);
                                    }
                                }
                            });
                        });
                        row.col(|ui| {
                            ui.horizontal(|ui| {
                                for meta in &doc.meta {
                                    if let Meta::UpdatedBy(list) = meta {
                                        list_meta_links(ui, list);
                                    }
                                }
                            });
                        });
                        row.col(|ui| {
                            ui.horizontal(|ui| {
                                for meta in &doc.meta {
                                    if let Meta::ObsoletedBy(list) = meta {
                                        list_meta_links(ui, list);
                                    }
                                }
                            });
                        });
                    });
                }
            });
    }
}
