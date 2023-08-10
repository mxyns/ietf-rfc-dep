mod cache;
mod doc;
mod gui;

use crate::gui::{RFCDepApp};

// TODO do not freeze ui while resolving
// TODO add lookup parameters
// TODO add support for drafts
// TODO stop using from_html for rfcs
// TODO add doc + screenshot
// TODO reduce .clone use on IdType
// TODO graph gui

fn main() {
    let options = eframe::NativeOptions {
        centered: true,
        ..Default::default()
    };
    eframe::run_native("rfc-dep", options, Box::new(|cc| Box::new(RFCDepApp::new(cc)))).unwrap()
}
