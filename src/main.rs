mod cache;
mod doc;

use crate::cache::{DocCache};
use crate::doc::IetfDoc;

fn main() {
    let mut cache = DocCache::new();
    let doc = IetfDoc::from_url("https://datatracker.ietf.org/doc/rfc4271");
    let _cached_root = doc.resolve_dependencies(&mut cache, true);

    println!("{:#?}", cache);
}
