use std::collections::{BTreeMap, HashSet};
use std::fmt;
use std::fmt::Debug;
use std::hash::Hash;
use serde;
use serde::{Deserialize, Serialize};
use rayon::prelude::*;


pub trait CacheIdentifier: Eq + Hash + Ord {}

impl<T> CacheIdentifier for T where T: Eq + Hash + Ord {}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Cache<IdType: CacheIdentifier, ValueType> {
    map: BTreeMap<IdType, ValueType>,
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
impl<IdType: CacheIdentifier, ValueType> Cache<IdType, ValueType> {
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
        self.map.retain(f);
    }

    /* returns number of cache entries */
    pub fn len(&self) -> usize {
        self.map.len()
    }

    /* remove entry */
    pub fn remove(&mut self, id: &IdType) -> Option<ValueType> {
        self.map.remove(id)
    }
}


/* allow to iter on cache reference */
impl<'h, IdType: CacheIdentifier, ValueType> IntoIterator for &'h Cache<IdType, ValueType> {
    type Item = <&'h BTreeMap<IdType, ValueType> as IntoIterator>::Item;
    type IntoIter = <&'h BTreeMap<IdType, ValueType> as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        (&self.map).into_iter()
    }
}

/* allow to iter_mut on cache mut reference */
impl<'h, IdType: CacheIdentifier, ValueType> IntoIterator for &'h mut Cache<IdType, ValueType> {
    type Item = <&'h mut BTreeMap<IdType, ValueType> as IntoIterator>::Item;
    type IntoIter = <&'h mut BTreeMap<IdType, ValueType> as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        (&mut self.map).into_iter()
    }
}

/* represents an entry containing references to other entries */
pub trait RelationalEntry<IdType> {
    // must return all keys of relations still not known in cache (CacheReference::Unknown)
    fn get_unknown_relations(&self) -> HashSet<IdType>;

    // must update all unknown relations of the entry
    // uses the callback 'is_known' to determine from within 'update_reference'
    // if an id is now known in the calling context
    // returns the number of new references
    fn update_unknown_references(&mut self, is_known: impl Fn(&IdType) -> bool) -> isize;
}

/* represents an entry which value can be retrieved using only its id
 *   (eg: a document using an url, a file from its path, etc.)
 */
pub trait ResolvableEntry<IdType> {
    // query the entry's value from its id
    fn get_value(id: IdType) -> Self;
}


impl<IdType, ValueType> Cache<IdType, ValueType>
    where
        IdType: CacheIdentifier + Sync + Clone + fmt::Display + Debug,
        ValueType: ResolvableEntry<IdType> + Send + Clone + Debug {
    /* query values from ids and cache_core the queried values */
    fn query_values(&mut self, ids: impl IntoIterator<Item=IdType>) -> HashSet<IdType> {
        let new_ids: HashSet<IdType> = ids.into_iter()
            .filter(|id| !self.has_id(&id))
            .collect();

        let values: Vec<_> = new_ids.par_iter().map(|id| {
            (id, ValueType::get_value(id.clone()))
        }).collect();

        for (id, value) in values {
            self.cache(id.clone(), value);
        }

        new_ids
    }
}

#[derive(Debug)]
pub enum ResolveTarget<IdType> {
    All,
    Single(IdType),
    Multiple(Vec<IdType>),
}

#[derive(Debug)]
pub struct ResolveParams {
    pub print: bool,
    pub query: bool,
    pub depth: usize,
}

/* resolve all dependencies in the cache
 * values must have relations to others (dependencies)
 * and must be resolvable to get (at least) their own dependencies
 */
impl<IdType, ValueType> Cache<IdType, ValueType>
    where
        IdType: CacheIdentifier + Sync + Clone + fmt::Display + Debug,
        ValueType: RelationalEntry<IdType> + Send + ResolvableEntry<IdType> + Clone + Debug
{
    pub fn resolve_dependencies<F>(&mut self,
                                   target: ResolveTarget<IdType>,
                                   params: ResolveParams,
                                   mut on_rel_change: F)
        where
            F: FnMut(&mut ValueType, isize) -> ()
    {
        let ResolveParams {
            print,
            query,
            depth: max_depth,
        } = params;


        if print {
            println!("Resolving for {:#?} with {:#?}", target, params);
        }

        let mut depth = 0;
        let mut last_updated_opt: Option<HashSet<IdType>>;
        last_updated_opt = match target {
            ResolveTarget::All => { None }
            ResolveTarget::Single(root) => { Some(HashSet::from([root])) }
            ResolveTarget::Multiple(roots) => { Some(HashSet::from_iter(roots)) }
        };

        loop {
            let mut to_update = HashSet::<IdType>::new();

            // Discover identifiers referenced in the cached documents
            if last_updated_opt.is_none() {
                for (_, doc) in self.into_iter() {
                    to_update.extend(doc.get_unknown_relations())
                }
            } else {
                let last_updated = last_updated_opt.as_mut().unwrap();
                for id in &*last_updated {
                    to_update.extend(self.get(id).unwrap().get_unknown_relations())
                }
                last_updated.clear();
            }

            if to_update.len() == 0 {
                if print { println!("early stop, no new entries found"); }
                break;
            }

            // Query uncached documents
            let id_doc_new = if query {
                self.query_values(to_update)
            } else {
                HashSet::<IdType>::new()
            };

            // Copy cache to lookup already existing entries when linking
            let old_ids: HashSet<IdType> = self.map.keys().cloned().collect();

            // Update current cache with new entries and new relations
            for (id, doc) in &mut self.into_iter() {
                let changed = doc.update_unknown_references(|meta_id| {
                    id_doc_new.get(meta_id).is_some() || old_ids.contains(meta_id)
                });

                if changed != 0 {
                    last_updated_opt.as_mut().map(|last_updated| {
                        last_updated.insert(id.clone());
                        last_updated
                    });

                    on_rel_change(doc, changed);
                }
            }

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