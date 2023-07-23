use std::cell::{RefCell};
use std::collections::HashMap;
use std::fmt::Debug;
use std::rc::Rc;
use crate::doc::{IetfDoc};

#[derive(Debug, Clone)]
pub struct DocCache {
    pub(crate) map: HashMap::<String, CachedDoc>
}

pub type CachedDoc = Rc<RefCell<IetfDoc>>;

impl DocCache {
    pub fn new() -> DocCache {
        DocCache {
            map: HashMap::new()
        }
    }

    pub fn put_doc(&mut self, doc: IetfDoc) -> CachedDoc {
        let name = doc.name.clone();
        let doc = Rc::new(RefCell::new(doc));
        self.map.insert(name, doc.clone());

        doc
    }

    pub fn get(&self, name: &String) -> Option<CachedDoc> {

        let doc = self.map.get(name);
        if doc.is_some() { // Doc not in cache
            return Some(doc.unwrap().clone());
        }

        return None
    }
}