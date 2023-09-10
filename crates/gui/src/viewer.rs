use eframe::egui;
use eframe::egui::Ui;
use rfc_dep_ietf::DocIdentifier;
use crate::app::RFCDepApp;
use crate::tabs::Tab;

impl RFCDepApp {

    pub(crate) fn open_viewer(&mut self, id: DocIdentifier) {
        self.viewed_doc = Some(id);
        self.selected_tab = Tab::Viewer
    }

    pub(crate) fn make_viewer_view(&mut self, ui: &mut Ui) {
        let id = self.viewed_doc.as_ref();
        if let None = id {
            ui.label("Open a document to view it here.");
            return;
        }

        let id = id.unwrap();

        let doc = self.cache.get_mut(id);
        if let None = doc {
            ui.label(format!("Document \"{id}\" is not in cache, please include it."));
            return;
        }

        if let None = doc.as_ref().unwrap().offline {
            ui.vertical(|ui| {
                ui.label(format!("Document \"{id}\" is not downloaded, please save it before viewing."));
                if ui.button("download").clicked() {
                    let _ = doc.unwrap().download();
                }
            });
            return;
        }

        egui::ScrollArea::vertical().show(ui, |ui| {
            ui.vertical_centered_justified(|ui| {
                ui.text_edit_multiline(&mut doc.unwrap().offline.as_mut().unwrap().as_str());
            });
        });
    }
}