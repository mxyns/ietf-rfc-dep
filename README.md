# ietf-rfc-dep

Lookup IETF DataTracker documents, parse their relations (Updates, Obsoletes, etc.) and get documents' dependencies.
Keep track of which document you read too. 

## Main crate (GUI App)
[rfc-dep-gui](/crates/gui)

![rfc-dep-gui](/crates/gui/assets/rfc-dep-gui.png)

## Sub-crates 
* [rfc-dep-ietf](/crates/ietf): get documents and parse metadata
* [rfc-dep-cache](/crates/cache): store documents and resolve relations/dependencies