use std::collections::{HashMap, HashSet};
use crate::doc::{DocRef, IetfDoc, Meta};
use serde;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DocCache {
    pub map: HashMap<String, IetfDoc>,
}

impl DocCache {

    pub fn resolve_dependencies(&mut self, print: bool, max_depth: usize, query: bool) {

        let mut depth = 0;
        loop {
            depth += 1;
            if print {
                println!("Depth = {depth}");
            }
            if depth > max_depth {
                println!("Reached max depth = {max_depth}");
                break;
            }

            let mut to_update = HashSet::new();

            // Discover identifiers referenced in the cached documents
            for (_, doc) in self.map.iter_mut() {
                let meta_list: &Vec<Meta> = doc.meta.as_ref();

                for meta in meta_list {
                    match meta {
                        Meta::Updates(list)
                        | Meta::Obsoletes(list)
                        | Meta::UpdatedBy(list)
                        | Meta::ObsoletedBy(list) => {
                            for item in list {
                                match item {
                                    DocRef::Identifier(id) => {
                                        to_update.insert(id.clone());
                                    }
                                    DocRef::CacheEntry(_) => {}
                                };
                            };
                        }
                        Meta::Was(_) | Meta::None => {}
                    }
                }
            }

            if to_update.len() == 0 {
                break;
            }
            // println!("{:#?}", self);
            // println!("{:#?}", to_update);


            // Query uncached documents
            let mut id_doc_new = HashSet::<String>::new();
            if query {
                for id in to_update {

                    // Filter out the ones that are already cached
                    if self.map.contains_key(&id) {
                        continue;
                    }

                    // Query document and cache them
                    let doc = IetfDoc::from_url(format!("https://datatracker.ietf.org/doc/{}", id));
                    self.map.insert(id.clone(), doc);
                    id_doc_new.insert(id);
                }
            }

            // println!("{:#?}", id_doc_new);

            // Copy cache to lookup already existing documents when linking
            let old_cache = self.clone();

            // Update current cache with new documents and new links
            for (_id, doc) in self.map.iter_mut() {
                for meta in &mut doc.meta {
                    match meta {
                        Meta::Updates(list)
                        | Meta::Obsoletes(list)
                        | Meta::UpdatedBy(list)
                        | Meta::ObsoletedBy(list) => {
                            for item in list {
                                match item {
                                    DocRef::Identifier(id) => {
                                        if let Some(cached) = id_doc_new.get(id.as_str()) {
                                            *item = DocRef::CacheEntry(cached.clone());
                                        } else if let Some(cached) = old_cache.map.get(id) {
                                            *item = DocRef::CacheEntry(cached.name.clone());
                                        } else {
                                            // Item to be discovered at next iteration
                                        }
                                    }
                                    DocRef::CacheEntry(_) => {}
                                };
                            };
                        }
                        Meta::Was(_) | Meta::None => {}
                    }
                }
            }

            // println!("{:#?}", cache);
        }
    }
}