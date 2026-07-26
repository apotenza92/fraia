# Knowledge-Next Retrieval Evals

These eval seeds define the first retrieval expectations for the typed knowledge store.

Each eval case is source-backed indirectly through its expected cards. The eval prompt is not a source of truth; it is a scenario that should retrieve the right card set and avoid unsafe answer patterns.

Generated or runtime eval implementations may transform these records, but should not weaken the expected card ids without also updating the cards or recording a deferral.
