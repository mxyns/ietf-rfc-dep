use crate::gui::RFCDepApp;
use eframe;

mod gui;
mod doc;
mod tabs;
mod menubar;
mod sidebar;
mod table_view;

fn main() {
    let options = eframe::NativeOptions {
        centered: true,
        ..Default::default()
    };
    eframe::run_native("rfc-dep", options, Box::new(|cc| Box::new(RFCDepApp::new(cc)))).unwrap()
}
