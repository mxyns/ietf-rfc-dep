use eframe::egui;
use eframe::egui::{Align, Ui};
use rayon::prelude::*;
use std::time::Duration;

use rfc_dep_ietf::IetfDoc;

use crate::app::RFCDepApp;
use crate::doc::{DocReference, StatefulDoc};

impl RFCDepApp {
    pub(crate) fn query_docs(&mut self) {
        let result = IetfDoc::<DocReference>::lookup(
            self.search_query.as_str(),
            self.settings.query.limit,
            self.settings.query.include_drafts,
        );

        if let Ok(result) = result {
            self.query_result = result;
            self.selected_query_docs = vec![false; self.query_result.len()];
        } else {
            self.toasts
                .error(format!("Lookup Error: {}", result.err().unwrap()))
                .set_closable(true)
                .set_duration(Some(Duration::from_secs(10)));
        }

        println!("{:#?}", self.query_result);
    }

    pub(crate) fn make_sidebar(&mut self, ui: &mut Ui) {
        ui.vertical(|ui| {
            ui.with_layout(egui::Layout::left_to_right(Align::TOP), |ui| {
                if (ui
                    .add(
                        egui::TextEdit::singleline(&mut self.search_query)
                            .hint_text("search datatracker.ietf.org"),
                    )
                    .lost_focus()
                    && ui.input(|i| i.key_pressed(egui::Key::Enter)))
                    || ui.button("lookup").clicked()
                {
                    self.query_docs();
                }

                self.make_query_settings_ui(ui);
            });
            ui.end_row();

            ui.with_layout(egui::Layout::bottom_up(Align::LEFT), |ui| {
                ui.with_layout(egui::Layout::right_to_left(Align::BOTTOM), |ui| {
                    ui.add(
                        egui::DragValue::new(&mut self.settings.max_depth)
                            .suffix(" max depth")
                            .clamp_range(1..=u64::MAX),
                    );

                    if ui.button("include").clicked() {
                        let selected = &self.selected_query_docs;
                        let mut results: Vec<_> = selected
                            .iter()
                            .enumerate()
                            .filter_map(|(i, b)| if *b { Some(i) } else { None })
                            .map(|i| self.query_result.get(i).unwrap().clone())
                            .collect();

                        let mut results: Vec<_> = results
                            .par_drain(..)
                            .filter_map(|summary| {
                                if let Ok(doc) = IetfDoc::from_url(summary.url) {
                                    Some(doc)
                                } else {
                                    None
                                }
                            })
                            .collect();

                        results.drain(..).for_each(|doc| {
                            self.cache
                                .cache(doc.summary.name.clone(), StatefulDoc::new(doc));
                        });
                    }
                });

                ui.separator();

                ui.with_layout(egui::Layout::top_down(Align::LEFT), |ui| {
                    egui::ScrollArea::vertical().drag_to_scroll(true).show_rows(
                        ui,
                        10.0,
                        self.query_result.len(),
                        |ui, range| {
                            let range_start = range.start;
                            for (idx, doc) in self.query_result[range].iter().enumerate() {
                                ui.separator();
                                ui.checkbox(
                                    self.selected_query_docs
                                        .get_mut(range_start + idx)
                                        .unwrap_or(&mut false),
                                    &doc.title,
                                );
                                ui.label(&doc.name);
                                ui.hyperlink_to("datatracker", &doc.url);
                            }
                            ui.separator();
                        },
                    );
                });
            });

            ui.end_row();
        });
    }
}
