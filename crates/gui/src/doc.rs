use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::mem;

use rfc_dep_cache::{CacheReference, RelationalEntry, ResolvableEntry};
use rfc_dep_ietf::{name_to_id, DocIdentifier, IdContainer, IetfDoc, Meta};

#[derive(Clone, Debug, Serialize, Deserialize)]
/* Type Wrapper needed because CacheReference is from rfc_dep_cache
 * and IdContainer is from rfc_dep_doc */
pub struct DocReference(pub CacheReference<DocIdentifier>);

/* make DocReference an IdContainer to allow it to be contained in IetfDoc::Meta */
impl IdContainer for DocReference {
    type Holder<T> = DocReference;

    fn from_inner_text(lines: Vec<&str>) -> Vec<Self::Holder<DocIdentifier>> {
        lines
            .into_iter()
            .skip(1)
            .step_by(2)
            .map(|x| CacheReference::Unknown(name_to_id(x)).into())
            .collect()
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

#[derive(Clone, Debug, Serialize, Deserialize)]
pub(crate) struct StatefulDoc {
    // Target document
    pub(crate) content: IetfDoc<DocReference>,

    // Real State
    pub(crate) is_read: bool,
    pub(crate) is_selected: bool,
    pub(crate) missing_dep_count: usize,

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
        };

        doc.missing_dep_count = doc.get_unknown_relations_count();

        doc
    }
}

impl ResolvableEntry<DocIdentifier> for StatefulDoc {
    fn get_value(id: DocIdentifier) -> Self {
        StatefulDoc::new(IetfDoc::from_url(format!(
            "https://datatracker.ietf.org/doc/{}",
            id
        )))
    }
}

pub(crate) fn update_missing_dep_count(doc: &mut StatefulDoc, new_deps: isize) {
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
                    for DocReference(item) in list {
                        match item {
                            CacheReference::Unknown(id) => {
                                to_update.insert(id.clone());
                            }
                            CacheReference::Cached(_) => {}
                        };
                    }
                }
                Meta::Was(_) | Meta::Replaces(_) => {}
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
                    for DocReference(ref mut cache_ref) in list {
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
                        }
                    }
                }
                Meta::Was(_) | Meta::Replaces(_) => {}
            }
        }

        change
    }

    fn get_unknown_relations_count(&self) -> usize {
        let mut missing = 0;
        for meta in &self.content.meta {
            match meta {
                Meta::Updates(list)
                | Meta::Obsoletes(list)
                | Meta::UpdatedBy(list)
                | Meta::ObsoletedBy(list) => {
                    for DocReference(item) in list {
                        match item {
                            CacheReference::Unknown(_) => {
                                missing += 1;
                            }
                            CacheReference::Cached(_) => {}
                        };
                    }
                }
                Meta::Was(_) | Meta::Replaces(_) => {}
            }
        }

        missing
    }
}
