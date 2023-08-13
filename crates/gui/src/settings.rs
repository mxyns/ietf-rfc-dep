use crate::app::RFCDepApp;
use derivative::Derivative;
use eframe::egui::{popup, DragValue, Ui};

#[derive(Debug, Derivative)]
#[derivative(Default)]
pub(crate) struct QuerySettings {
    #[derivative(Default(value = "100"))]
    pub(crate) limit: usize,

    #[derivative(Default(value = "true"))]
    pub(crate) include_drafts: bool,
}

#[derive(Default, Debug)]
pub(crate) struct Settings {
    pub(crate) query: QuerySettings,

    pub(crate) max_depth: usize,
}

impl RFCDepApp {
    pub(crate) fn make_query_settings_ui(&mut self, ui: &mut Ui) {
        let settings_id = ui.make_persistent_id("settings");
        let button = ui.button("⛭");
        if button.clicked() {
            ui.memory_mut(|mem| mem.toggle_popup(settings_id));
        }
        let was_open = popup::popup_below_widget(ui, settings_id, &button, |ui| {
            ui.vertical(|ui| {
                ui.horizontal(|ui| {
                    ui.label("limit");
                    ui.add(DragValue::new(&mut self.settings.query.limit).suffix(" docs"))
                });

                ui.horizontal(|ui| {
                    ui.label("include drafts");
                    ui.checkbox(&mut self.settings.query.include_drafts, "");
                });
            });
        });

        if was_open.is_some() && button.clicked_elsewhere() {
            ui.memory_mut(|mem| mem.open_popup(settings_id));
        }
    }
}
