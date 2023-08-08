mod cache;
mod doc;
mod gui;

use crate::gui::{RFCDepApp};

// TODO confirm dialog for clear
// TODO async concurrent/parallel doc queries
// TODO graph gui

fn main() {
    let options = eframe::NativeOptions {
        centered: true,
        ..Default::default()
    };
    eframe::run_native("rfc-dep", options, Box::new(|cc| Box::new(RFCDepApp::new(cc)))).unwrap()
}
