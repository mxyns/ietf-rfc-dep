use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fmt;
use std::fmt::Debug;
use std::hash::Hash;

pub trait CacheIdentifier: Eq + Hash + Ord {}

impl<T> CacheIdentifier for T where T: Eq + Hash + Ord {}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Cache<IdType: CacheIdentifier, ValueType> {
    pub(crate) map: BTreeMap<IdType, ValueType>,
}

impl<IdType: CacheIdentifier, ValueType> Default for Cache<IdType, ValueType> {
    fn default() -> Self {
        Cache {
            map: BTreeMap::default(),
        }
    }
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
            CacheReference::Unknown(id) => {
                write!(f, "Unknown(\"{}\")", id)
            }
            CacheReference::Cached(id) => {
                write!(f, "Cached(\"{}\")", id)
            }
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
    /* returns number of cache entries */
    pub fn is_empty(&self) -> bool {
        self.map.is_empty()
    }

    /* remove entry */
    pub fn remove(&mut self, id: &IdType) -> Option<ValueType> {
        self.map.remove(id)
    }
}

/* allow to into_iter on cache reference */
impl<'h, IdType: CacheIdentifier, ValueType> IntoIterator for &'h Cache<IdType, ValueType> {
    type Item = <&'h BTreeMap<IdType, ValueType> as IntoIterator>::Item;
    type IntoIter = <&'h BTreeMap<IdType, ValueType> as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.map.iter()
    }
}

/* allow to into_iter on cache mut reference */
impl<'h, IdType: CacheIdentifier, ValueType> IntoIterator for &'h mut Cache<IdType, ValueType> {
    type Item = <&'h mut BTreeMap<IdType, ValueType> as IntoIterator>::Item;
    type IntoIter = <&'h mut BTreeMap<IdType, ValueType> as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.map.iter_mut()
    }
}
