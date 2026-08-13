# fraia-revision

`fraia-revision` is the clean-slate domain boundary for immutable Fraia design
history. It depends on `fraia-core` for the typed structural model vocabulary.
Higher layers, including `fraia-app-api` and `fraia-appd`, depend on this crate.
`fraia-core` must not depend on `fraia-revision`.

The crate owns canonical model serialization and SHA-256 snapshot identities,
immutable revisions and evidence, typed patch and diff contracts, conversation
graphs, working copies, deterministic analysis services, and SQLite persistence.
It keeps authored snapshots separate from derived analysis records and updates
conversation heads through explicit repository operations.
