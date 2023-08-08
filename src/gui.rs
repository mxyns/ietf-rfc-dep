use std::collections::{HashSet};
use std::fs::File;
use eframe::egui;
use eframe::egui::Align;
use egui_extras::{Column, TableBuilder};
use if_chain::if_chain;
use serde::{Deserialize, Serialize};
use crate::doc::{DocIdentifier, IetfDoc, Meta};
use crate::cache::{Cache, CacheReference, RelationalEntry, ResolvableEntry};

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
struct StatefulDoc {
    // Target document
    content: IetfDoc,

    // Real State
    is_read: bool,
    is_selected: bool,
    missing_dep_count: usize,

    // Temporary State
    to_resolve: bool,
}

impl StatefulDoc {
    fn new(doc: IetfDoc) -> StatefulDoc {
        StatefulDoc {
            missing_dep_count: doc.missing(),
            content: doc,
            is_read: false,
            is_selected: false,
            to_resolve: false,
        }
    }
}

fn update_missing_dep_count(doc: &mut StatefulDoc, new_deps: isize) {
    doc.missing_dep_count = (doc.missing_dep_count as isize - new_deps) as usize;
}

// Implement resolve dependency algorithms when value is IetfDoc
impl RelationalEntry<DocIdentifier> for StatefulDoc {
    fn get_unknown_relations(&self) -> HashSet<DocIdentifier> {
        let mut to_update = HashSet::new();
        for meta in &self.content.meta {
            match meta {
                Meta::Updates(list)
                | Meta::Obsoletes(list)
                | Meta::UpdatedBy(list)
                | Meta::ObsoletedBy(list) => {
                    for item in list {
                        match item {
                            CacheReference::Unknown(id) => {
                                to_update.insert(id.clone());
                            }
                            CacheReference::Cached(_) => {}
                        };
                    };
                }
                Meta::Was(_) => {}
            }
        }

        to_update
    }

    fn update_unknown_references(&mut self, is_known: impl Fn(&DocIdentifier) -> bool) -> isize {
        let mut change = 0;
        for meta in &mut self.content.meta {
            match meta {
                Meta::Updates(list)
                | Meta::Obsoletes(list)
                | Meta::UpdatedBy(list)
                | Meta::ObsoletedBy(list) => {
                    for item in list {
                        let (CacheReference::Cached(ref_id) | CacheReference::Unknown(ref_id)) = item.clone();
                        let is_known = is_known(&ref_id);

                        // was unknown
                        if let CacheReference::Unknown(_) = item {
                            if is_known {
                                change += 1;
                                *item = CacheReference::Cached(ref_id);
                            }
                        } else { // was known
                            if !is_known {
                                change -= 1;
                                *item = CacheReference::Unknown(ref_id);
                            }
                        }
                    };
                }
                Meta::Was(_) => {}
            }
        }

        change
    }
}

impl ResolvableEntry<DocIdentifier> for StatefulDoc {
    fn get_value(id: DocIdentifier) -> Self {
        StatefulDoc::new(IetfDoc::from_url(format!("https://datatracker.ietf.org/doc/{}", id)))
    }
}

#[derive(Default, Debug)]
pub struct RFCDepApp {
    // Lookup Related
    search_query: String,
    query_result: Vec<IetfDoc>,
    selected_query_docs: Vec<bool>,

    // Settings
    max_depth: usize,

    // Doc State
    cache: Cache<DocIdentifier, StatefulDoc>,
    cache_requires_update: bool,
    list_selected_count: usize,

    // RFC Viewer
    selected_tab: usize,
}

impl RFCDepApp {
    pub(crate) fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        let app = RFCDepApp {
            ..Self::default()
        };

        app
    }

    fn query_docs(&mut self) {
        self.query_result = IetfDoc::lookup(self.search_query.as_str());
        self.selected_query_docs = vec![false; self.query_result.len()];

        println!("{:#?}", self.query_result);
        println!("{:#?}", self.selected_query_docs);
    }

    fn merge_caches(&mut self, other: Cache<DocIdentifier, StatefulDoc>) {
        self.cache.merge_with(other);
        self.update_cache(None);
    }


    fn update_cache(&mut self, new_cache: Option<Cache<DocIdentifier, StatefulDoc>>) {
        // Check if import resolved some dependencies
        // Do not query new documents, use only the already provided
        // Max depth = 1
        if let Some(new_cache) = new_cache {
            self.cache = new_cache;
        }
        self.cache.resolve_all_dependencies(true, 1, false, update_missing_dep_count);
    }

    fn reset(&mut self) {
        self.cache.clear();
        self.list_selected_count = 0;
        self.cache_requires_update = false;
    }
}

impl eframe::App for RFCDepApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {

        let confirm_clear = egui_modal::Modal::new(ctx, "confirm_clear")
            .with_close_on_outside_click(false);
        confirm_clear.show(|ui| {
            confirm_clear.title(ui, "Confirm Clear");
            confirm_clear.body_and_icon(ui, "Clearing the current state will result in loss of any unsaved change", egui_modal::Icon::Warning);
            confirm_clear.buttons(ui, |ui| {
                if confirm_clear.caution_button(ui, "cancel").clicked() { confirm_clear.close() }
                if confirm_clear.suggested_button(ui, "clear").clicked() {
                    self.reset();
                    println!("{:#?}", self.cache);
                };
            });
        });

        egui::TopBottomPanel::top("menu").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                ui.menu_button("File", |ui| {
                    // Open Button
                    if_chain! {
                        if ui.button("Open").clicked();
                        if let Some(path) = rfd::FileDialog::new()
                            .add_filter("json", &["json"])
                            .pick_file();
                        if let Ok(file) = File::open(path);
                        then {
                            self.update_cache(Some(
                                serde_json::from_reader(file).unwrap()
                            ));
                        }
                    }

                    // Save Button
                    if_chain! {
                        if ui.button("Save").clicked();
                        if let Some(path) = rfd::FileDialog::new()
                            .add_filter("json", &["json"])
                            .save_file();
                        if let Ok(file) = &File::create(path);
                        then {
                            serde_json::to_writer_pretty(file, &self.cache).unwrap();
                        }
                    }

                    // Import Button
                    if_chain! {
                        if ui.button("Import").clicked();
                        if let Some(path) = rfd::FileDialog::new()
                            .add_filter("json", &["json"])
                            .pick_file();
                        if let Ok(file) = File::open(path);
                        then {
                            let new_state: Cache<DocIdentifier, StatefulDoc> = serde_json::from_reader(file).unwrap();
                            println!("{:#?}", new_state);
                            self.merge_caches(new_state);
                            println!("{:#?}", self.cache);
                        }
                    }

                    ui.separator();

                    // Clear Button
                    if ui.button("Clear").clicked() {
                        confirm_clear.open();
                    }
                });

                let cache_size = self.cache.len();
                ui.add_enabled_ui(cache_size > 0, |ui| {
                    ui.menu_button("Select", |ui| {
                        ui.add_enabled_ui(self.list_selected_count < cache_size, |ui| {
                            if ui.button("Select All").clicked() {
                                (&mut self.cache).into_iter().for_each(|(_, state)| {
                                    state.is_selected = true;
                                });
                                self.list_selected_count = self.cache.len();
                            }
                        });

                        ui.add_enabled_ui(self.list_selected_count >= cache_size, |ui| {
                            if ui.button("Deselect All").clicked() {
                                (&mut self.cache).into_iter().for_each(|(_, state)| {
                                    state.is_selected = false;
                                });
                                self.list_selected_count = 0;
                            }
                        });
                        ui.add_enabled_ui(self.list_selected_count > 0, |ui| {
                            if ui.button("Remove selected").clicked() {
                                self.cache.retain(|_, state| state.is_selected == false);
                                self.update_cache(None);
                                self.list_selected_count = 0;
                            }
                        });
                    });
                });

                // update value since cache may have been updated
                let cache_size = self.cache.len();
                ui.add_enabled_ui(cache_size > 0, |ui| {
                    ui.menu_button("Resolve", |ui| {
                        ui.add_enabled_ui(self.list_selected_count > 0, |ui| {
                            if ui.button("Resolve Selected").clicked() {
                                for (_id, doc) in (&mut self.cache).into_iter() {
                                    if doc.is_selected {
                                        doc.to_resolve = true;
                                        self.cache_requires_update = true;
                                    }
                                }
                            }
                        });

                        if ui.button("Resolve All").clicked() {
                            self.cache.resolve_all_dependencies(true, self.max_depth.clone(), true, update_missing_dep_count);
                        }
                    });
                });
            });
        });

        egui::SidePanel::left("search").show(ctx, |ui| {
            ui.vertical(|ui| {
                ui.with_layout(egui::Layout::left_to_right(Align::TOP), |ui| {
                    if (ui.add(egui::TextEdit::singleline(&mut self.search_query).hint_text("search datatracker.ietf.org")).lost_focus()
                        && ui.input(|i| i.key_pressed(egui::Key::Enter)))
                        || ui.button("lookup").clicked() {
                        self.query_docs();
                    }
                });
                ui.end_row();

                ui.with_layout(egui::Layout::bottom_up(Align::LEFT), |ui| {
                    ui.with_layout(egui::Layout::right_to_left(Align::BOTTOM), |ui| {
                        ui.add(egui::DragValue::new(&mut self.max_depth).suffix(" max depth").clamp_range(std::ops::RangeInclusive::new(1, u64::MAX)));

                        if ui.button("include").clicked() {
                            let selected = &self.selected_query_docs;
                            let mut results: Vec<IetfDoc> = selected.iter().enumerate()
                                .filter_map(|(i, b)| if *b { Some(i) } else { None })
                                .map(|i| self.query_result.get(i).unwrap().clone()).collect();

                            results.drain(..).for_each(|doc| {
                                self.cache.cache(doc.name.clone(), StatefulDoc::new(doc));
                            });
                        }
                    });

                    ui.separator();

                    ui.with_layout(egui::Layout::top_down(Align::LEFT), |ui| {
                        egui::ScrollArea::vertical().drag_to_scroll(true).show_rows(ui, 10.0, self.query_result.len(), |ui, range| {
                            for (idx, doc) in self.query_result[range].iter().enumerate() {
                                ui.separator();
                                ui.checkbox(self.selected_query_docs.get_mut(idx).unwrap_or(&mut false), &doc.title);
                                ui.label(&doc.name);
                                ui.hyperlink_to("datatracker", &doc.url);
                            }
                            ui.separator();
                        });
                    });
                });

                ui.end_row();
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            let tabs = Tabs::all();

            ui.with_layout(egui::Layout::left_to_right(Align::TOP), |ui| {
                tabs.iter().for_each(|tab| {
                    let item: TabItem = tab.clone().into();
                    (item.ui.unwrap())(ui, self);
                });
            });

            ui.separator();

            let name_to_href = |ui: &mut egui::Ui, s: &String| {
                ui.hyperlink_to(s, format!("https://datatracker.ietf.org/doc/{s}"))
            };

            let selected_tab = tabs.get(self.selected_tab.clone()).unwrap();
            match selected_tab {
                Tabs::List => {
                    TableBuilder::new(ui)
                        .striped(true)
                        .vscroll(true)
                        .column(Column::initial(20.0).clip(true).resizable(true))
                        .column(Column::initial(30.0).clip(true).resizable(true))
                        .column(Column::initial(30.0).clip(true).resizable(true))
                        .column(Column::initial(50.0).clip(true).resizable(true))
                        .column(Column::initial(160.0).clip(true).resizable(true))
                        .column(Column::initial(50.0).clip(true).resizable(true))
                        .column(Column::initial(30.0).clip(true).resizable(true))
                        .column(Column::initial(75.0).clip(true).resizable(true))
                        .column(Column::initial(75.0).clip(true).resizable(true))
                        .column(Column::initial(75.0).clip(true).resizable(true))
                        .column(Column::remainder())
                        .header(10.0, |mut header| {
                            vec!["", "Missing", "Read", "Name", "Title", "Relations", "Was", "Updates", "Obsoletes", "Updated By", "Obsoleted By"].drain(..).for_each(
                                |x| {
                                    header.col(|ui| {
                                        ui.label(x);
                                    });
                                }
                            );
                        })
                        .body(|mut body| {
                            for (id, state) in (&mut self.cache).into_iter() {
                                body.row(20.0, |mut row| {
                                    row.col(|ui| {
                                        if ui.checkbox(&mut state.is_selected, "").clicked() {
                                            if state.is_selected.clone() {
                                                self.list_selected_count += 1
                                            } else {
                                                self.list_selected_count -= 1
                                            }
                                        }
                                    });
                                    row.col(|ui| {
                                        let missing = &state.missing_dep_count;
                                        if missing > &0 && ui.small_button(format!("+ {}", missing)).clicked() {
                                            state.to_resolve = true;
                                            self.cache_requires_update = true;
                                        };
                                    });

                                    let doc = &state.content;
                                    row.col(|ui| { ui.checkbox(&mut state.is_read, ""); });
                                    row.col(|ui| { name_to_href(ui, id); });
                                    row.col(|ui| { ui.label(doc.title.clone()); });
                                    row.col(|ui| { ui.label(doc.meta_count().to_string()); });
                                    row.col(|ui| {
                                        ui.horizontal(|ui| {
                                            for meta in &doc.meta {
                                                if let Meta::Was(id) = meta {
                                                    name_to_href(ui, id);
                                                }
                                            }
                                        });
                                    });
                                    row.col(|ui| {
                                        ui.horizontal(|ui| {
                                            for meta in &doc.meta {
                                                if let Meta::Updates(list) = meta {
                                                    for meta in list {
                                                        match meta {
                                                            CacheReference::Unknown(id) => { name_to_href(ui, id); }
                                                            CacheReference::Cached(id) => { name_to_href(ui, id); }
                                                        }
                                                    }
                                                }
                                            }
                                        });
                                    });
                                    row.col(|ui| {
                                        ui.horizontal(|ui| {
                                            for meta in &doc.meta {
                                                if let Meta::Obsoletes(list) = meta {
                                                    for meta in list {
                                                        match meta {
                                                            CacheReference::Unknown(id) => { name_to_href(ui, id); }
                                                            CacheReference::Cached(id) => { name_to_href(ui, id); }
                                                        }
                                                    }
                                                }
                                            }
                                        });
                                    });
                                    row.col(|ui| {
                                        ui.horizontal(|ui| {
                                            for meta in &doc.meta {
                                                if let Meta::UpdatedBy(list) = meta {
                                                    for meta in list {
                                                        match meta {
                                                            CacheReference::Unknown(id) => { name_to_href(ui, id); }
                                                            CacheReference::Cached(id) => { name_to_href(ui, id); }
                                                        }
                                                    }
                                                }
                                            }
                                        });
                                    });
                                    row.col(|ui| {
                                        ui.horizontal(|ui| {
                                            for meta in &doc.meta {
                                                if let Meta::ObsoletedBy(list) = meta {
                                                    for meta in list {
                                                        match meta {
                                                            CacheReference::Unknown(id) => { name_to_href(ui, id); }
                                                            CacheReference::Cached(id) => { name_to_href(ui, id); }
                                                        }
                                                    }
                                                }
                                            }
                                        });
                                    });
                                });
                            }
                        });
                }
                Tabs::Graph => {
                    ui.label("TODO");
                }
            }
        });

        if self.cache_requires_update {
            let to_resolve: Vec<DocIdentifier> = self.cache.into_iter()
                .filter_map(|(id, state)| {
                    if state.to_resolve { Some(id) } else { None }
                }).cloned().collect();

            for to_resolve in to_resolve {
                self.cache.resolve_entry_dependencies(to_resolve, true, self.max_depth.clone(), true, update_missing_dep_count);
            }


            self.cache_requires_update = false;
        }
    }
}

#[derive(Default, Clone)]
pub struct TabItem {
    pub title: String,
    pub ui: Option<fn(&mut egui::Ui, &mut RFCDepApp)>,
}

#[derive(Clone)]
pub enum Tabs {
    List,
    Graph,
}

impl Into<TabItem> for Tabs {
    fn into(self) -> TabItem {
        match self {
            Tabs::List => TabItem {
                title: "list".to_string(),
                ui: Some(|ui, app| {
                    if ui.button("list").clicked() { app.selected_tab = 0; }
                }),
            },
            Tabs::Graph => TabItem {
                title: "graph".to_string(),
                ui: Some(|ui, app| {
                    if ui.button("graph").clicked() { app.selected_tab = 1; }
                }),
            },
        }
    }
}

impl Tabs {
    pub(crate) fn all() -> Vec<Tabs> {
        vec![
            Tabs::List,
            Tabs::Graph,
        ]
    }
}