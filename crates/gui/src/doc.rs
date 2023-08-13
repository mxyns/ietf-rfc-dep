use std::collections::HashSet;
use rfc_dep_cache::{CacheReference, RelationalEntry, ResolvableEntry};
use rfc_dep_ietf::{DocIdentifier, IdContainer, IetfDoc, Meta, name_to_id};
use serde::{Serialize, Deserialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
/* Type Wrapper needed because CacheReference is from rfc_dep_cache
 * and IdContainer is from rfc_dep_doc
 */
pub struct DocReference(pub CacheReference<DocIdentifier>);

/* make DocReference an IdContainer to allow it to be contained in IetfDoc::Meta */
impl IdContainer for DocReference {
    type Holder<T> = DocReference;

    fn from_inner_text(lines: Vec<&str>) -> Vec<Self::Holder<DocIdentifier>> {
        lines.into_iter().skip(1).step_by(2).map(|x| {
            DocReference(CacheReference::Unknown(name_to_id(x)))
        }).collect()
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
        StatefulDoc::new(IetfDoc::from_url(format!("https://datatracker.ietf.org/doc/{}", id)))
    }
}

pub(crate) fn update_missing_dep_count(doc: &mut StatefulDoc, new_deps: isize) {
    doc.missing_dep_count = (doc.missing_dep_count.clone() as isize - new_deps) as usize;
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
                            DocReference(CacheReference::Unknown(id)) => {
                                to_update.insert(id.clone());
                            }
                            DocReference(CacheReference::Cached(_)) => {}
                        };
                    };
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
                    for item in list {
                        let (DocReference(CacheReference::Cached(ref_id))
                        | DocReference(CacheReference::Unknown(ref_id))) = item.clone();
                        let is_known = is_known(&ref_id);

                        // was unknown
                        if let DocReference(CacheReference::Unknown(_)) = item {
                            if is_known {
                                change += 1;
                                *item = DocReference(CacheReference::Cached(ref_id));
                            }
                        } else { // was known
                            if !is_known {
                                change -= 1;
                                *item = DocReference(CacheReference::Unknown(ref_id));
                            }
                        }
                    };
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
                    for item in list {
                        match item {
                            DocReference(CacheReference::Unknown(_)) => {
                                missing += 1;
                            }
                            DocReference(CacheReference::Cached(_)) => {}
                        };
                    };
                }
                Meta::Was(_) | Meta::Replaces(_) => {}
            }
        };

        missing
    }
}
