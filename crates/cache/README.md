# rfc-dep-cache

Sub-crate for [rfc-dep-gui](/crates/gui)
Defines a cache for any type.

Has options for `RelationalEntry`s (entries having relations to others by holding their Id).
Can resolve dependencies recursively (with a maximum depth) between entries, if they are also `ResolvableEntry` (can be retrieved only based on their Id)

Resolving dependencies/relations between entries uses [rayon](https://crates.io/crates/rayon) to query the values of ResolvableEntries in parallel.