mod cache;
mod doc;
mod gui;

use crate::gui::{RFCDepApp};

// TODO rework cache to be independent of docs (w/ generic)

fn main() {
    let options = eframe::NativeOptions {
        centered: true,
        ..Default::default()
    };
    eframe::run_native("rfc-dep", options, Box::new(|cc| Box::new(RFCDepApp::new(cc)))).unwrap()
}
