use crate::app::RFCDepApp;

mod app;
mod cache;
mod doc;
mod menubar;
mod settings;
mod sidebar;
mod table_view;
mod tabs;

// TODO do not save cachereference as known / unknown but as ids and update after import/open
// TODO support datatracker -> refererences/referencedby (https://datatracker.ietf.org/doc/draft-raszuk-idr-flow-spec-v6/)
fn main() {
    let options = eframe::NativeOptions {
        centered: true,
        ..Default::default()
    };
    eframe::run_native(
        "rfc-dep",
        options,
        Box::new(|cc| Box::new(RFCDepApp::new(cc))),
    )
    .unwrap()
}
