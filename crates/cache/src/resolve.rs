/* represents an entry containing references to other entries */
use crate::{Cache, CacheIdentifier};
use rayon::prelude::*;
use std::collections::HashSet;
use std::fmt;
use std::fmt::Debug;

pub trait RelationalEntry<IdType> {
    // must return all keys of relations still not known in cache (CacheReference::Unknown)
    fn get_unknown_relations(&self) -> HashSet<IdType>;

    // must update all unknown relations of the entry
    // uses the callback 'is_known' to determine from within 'update_reference'
    // if an id is now known in the calling context
    // returns the number of new references
    fn update_unknown_references(&mut self, is_known: impl Fn(&IdType) -> bool) -> isize;

    // give the self.get_unknown_relations().len() without computing it
    fn get_unknown_relations_count(&self) -> usize;
}

/* represents an entry which value can be retrieved using only its id
 *   (eg: a document using an url, a file from its path, etc.)
 */
pub trait ResolvableEntry<IdType>: Sized {
    // query the entry's value from its id
    fn get_value(id: IdType) -> Result<Self, String>;
}

impl<IdType, ValueType> Cache<IdType, ValueType>
where
    IdType: CacheIdentifier + Sync + Clone + fmt::Display + Debug,
    ValueType: ResolvableEntry<IdType> + Send + Clone + Debug,
{
    /* query values from ids and cache_core the queried values */
    fn query_values(&mut self, ids: impl IntoIterator<Item = IdType>) -> HashSet<IdType> {
        let new_ids: HashSet<IdType> = ids.into_iter().filter(|id| !self.has_id(id)).collect();

        let values: Vec<_> = new_ids
            .par_iter()
            .map(|id| (id, ValueType::get_value(id.clone())))
            .collect();

        for (id, value) in values {
            if let Ok(value) = value {
                self.cache(id.clone(), value);
            } else {
                println!("Could not query {}", value.err().unwrap())
            }
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
    ValueType: RelationalEntry<IdType> + Send + ResolvableEntry<IdType> + Clone + Debug,
{
    pub fn resolve_dependencies<F>(
        &mut self,
        target: ResolveTarget<IdType>,
        params: ResolveParams,
        mut on_rel_change: F,
    ) where
        F: FnMut(&mut ValueType, isize),
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
            ResolveTarget::All => None,
            ResolveTarget::Single(root) => Some(HashSet::from([root])),
            ResolveTarget::Multiple(roots) => Some(HashSet::from_iter(roots)),
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

            if to_update.is_empty() {
                if print {
                    println!("early stop, no new entries found");
                }
                break;
            }

            // Query uncached documents
            let id_doc_new = if query {
                self.query_values(to_update)
            } else {
                HashSet::<IdType>::new()
            };

            self.update_relations(
                |meta_id| id_doc_new.get(meta_id).is_some(),
                |id, value, change| {
                    last_updated_opt.as_mut().map(|last_updated| {
                        last_updated.insert(id.clone());
                        last_updated
                    });

                    on_rel_change(value, change);
                },
            );

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

    // id_doc_new.get(meta_id).is_some()
    pub fn update_relations<OnChangeFn, IsKnownFn>(
        &mut self,
        is_known: IsKnownFn,
        mut on_change: OnChangeFn,
    ) where
        IsKnownFn: Fn(&IdType) -> bool,
        OnChangeFn: FnMut(&IdType, &mut ValueType, isize),
    {
        // Copy cache to lookup already existing entries when linking
        let old_ids: HashSet<IdType> = self.map.keys().cloned().collect();

        // Update current cache with new entries and new relations
        for (id, doc) in &mut self.into_iter() {
            let changed = doc.update_unknown_references(|meta_id| {
                old_ids.contains(meta_id) || is_known(meta_id)
            });

            if changed != 0 {
                on_change(id, doc, changed);
            }
        }
    }
}
