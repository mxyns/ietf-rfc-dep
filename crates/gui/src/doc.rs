use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::mem;
use std::ops::{Deref, DerefMut};

use rfc_dep_cache::{CacheReference, RelationalEntry, ResolvableEntry};
use rfc_dep_ietf::{DocIdentifier, IdContainer, IetfDoc, Meta};

#[derive(Clone, Debug, Serialize, Deserialize, Hash, PartialEq, Eq)]
/* Type Wrapper needed because CacheReference is from rfc_dep_cache
 * and IdContainer is from rfc_dep_doc */
pub struct DocReference(pub CacheReference<DocIdentifier>);

/* make DocReference an IdContainer to allow it to be contained in IetfDoc::Meta */
impl IdContainer for DocReference {
    type Holder<T> = DocReference;
}

impl From<DocIdentifier> for DocReference {
    fn from(value: DocIdentifier) -> Self {
        CacheReference::Unknown(value).into()
    }
}

impl From<DocReference> for CacheReference<DocIdentifier> {
    fn from(value: DocReference) -> Self {
        value.0
    }
}

impl From<CacheReference<DocIdentifier>> for DocReference {
    fn from(value: CacheReference<DocIdentifier>) -> Self {
        DocReference(value)
    }
}

impl Deref for DocReference {
    type Target = DocIdentifier;

    fn deref(&self) -> &Self::Target {
        self.0.deref()
    }
}

impl DocReference {
    fn get_mut_id(&mut self) -> &mut DocIdentifier {
        let DocReference(ref mut cache) = self;
        cache.get_mut()
    }

    fn get_mut(&mut self) -> &mut CacheReference<DocIdentifier> {
        let DocReference(ref mut cache) = self;
        cache
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub(crate) struct StatefulDoc {
    // Target document
    pub(crate) content: IetfDoc<DocReference>,

    // Real State
    pub(crate) is_read: bool,
    pub(crate) is_selected: bool,
    pub(crate) missing_dep_count: usize,
    pub(crate) offline: Option<String>,

    // Temporary State
    pub(crate) to_resolve: bool,
}

impl StatefulDoc {
    pub(crate) fn new(doc: IetfDoc<DocReference>) -> StatefulDoc {
        let mut doc = StatefulDoc {
            missing_dep_count: 0,
            content: doc,
            is_read: false,
            is_selected: false,
            to_resolve: false,
            offline: None,
        };

        doc.missing_dep_count = doc.get_unknown_relations_count();

        doc
    }

    pub(crate) fn download(&mut self) -> rfc_dep_ietf::error::Result<()> {

        if let Some(_) = self.offline {
            return Ok(())
        }

        self.offline = Some (
            self.content.download_raw()?
        );

        Ok(())
    }
}

impl ResolvableEntry<DocIdentifier> for StatefulDoc {
    fn get_value(id: DocIdentifier) -> Result<Self, String> {
        let doc = IetfDoc::from_name(id)?;
        Ok(StatefulDoc::new(doc))
    }
}

pub(crate) fn update_missing_dep_count(doc: &mut StatefulDoc, new_deps: isize) {
    doc.missing_dep_count = (doc.missing_dep_count as isize - new_deps) as usize;
}

// Implement resolve dependency algorithms when value is IetfDoc
impl RelationalEntry<DocIdentifier> for StatefulDoc {
    fn get_unknown_relations(&self) -> HashSet<DocIdentifier> {
        let mut to_update = HashSet::new();
        let mut add_unknown = |item: &CacheReference<DocIdentifier>| {
            match item {
                CacheReference::Unknown(id) => {
                    to_update.insert(id.clone());
                }
                CacheReference::Cached(_) => {}
            };
        };

        for (_, meta) in self.content.meta.deref() {
            match meta {
                Meta::Updates(list)
                | Meta::Obsoletes(list)
                | Meta::UpdatedBy(list)
                | Meta::ObsoletedBy(list) => {
                    for DocReference(item) in list {
                        add_unknown(item);
                    }
                }
                Meta::Replaces(DocReference(item)) | Meta::ReplacedBy(DocReference(item)) => {
                    add_unknown(item);
                }
                Meta::Was(_) | Meta::AlsoKnownAs(_) => {}
            }
        }

        to_update
    }

    fn update_unknown_references(&mut self, is_known: impl Fn(&DocIdentifier) -> bool) -> isize {
        let mut change = 0;

        let mut update_cache_ref = |cache_ref: &mut CacheReference<DocIdentifier>|
            *cache_ref = match cache_ref {
            CacheReference::Unknown(ref mut r) if is_known(r) => {
                change += 1;
                CacheReference::Cached(mem::take(r))
            }
            CacheReference::Cached(ref mut r) if !is_known(r) => {
                change += -1;
                CacheReference::Unknown(mem::take(r))
            }
            CacheReference::Unknown(ref mut r) => {
                CacheReference::Unknown(mem::take(r))
            }
            CacheReference::Cached(ref mut r) => {
                CacheReference::Cached(mem::take(r))
            }
        };

        for (_, meta) in self.content.meta.deref_mut() {
            match meta {
                Meta::Updates(set)
                | Meta::Obsoletes(set)
                | Meta::UpdatedBy(set)
                | Meta::ObsoletedBy(set) => {
                    let mut values = mem::take(set);
                    let _ = mem::replace(set, values.drain().map(|mut x| {
                        update_cache_ref(x.get_mut());
                        x
                    }).collect());
                }
                Meta::Replaces(DocReference(ref mut cache_ref))
                | Meta::ReplacedBy(DocReference(ref mut cache_ref)) => {
                    update_cache_ref(cache_ref);
                }
                Meta::Was(_) | Meta::AlsoKnownAs(_) => {}
            }
        }

        change
    }

    fn get_unknown_relations_count(&self) -> usize {
        let mut missing = 0;

        let count_meta = |cache_ref: &CacheReference<_>| match cache_ref {
            CacheReference::Unknown(_) => 1,
            CacheReference::Cached(_) => 0,
        };

        for (_, meta) in self.content.meta.deref() {
            match meta {
                Meta::Updates(set)
                | Meta::Obsoletes(set)
                | Meta::UpdatedBy(set)
                | Meta::ObsoletedBy(set) => {
                    for DocReference(item) in set {
                        missing += count_meta(item);
                    }
                }
                Meta::Replaces(DocReference(item)) | Meta::ReplacedBy(DocReference(item)) => {
                    missing += count_meta(item);
                }
                Meta::Was(_) | Meta::AlsoKnownAs(_) => {}
            }
        }

        missing
    }
}
