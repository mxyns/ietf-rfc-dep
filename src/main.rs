mod cache;
mod doc;
mod gui;

use crate::gui::{RFCDepApp};

// TODO persistent read and cache
    // TODO move dep count to DocState (needs algo with root search)
    // TODO save DocState with DocCache (DocState has a CachedDoc which needs DocCache for Deser. so I need to rework cache before)
// TODO reduce load on CPU when nothing is needed (eg: do not recompute DocState each frame)
// TODO async concurrent/parallel doc queries
// TODO graph gui

fn main() {
    let options = eframe::NativeOptions {
        centered: true,
        ..Default::default()
    };
    eframe::run_native("rfc-dep", options, Box::new(|cc| Box::new(RFCDepApp::new(cc)))).unwrap()
}
