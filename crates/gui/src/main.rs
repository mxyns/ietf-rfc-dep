use crate::app::RFCDepApp;

mod app;
mod cache;
mod doc;
mod menubar;
mod settings;
mod sidebar;
mod table_view;
mod tabs;

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
