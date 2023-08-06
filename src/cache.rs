use std::collections::{HashMap, HashSet};
use std::fmt;
use std::fmt::Debug;
use std::hash::Hash;
use serde;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Cache<IdType: Eq + Hash, ValueType> {
    map: HashMap<IdType, ValueType>,
}

#[derive(Clone, Serialize, Deserialize)]
pub enum CacheReference<IdType> {
    Unknown(IdType),
    Cached(IdType),
}

/* debug print for struct CacheReference */
impl<IdType: fmt::Display> Debug for CacheReference<IdType> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            CacheReference::Unknown(id) => { write!(f, "Unknown(\"{}\")", id) }
            CacheReference::Cached(id) => { write!(f, "Cached(\"{}\")", id) }
        }
    }
}

/* Cache API */
impl<IdType: Eq + Hash, ValueType> Cache<IdType, ValueType> {
    /* get value with identifier id from cache */
    pub fn get(&self, id: &IdType) -> Option<&ValueType> {
        self.map.get(id)
    }

    /* put value in case with identifier id */
    pub fn cache(&mut self, id: IdType, value: ValueType) -> Option<ValueType> {
        self.map.insert(id, value)
    }

    /* returns true if the id is used in cache */
    pub fn has_id(&self, id: &IdType) -> bool {
        self.map.contains_key(id)
    }

    /* consumes another cache and inserts its entries in the current cache */
    pub fn merge_with(&mut self, other: Cache<IdType, ValueType>) {
        self.map.extend(other.map)
    }

    /* clear all cache entries */
    pub fn clear(&mut self) {
        self.map.clear()
    }

    /* retain only entries matching f */
    pub fn retain<F>(&mut self, f: F)
        where
            F: FnMut(&IdType, &mut ValueType) -> bool,
    {
        self.map.retain(f)
    }

    /* remove entry */
    pub fn remove(&mut self, id: &IdType) -> Option<ValueType> {
        self.map.remove(id)
    }
}


/* allow to iter on cache reference */
impl<'h, IdType: Eq + Hash, ValueType> IntoIterator for &'h Cache<IdType, ValueType> {
    type Item = <&'h HashMap<IdType, ValueType> as IntoIterator>::Item;
    type IntoIter = <&'h HashMap<IdType, ValueType> as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        (&self.map).into_iter()
    }
}

/* allow to iter_mut on cache mut reference */
impl<'h, IdType: Eq + Hash, ValueType> IntoIterator for &'h mut Cache<IdType, ValueType> {
    type Item = <&'h mut HashMap<IdType, ValueType> as IntoIterator>::Item;
    type IntoIter = <&'h mut HashMap<IdType, ValueType> as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        (&mut self.map).into_iter()
    }
}

/* represents an entry containing references to other entries */
pub trait RelationalEntry<IdType> {
    fn get_relations(&self) -> HashSet<IdType>;
    fn update_reference(&mut self, id: &IdType, is_known: impl Fn(&IdType) -> bool);
}

/* represents an entry which value can be retrieved using only its id
 *   (eg: a document using an url, a file from its path, etc.)
 */
pub trait ResolvableEntry<IdType> {
    fn get_value(id: IdType) -> Self;
}

/* resolve all dependencies in the cache
 * values must have relations to others (dependencies)
 * and must be resolvable to get (at least) their own dependencies
 */
impl<IdType, ValueType> Cache<IdType, ValueType>
    where
        IdType: Eq + Hash + Clone + fmt::Display + Debug,
        ValueType: RelationalEntry<IdType> + ResolvableEntry<IdType> + Clone + Debug
{
    pub fn resolve_dependencies(&mut self, print: bool, max_depth: usize, resolve: bool) {
        let mut depth = 0;
        loop {
            let mut to_update = HashSet::<IdType>::new();

            // Discover identifiers referenced in the cached documents
            for (_, doc) in self.into_iter() {
                to_update.extend(doc.get_relations())
            }

            if to_update.len() == 0 {
                if print { println!("early stop, no new entries found"); }
                break;
            }

            // println!("{:#?}", self);
            // println!("{:#?}", to_update);

            // Query uncached documents
            let mut id_doc_new = HashSet::<IdType>::new();
            if resolve {
                for id in to_update {

                    // Filter out the ones that are already cached
                    if self.has_id(&id) {
                        continue;
                    }

                    // Query document and cache them
                    let doc = ValueType::get_value(id.clone());
                    self.cache(id.clone(), doc);
                    id_doc_new.insert(id.clone());
                }
            }

            // println!("{:#?}", id_doc_new);

            // Copy cache to lookup already existing entries when linking
            let old_ids: HashSet<IdType> = self.map.keys().cloned().collect();

            // Update current cache with new entries and new relations
            for (id, doc) in &mut self.into_iter() {
                doc.update_reference(id, |meta_id| {
                    id_doc_new.get(meta_id).is_some() || old_ids.contains(meta_id)
                });
            }

            // println!("{:#?}", cache);

            depth += 1;

            if print {
                println!("Depth = {depth}");
            }
            if depth >= max_depth {
                println!("Reached max depth = {max_depth}");
                break;
            }
        }
    }

    pub fn resolve_entry_dependencies(&mut self, root: IdType, print: bool, max_depth: usize, resolve: bool) {
        if print {
            println!("Resolving for {}", &root);
        }

        let mut depth = 0;
        let mut updated = HashSet::<IdType>::from([root]);

        loop {
            let mut to_update = HashSet::<IdType>::new();

            // Discover identifiers referenced in the cached documents
            for id in &updated {
                to_update.extend(self.get(id).unwrap().get_relations())
            }
            updated.clear();

            if to_update.len() == 0 {
                if print { println!("early stop, no new entries found"); }
                break;
            }

            // println!("{:#?}", self);
            // println!("{:#?}", to_update);

            // Query uncached documents
            let mut id_doc_new = HashSet::<IdType>::new();
            if resolve {
                for id in to_update {

                    // Filter out the ones that are already cached
                    if self.has_id(&id) {
                        continue;
                    }

                    // Query document and cache them
                    let doc = ValueType::get_value(id.clone());
                    self.cache(id.clone(), doc);
                    id_doc_new.insert(id.clone());
                }
            }

            // println!("id_doc_new = {:#?}", id_doc_new);

            // Copy cache to lookup already existing entries when linking
            let old_ids: HashSet<IdType> = self.map.keys().cloned().collect();
            // println!("old_ids = {:#?}", old_ids);

            // Update current cache with new entries and new relations
            for (id, doc) in &mut self.into_iter() {
                doc.update_reference(id, |meta_id| {
                    id_doc_new.get(meta_id).is_some() || old_ids.contains(meta_id)
                });
                updated.insert(id.clone());
            }

            println!("new_cache = {:#?}", self);

            depth += 1;

            if print {
                println!("Depth = {depth}");
            }
            if depth >= max_depth {
                println!("Reached max depth = {max_depth}");
                break;
            }
        }
    }
}