mod cache;
mod doc;
mod gui;

use crate::gui::{RFCDepApp};

// TODO resolve: pass array of roots instead of single root for multiple selection resolve
// TODO reduce .clone on IdType
// TODO graph gui

fn main() {
    let options = eframe::NativeOptions {
        centered: true,
        ..Default::default()
    };
    eframe::run_native("rfc-dep", options, Box::new(|cc| Box::new(RFCDepApp::new(cc)))).unwrap()
}
