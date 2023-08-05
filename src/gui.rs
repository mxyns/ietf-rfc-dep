use std::cell::RefCell;
use std::collections::HashMap;
use eframe::egui;
use eframe::egui::Align;
use egui_extras::{Column, TableBuilder};
use crate::doc::{DocRef, IetfDoc, Meta};
use crate::cache::{CachedDoc, DocCache};
use crate::doc::DocRef::Identifier;

#[derive(Clone, Debug)]
struct DocState {
    cache: CachedDoc,
    read: bool,
    to_resolve: bool,
}

type DocStates = RefCell<HashMap<String, DocState>>;

#[derive(Default)]
pub struct RFCDepApp {
    search_query: String,
    query_result: Vec<IetfDoc>,
    selected_docs: Vec<bool>,

    max_depth: usize,

    cache: DocCache,
    docs_state: DocStates,


    // RFC Viewer
    selected_tab: usize,
    // tabs: TabbedLayout<TabItem>,
}

impl RFCDepApp {
    pub(crate) fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        let app = RFCDepApp {
            ..Self::default()
        };

        app
    }

    fn put_doc(cache: &mut DocCache, docs_state: &mut DocStates, doc: IetfDoc) {
        let name = doc.name.clone();
        let cached = cache.put_doc(doc);

        docs_state.borrow_mut().insert(name, DocState {
            cache: cached,
            read: false,
            to_resolve: false,
        });
    }

    fn query_docs(&mut self) {
        self.query_result = IetfDoc::lookup(self.search_query.as_str());
        self.selected_docs = vec![false; self.query_result.len()];

        println!("{:#?}", self.query_result);
        println!("{:#?}", self.selected_docs);
    }

    fn refresh_docs_state(&self) {
        let mut docs_state = self.docs_state.borrow_mut();

        for (name, cached) in self.cache.map.iter() {
            if !docs_state.contains_key(&*name) {
                docs_state.insert(name.clone(), DocState {
                    cache: cached.clone(),
                    read: false,
                    to_resolve: false,
                });
            }

        }
    }
}

impl eframe::App for RFCDepApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
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
                        ui.add(egui::DragValue::new(&mut self.max_depth).suffix(" max depth").clamp_range(1..=256));

                        if ui.button("include").clicked() {
                            self.selected_docs.iter().enumerate()
                                .filter_map(|(i, b)| if *b { Some(i) } else { None })
                                .map(|i| self.query_result.get(i).unwrap())
                                .cloned()
                                .for_each(|doc| { RFCDepApp::put_doc(&mut self.cache, &mut self.docs_state, doc); })
                        }
                    });

                    ui.separator();

                    ui.with_layout(egui::Layout::top_down(Align::LEFT), |ui| {
                        egui::ScrollArea::vertical().drag_to_scroll(true).show_rows(ui, 10.0, self.query_result.len(), |ui, range| {
                            for (idx, doc) in self.query_result[range].iter().enumerate() {
                                ui.separator();
                                ui.checkbox(self.selected_docs.get_mut(idx).unwrap_or(&mut false), &doc.title);
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

            let name_to_href = |ui: &mut egui::Ui, s: &String| {
                ui.hyperlink_to(s, format!("https://datatracker.ietf.org/doc/{s}"))
            };

            let selected_tab = tabs.get(self.selected_tab).unwrap();
            match selected_tab {
                Tabs::List => {
                    TableBuilder::new(ui)
                        .striped(true)
                        .vscroll(true)
                        .auto_shrink([false, false])
                        .column(Column::initial(20.0).clip(true).resizable(true))
                        .column(Column::initial(20.0).clip(true).resizable(true))
                        .column(Column::initial(40.0).clip(true).resizable(true))
                        .column(Column::initial(80.0).clip(true).resizable(true))
                        .column(Column::initial(50.0).clip(true).resizable(true))
                        .column(Column::initial(50.0).clip(true).resizable(true))
                        .column(Column::initial(50.0).clip(true).resizable(true))
                        .column(Column::initial(50.0).clip(true).resizable(true))
                        .column(Column::remainder())
                        .header(10.0, |mut header| {
                            vec!["dep", "Read", "Name", "Title", "Relations", "Updates", "Obsoletes", "Updated By", "Obsoleted By"].drain(..).for_each(
                                |x| {
                                    header.col(|ui| {
                                        ui.label(x);
                                    });
                                }
                            );
                        })
                        .body(|mut body| {
                            for (name, doc) in self.docs_state.borrow_mut().iter_mut() {
                                body.row(20.0, |mut row| {
                                    let cache = doc.cache.borrow();
                                    row.col(|ui| {
                                        if ui.button(format!("+ {}", cache.missing())).clicked() {
                                            doc.to_resolve = true;
                                        };
                                    });
                                    row.col(|ui| { ui.checkbox(&mut doc.read, ""); });
                                    row.col(|ui| { name_to_href(ui, name); });
                                    row.col(|ui| { ui.label(cache.title.clone()); });
                                    row.col(|ui| { ui.label(cache.meta.len().to_string() ); });
                                    row.col(|ui| {
                                        ui.horizontal(|ui| {
                                            for meta in &cache.meta {
                                                if let Meta::Updates(list) = meta {
                                                    for meta in list {
                                                        match meta {
                                                            Identifier(id) => { name_to_href(ui, id); }
                                                            DocRef::CacheEntry(entry) => { name_to_href(ui, &entry.borrow().name.clone()); }
                                                        }
                                                    }
                                                }
                                            }
                                        });
                                    });
                                    row.col(|ui| {
                                        ui.horizontal(|ui| {
                                            for meta in &cache.meta {
                                                if let Meta::Obsoletes(list) = meta {
                                                    for meta in list {
                                                        match meta {
                                                            Identifier(id) => { name_to_href(ui, id); }
                                                            DocRef::CacheEntry(entry) => { name_to_href(ui, &entry.borrow().name.clone()); }
                                                        }
                                                    }
                                                }
                                            }
                                        });
                                    });
                                    row.col(|ui| {
                                        ui.horizontal(|ui| {
                                            for meta in &cache.meta {
                                                if let Meta::UpdatedBy(list) = meta {
                                                    for meta in list {
                                                        match meta {
                                                            Identifier(id) => { name_to_href(ui, id); }
                                                            DocRef::CacheEntry(entry) => { name_to_href(ui, &entry.borrow().name.clone()); }
                                                        }
                                                    }
                                                }
                                            }
                                        });
                                    });
                                    row.col(|ui| {
                                        ui.horizontal(|ui| {
                                            for meta in &cache.meta {
                                                if let Meta::ObsoletedBy(list) = meta {
                                                    for meta in list {
                                                        match meta {
                                                            Identifier(id) => { name_to_href(ui, id); }
                                                            DocRef::CacheEntry(entry) => { name_to_href(ui, &entry.borrow().name.clone()); }
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

        for (_, doc) in self.docs_state.borrow_mut().iter_mut() {
            if !&doc.to_resolve { continue }

            let ietf_doc: IetfDoc = doc.cache.borrow().clone();
            self.cache.resolve_dependencies(ietf_doc, true, self.max_depth);

            doc.to_resolve = false;
        }

        self.refresh_docs_state();
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