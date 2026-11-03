# Craft Anything xtask

The `xtask` binary delegates `ico` to `craft-xtask` (`craft_xtask::ico::Config`,
`compute`, `apply`, `pack_ico`, and `cli_help`), configured with the repository
hicolor icon paths and `assets/app-icon/craft-anything.ico` while accepting
caller-supplied square PNG paths (1–256 px) and preserving silent stdout.

Craft Anything has no standalone `version.rs`, `stats.rs`, `layers.rs`, or
SHA-256 helper module. Craft Anything-specific tasks (`i18n_coverage`, `locks`,
`secrets`, `tools`, `schemas`, `pending`, `licenses`, `wasm`, `ci`) and the
inline `layers` (with AST/source rule checks) and `version` helpers in
`src/main.rs` remain local.

`tests/golden/` captures the original `ico` command output (`ico.hex` and empty
`ico.stdout`) on the pre-migration checkout (2026-11-03).
