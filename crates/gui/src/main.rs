use crate::app::RFCDepApp;
use eframe;

mod app;
mod doc;
mod tabs;
mod menubar;
mod sidebar;
mod table_view;
mod settings;
mod cache;

fn main() {
    let options = eframe::NativeOptions {
        centered: true,
        ..Default::default()
    };
    eframe::run_native("rfc-dep", options, Box::new(|cc| Box::new(RFCDepApp::new(cc)))).unwrap()
}
