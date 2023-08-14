use derivative::Derivative;
use eframe::egui;
use egui_notify::Toasts;
use std::thread::JoinHandle;

use rfc_dep_cache::{ResolveParams, ResolveTarget};
use rfc_dep_ietf::{DocIdentifier, Summary};

use crate::cache::DocCache;
use crate::settings::Settings;
use crate::tabs::Tab;

#[derive(Default, Derivative)]
#[derivative(Debug)]
pub struct RFCDepApp {
    // Lookup Related
    pub(crate) search_query: String,
    #[derivative(Debug = "ignore")]
    pub(crate) toasts: Toasts,
    pub(crate) query_result: Vec<Summary>,
    pub(crate) selected_query_docs: Vec<bool>,
    pub(crate) direct_import_name: String,
    pub(crate) query_filter: String,

    // Settings
    pub(crate) settings: Settings,

    // Doc State
    pub(crate) cache: DocCache,
    pub(crate) cache_requires_update: bool,
    pub(crate) list_selected_count: usize,
    pub(crate) resolve_handle: Option<JoinHandle<DocCache>>,

    // RFC Viewer
    pub(crate) selected_tab: Tab,
}

impl RFCDepApp {
    pub(crate) fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        RFCDepApp { ..Self::default() }
    }

    pub(crate) fn reset(&mut self) {
        self.cache.clear();
        self.list_selected_count = 0;
        self.cache_requires_update = false;
    }
}

impl eframe::App for RFCDepApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let confirm_clear = self.make_clear_confirm_dialog(ctx);
        let import_name = self.make_import_name_modal(ctx);

        egui::TopBottomPanel::top("menu").show(ctx, |ui| {
            self.make_menu(ui, confirm_clear, import_name);
        });

        egui::SidePanel::left("search").show(ctx, |ui| {
            self.make_sidebar(ui);
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            self.make_tab_list(ui);
            ui.separator();
            self.make_tab_view(ui);
        });

        self.toasts.show(ctx);

        self.check_resolve_result();

        if !self.is_resolving() && self.cache_requires_update {
            let to_resolve: Vec<DocIdentifier> = self
                .cache
                .into_iter()
                .filter_map(|(id, state)| if state.to_resolve { Some(id) } else { None })
                .cloned()
                .collect();

            self.task_resolve_dependencies(
                ResolveTarget::Multiple(to_resolve),
                ResolveParams {
                    print: true,
                    depth: self.settings.max_depth,
                    query: true,
                },
            );

            self.cache_requires_update = false;
        }
    }
}
