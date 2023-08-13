use crate::app::RFCDepApp;
use eframe::egui;
use eframe::egui::{Align, Ui};

pub(crate) struct Tabs;

impl Tabs {
    pub(crate) fn all() -> Vec<Tab> {
        vec![Tab::Table, Tab::Graph]
    }
}

impl RFCDepApp {
    pub(crate) fn make_tab_list(&mut self, ui: &mut Ui) {
        ui.with_layout(egui::Layout::left_to_right(Align::TOP), |ui| {
            Tabs::all().iter().for_each(|tab| {
                tab.make_tab_ui(self, ui);
            });
        });
    }

    pub(crate) fn make_tab_view(&mut self, ui: &mut Ui) {
        match self.selected_tab {
            Tab::Table => {
                self.make_table_view(ui);
            }
            Tab::Graph => {
                ui.label("todo!()");
            }
        }
    }
}

#[derive(Clone, Default, Debug)]
pub enum Tab {
    #[default]
    Table,
    Graph,
}

impl Tab {
    pub(crate) fn make_tab_ui(&self, app: &mut RFCDepApp, ui: &mut Ui) {
        match self {
            Tab::Table => {
                if ui.button("list").clicked() {
                    app.selected_tab = Tab::Table;
                }
            }
            Tab::Graph => {
                if ui.button("graph").clicked() {
                    app.selected_tab = Tab::Graph;
                }
            }
        }
    }
}
