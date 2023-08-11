use std::collections::HashSet;
use rfc_dep_cache::{CacheReference, RelationalEntry, ResolvableEntry};
use rfc_dep_ietf::{DocIdentifier, IetfDoc, Meta};
use serde::{Serialize, Deserialize};

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub(crate) struct StatefulDoc {
    // Target document
    pub(crate) content: IetfDoc,

    // Real State
    pub(crate) is_read: bool,
    pub(crate) is_selected: bool,
    pub(crate) missing_dep_count: usize,

    // Temporary State
    pub(crate) to_resolve: bool,
}

impl StatefulDoc {
    pub(crate) fn new(doc: IetfDoc) -> StatefulDoc {
        StatefulDoc {
            missing_dep_count: doc.missing(),
            content: doc,
            is_read: false,
            is_selected: false,
            to_resolve: false,
        }
    }
}

impl ResolvableEntry<DocIdentifier> for StatefulDoc {
    fn get_value(id: DocIdentifier) -> Self {
        StatefulDoc::new(IetfDoc::from_url(format!("https://datatracker.ietf.org/doc/{}", id)))
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
                    for item in list {
                        match item {
                            CacheReference::Unknown(id) => {
                                to_update.insert(id.clone());
                            }
                            CacheReference::Cached(_) => {}
                        };
                    };
                }
                Meta::Was(_) => {}
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
                    for item in list {
                        let (CacheReference::Cached(ref_id) | CacheReference::Unknown(ref_id)) = item.clone();
                        let is_known = is_known(&ref_id);

                        // was unknown
                        if let CacheReference::Unknown(_) = item {
                            if is_known {
                                change += 1;
                                *item = CacheReference::Cached(ref_id);
                            }
                        } else { // was known
                            if !is_known {
                                change -= 1;
                                *item = CacheReference::Unknown(ref_id);
                            }
                        }
                    };
                }
                Meta::Was(_) => {}
            }
        }

        change
    }
}
