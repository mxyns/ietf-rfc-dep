use crate::app::RFCDepApp;
use crate::doc::{update_missing_dep_count, StatefulDoc};
use rfc_dep_cache::{Cache, ResolveParams, ResolveTarget};
use rfc_dep_ietf::DocIdentifier;
use std::time::Duration;
use std::{mem, thread};

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

        self.cache.resolve_dependencies(
            ResolveTarget::All,
            ResolveParams {
                print: true,
                depth: 1,
                query: false,
            },
            update_missing_dep_count,
        );
    }

    pub(crate) fn is_resolving(&self) -> bool {
        self.resolve_handle.is_some() && !self.resolve_handle.as_ref().unwrap().is_finished()
    }

    pub(crate) fn task_resolve_dependencies(
        &mut self,
        target: ResolveTarget<DocIdentifier>,
        params: ResolveParams,
    ) {
        if self.resolve_handle.is_some() {
            self.toasts
                .error("Resolve already pending")
                .set_duration(Some(Duration::from_secs(5)));
            return;
        }

        self.toasts
            .info("Resolving...")
            .set_duration(Some(Duration::from_secs(5)));
        let cache = mem::take(&mut self.cache);
        self.resolve_handle = Some(thread::spawn(move || {
            let mut cache = cache;
            cache.resolve_dependencies(target, params, update_missing_dep_count);
            cache
        }));
    }

    pub(crate) fn check_resolve_result(&mut self) {
        if self.is_resolving() || self.resolve_handle.is_none() {
            return;
        }
        let handle = self.resolve_handle.take();

        let cache = handle.unwrap().join().unwrap();
        self.cache.merge_with(cache);
        self.toasts
            .success("Resolve completed!")
            .set_duration(Some(Duration::from_secs(5)));
    }
}
