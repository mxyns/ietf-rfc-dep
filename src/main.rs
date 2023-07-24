mod cache;
mod doc;

use std::collections::{HashMap, HashSet};
use crate::cache::{CachedDoc, DocCache};
use crate::doc::{DocRef, Meta};
use crate::doc::IetfDoc;

fn main() {
    let mut cache = DocCache::new();
    let doc = IetfDoc::from_url("https://datatracker.ietf.org/doc/rfc4271");
    cache.put_doc(doc);

    let mut loop_count = 0;
    loop {

        loop_count += 1;
        println!("Depth = {loop_count}");

        let mut to_update = HashSet::new();

        // Discover identifiers referenced in the cached documents
        for item in cache.map.iter_mut() {
            let item = item.1.borrow_mut();
            let meta_list: &Vec<Meta> = item.meta.as_ref();

            for meta in meta_list {
                match meta {
                    Meta::Updates(list)
                    | Meta::Obsoletes(list)
                    | Meta::UpdatedBy(list) => {
                        for item in list {
                            match item {
                                DocRef::Identifier(id) => {
                                    to_update.insert(id.clone());
                                }
                                DocRef::CacheEntry(_) => {}
                            };
                        };
                    }
                    Meta::Was(_) => {}
                }
            }
        }

        if to_update.len() == 0 {
            break
        }


        // Query uncached documents
        let mut id_doc_new = HashMap::<String, CachedDoc>::new();
        for id in to_update {

            // Filter out the ones that are already cached
            if cache.map.contains_key(&id) {
                continue;
            }

            // Query document and cache them
            let doc = IetfDoc::from_url(format!("https://datatracker.ietf.org/doc/{}", id).as_str());
            let cached = cache.put_doc(doc);
            id_doc_new.insert(id, cached);
        }

        // Copy cache to lookup already existing documents when linking
        let old_cache = cache.clone();

        // Update current cache with new documents and new links
        for item in cache.map.iter_mut() {
            let item_ref = &mut *item.1.borrow_mut();
            for meta in &mut item_ref.meta {
                match meta {
                    Meta::Updates(list)
                    | Meta::Obsoletes(list)
                    | Meta::UpdatedBy(list) => {
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
                    Meta::Was(_) => {}
                }
            }
        }
    }

    println!("{:#?}", cache);
}
