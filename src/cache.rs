use std::cell::{RefCell};
use std::collections::{HashMap, HashSet};
use std::rc::Rc;
use crate::doc::{DocRef, IetfDoc, Meta};
use serde;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde::ser::SerializeMap;

#[derive(Debug, Clone, Default)]
pub struct DocCache {
    pub map: HashMap<String, CachedDoc>,
}

impl Serialize for DocCache {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where S: Serializer,
    {
        let mut map = serializer.serialize_map(Some(self.map.len()))?;
        for (name, cached) in &self.map {
            map.serialize_entry(name, &cached.borrow().clone())?;
        }

        map.end()
    }
}

impl<'de> Deserialize<'de> for DocCache {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: Deserializer<'de>,
    {
        let mut map = HashMap::<String, IetfDoc>::deserialize(deserializer)?;
        let map = map.drain().map(|(id, doc)| {
            (id, Rc::new(RefCell::new(doc)))
        }).collect();

        Ok(
            DocCache {
                map
            }
        )
    }
}

pub type CachedDoc = Rc<RefCell<IetfDoc>>;

impl DocCache {
    pub fn put_doc(&mut self, doc: IetfDoc) -> CachedDoc {
        let name = doc.name.clone();

        return if self.map.contains_key(name.as_str()) {
            self.map.get(name.as_str()).unwrap().clone()
        } else {
            let doc = Rc::new(RefCell::new(doc));
            self.map.insert(name, doc.clone());

            doc
        };
    }

    pub fn get(&self, name: &String) -> Option<CachedDoc> {
        let doc = self.map.get(name);
        if doc.is_some() { // Doc not in cache
            return Some(doc.unwrap().clone());
        }

        return None;
    }

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
            for item in self.map.iter_mut() {
                let item = item.1.borrow_mut();
                let meta_list: &Vec<Meta> = item.meta.as_ref();

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
            let mut id_doc_new = HashMap::<String, CachedDoc>::new();
            if query {
                for id in to_update {

                    // Filter out the ones that are already cached
                    if self.map.contains_key(&id) {
                        continue;
                    }

                    // Query document and cache them
                    let doc = IetfDoc::from_url(format!("https://datatracker.ietf.org/doc/{}", id));
                    let cached = self.put_doc(doc);
                    id_doc_new.insert(id, cached);
                }
            }

            // println!("{:#?}", id_doc_new);

            // Copy cache to lookup already existing documents when linking
            let old_cache = self.clone();

            // Update current cache with new documents and new links
            for item in self.map.iter_mut() {
                let item_ref = &mut *item.1.borrow_mut();
                for meta in &mut item_ref.meta {
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
                                        } else if let Some(cached) = old_cache.get(&id) {
                                            *item = DocRef::CacheEntry(cached.clone());
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