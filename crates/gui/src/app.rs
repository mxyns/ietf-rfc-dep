use eframe::egui;
use rfc_dep_ietf::{DocIdentifier, IetfDoc};
use rfc_dep_cache::{Cache, ResolveParams, ResolveTarget};
use crate::doc::{StatefulDoc, update_missing_dep_count};
use crate::tabs::{Tab};
use crate::settings::{Settings};

#[derive(Default, Debug)]
pub struct RFCDepApp {
    // Lookup Related
    pub(crate) search_query: String,
    pub(crate) query_result: Vec<IetfDoc>,
    pub(crate) selected_query_docs: Vec<bool>,

    // Settings
    pub(crate) settings: Settings,

    // Doc State
    pub(crate) cache: Cache<DocIdentifier, StatefulDoc>,
    pub(crate) cache_requires_update: bool,
    pub(crate) list_selected_count: usize,

    // RFC Viewer
    pub(crate) selected_tab: Tab,
}

impl RFCDepApp {
    pub(crate) fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        let app = RFCDepApp {
            ..Self::default()
        };

        app
    }

    pub(crate) fn query_docs(&mut self) {
        self.query_result = IetfDoc::lookup(self.search_query.as_str(),
                                            self.settings.query.limit,
                                            self.settings.query.rfc_only);
        self.selected_query_docs = vec![false; self.query_result.len()];

        println!("{:#?}", self.query_result);
        println!("{:#?}", self.selected_query_docs);
    }

    pub(crate) fn merge_caches(&mut self, other: Cache<DocIdentifier, StatefulDoc>) {
        self.cache.merge_with(other);
        self.update_cache(None);
    }


    pub(crate) fn update_cache(&mut self, new_cache: Option<Cache<DocIdentifier, StatefulDoc>>) {
        // Check if import resolved some dependencies
        // Do not query new documents, use only the already provided
        // Max depth = 1
        if let Some(new_cache) = new_cache {
            self.cache = new_cache;
        }

        self.cache.resolve_dependencies(ResolveTarget::All, ResolveParams {
            print: true,
            depth: 1,
            query: false,
        }, update_missing_dep_count);
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

        egui::TopBottomPanel::top("menu").show(ctx, |ui| {
            self.make_menu(ui, confirm_clear);
        });

        egui::SidePanel::left("search").show(ctx, |ui| {
            self.make_sidebar(ui);
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            self.make_tab_list(ui);
            ui.separator();
            self.make_tab_view(ui);
        });

        if self.cache_requires_update {
            let to_resolve: Vec<DocIdentifier> = self.cache.into_iter()
                .filter_map(|(id, state)| {
                    if state.to_resolve { Some(id) } else { None }
                }).cloned().collect();

            self.cache.resolve_dependencies(ResolveTarget::Multiple(to_resolve),
                                            ResolveParams {
                                                print: true,
                                                depth: self.settings.max_depth.clone(),
                                                query: true,
                                            }, update_missing_dep_count);

            self.cache_requires_update = false;
        }
    }
}