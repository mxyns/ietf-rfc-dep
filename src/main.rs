mod cache;
mod doc;
mod gui;

use crate::gui::{RFCDepApp};

// TODO rework cache to be independent of docs (w/ generic)
// TODO rework cache to not use references anymore, just ids since we do not need references after all
// TODO graph gui
// TODO persistent read and cache
    // TODO move dep count to DocState
    // TODO save DocState with DocCache
// TODO reduce load on CPU when nothing is needed (eg: do not recompute DocState each frame)
// TODO async concurrent/parallel doc queries
// TODO algo to resolve dependencies only for a document as root

fn main() {
    let options = eframe::NativeOptions {
        centered: true,
        ..Default::default()
    };
    eframe::run_native("rfc-dep", options, Box::new(|cc| Box::new(RFCDepApp::new(cc)))).unwrap()
}
