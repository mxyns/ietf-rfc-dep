use crate::app::RFCDepApp;
use crate::doc::StatefulDoc;
use eframe::egui;
use eframe::egui::{Align, Ui};
use rfc_dep_ietf::IetfDoc;

impl RFCDepApp {
    pub(crate) fn query_docs(&mut self) {
        self.query_result = IetfDoc::lookup(
            self.search_query.as_str(),
            self.settings.query.limit,
            self.settings.query.rfc_only,
        );
        self.selected_query_docs = vec![false; self.query_result.len()];

        println!("{:#?}", self.query_result);
        println!("{:#?}", self.selected_query_docs);
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
                        let mut results: Vec<IetfDoc<_>> = selected
                            .iter()
                            .enumerate()
                            .filter_map(|(i, b)| if *b { Some(i) } else { None })
                            .map(|i| self.query_result.get(i).unwrap().clone())
                            .collect();

                        results.drain(..).for_each(|doc| {
                            self.cache.cache(doc.name.clone(), StatefulDoc::new(doc));
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
                            for (idx, doc) in self.query_result[range].iter().enumerate() {
                                ui.separator();
                                ui.checkbox(
                                    self.selected_query_docs.get_mut(idx).unwrap_or(&mut false),
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
