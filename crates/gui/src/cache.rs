use rfc_dep_cache::{Cache, ResolveParams, ResolveTarget};
use rfc_dep_ietf::DocIdentifier;
use crate::app::RFCDepApp;
use crate::doc::{StatefulDoc, update_missing_dep_count};

pub(crate) type DocCache = Cache<DocIdentifier, StatefulDoc>;

impl RFCDepApp {

    pub(crate) fn merge_caches(&mut self, other: DocCache) {
        self.cache.merge_with(other);
        self.update_cache(None);
    }


    pub(crate) fn update_cache(&mut self, new_cache: Option<DocCache>) {
        // Check if import resolved some dependencies
        // Do not query new documents, use only the already provided
        // Max depth = 1
        if let Some(new_cache) = new_cache {
            self.cache = new_cache;
        }

        self.cache.resolve_dependencies(ResolveTarget::All, ResolveParams {
            print: true,
            depth: 1,
            query: false,
        }, update_missing_dep_count);
    }

}