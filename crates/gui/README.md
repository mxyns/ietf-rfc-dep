# rfc-dep-gui

Main crate of the project [rfc-dep-gui](/crates/gui)
Uses both [rfc-dep-ietf](/crates/ietf) and [rfc-dep-cache](/crates/cache).

## Use

Allows looking up IETF DataTracker documents, finding their dependencies (when known) and keeping track of which document you've read.

You can also Save, Open or Merge (using File -> Import) projects.

## Screenshot
![rfc-dep-gui screenshot](/crates/gui/assets/rfc-dep-gui.png)

## Instructions
Requires `libgtk-3-dev` on Ubuntu

Nothing special: 
* run with `cargo run`
* build executable `cargo build --release`

## TODO
[~] Vec<Meta> => struct(Meta::*::(_)) [waiting for variant-map to impl IntoIter on StructMap]
[] allow downloading files for offline reading (airplane mode)
[] real tabs
[] reduce .clone use on IdType
[] add cli
[] graph gui
[x] better error handling (eg don't panic on req timed-out)
