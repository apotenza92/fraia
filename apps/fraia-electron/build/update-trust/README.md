# Fraia production TUF trust

`root.json` is the reviewed public trust anchor embedded in Windows and Linux
packages. It contains no private key material.

Reviewed `root.json` SHA-256:
`db88d4445135c02065824de9d035803bfc0b2b7a6eb0e5bb2fc57556e39d478e`.

The one-time key ceremony is implemented by
`scripts/create-tuf-production-trust.cjs`. It creates distinct Ed25519 keys for
the root, targets, snapshot, and timestamp roles. The public root is committed;
the private bundle must be stored through the approved `op` CLI workflow and
removed from disk immediately afterward.

Fraia currently uses threshold 1 for each role because it is operated by one
maintainer. The distinct offline root key limits routine release jobs to the
targets, snapshot, and timestamp roles. Any future root rotation must be signed
by the currently trusted root according to the TUF specification and reviewed
before packaging.

Never regenerate `root.json` during packaging, substitute a test root, or place
a production private key in this repository, a workflow artifact, or a log.
