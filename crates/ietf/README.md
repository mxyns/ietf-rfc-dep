# rfc-dep-ietf

Sub-crate for [rfc-dep-gui](/crates/gui)
Handles the querying and parsing of IETF Documents.

Lookup of documents is done using a HTTP GET to the IETF [DataTracker](https://github.com/ietf-tools/datatracker) API https://datatracker.ietf.org/api/v1/doc/document/{parameters}
A `Summary` per doc (rfc or draft only) is extracted from the result: It contains:
* `name`
* `url`
* `title`
* `revision`
* `is_rfc`

HTML Documents are scraped from https://datatracker.ietf.org/doc/{name}
Relations to other documents are parsed and stored in `rfc_dep_gui::IetfDoc::meta`, supported `rfc_dep_gui::Meta`s are:
* `Updates` (List),
* `UpdatedBy` (List),
* `Obsoletes` (List),
* `ObsoletedBy` (List),
* `AlsoKnownAs` (Item),
* `Replaces` (Item),
* `Was` (Item),
