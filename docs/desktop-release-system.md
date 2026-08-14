# Desktop Release System

This document is the maintained operating procedure for Fraia desktop releases.
The workflow files remain the executable contract.

## Normal release sequence

1. Merge the intended release changes into `main`.
2. Set the same release version in the Rust workspace and Electron package.
3. Add authoritative release notes for that version to `CHANGELOG.md`.
4. Dispatch **macOS release candidate** on `main` while final checks continue.
5. Wait for the candidate workflow to succeed.
6. Push the owner-authorized release tag from the exact candidate commit:
   - Stable: `vX.Y.Z`
   - Beta: `vX.Y.Z-beta.N`
7. Wait for **Desktop release** to complete and verify the public release, updater feeds, provenance, and Homebrew publication.

Do not create a real tag to test release automation. Dispatch **Desktop release**
manually with simulation versions instead. A simulation builds and tests native
Windows and Linux packages without signing or publication.

## macOS release candidates

`.github/workflows/macos-release-candidate.yml` prepares macOS work before the
release decision. It builds both Fraia identities for ARM64 and Intel in
parallel. Every package must pass:

- Developer ID signing;
- application and disk-image notarization;
- stapling and Gatekeeper assessment;
- packaged application launch;
- packaged CalculiX solver execution; and
- final artifact and update-metadata assembly.

The workflow then seals the complete candidate file set in a SHA-256 manifest.
The manifest records the exact repository, commit, version, channels, and native
targets. GitHub stores the packages as private Actions artifacts for 14 days.
The candidate workflow does not create a tag, release, public asset, updater
feed, or Homebrew change.

## Candidate reuse at tag time

`.github/workflows/release.yml` searches for a successful, unexpired candidate
from the exact tagged commit. The release imports a candidate only when all of
these values match:

- repository;
- commit SHA;
- application version;
- required Stable and Beta identities for the release policy;
- ARM64 and Intel targets;
- exact file set; and
- every recorded SHA-256 digest.

The release then runs the normal N-1 updater replacement tests and all remaining
publication gates. Candidate reuse removes repeated Apple notarization from the
tag-to-publication critical path. It does not bypass updater, provenance,
Homebrew, public-release, or feed verification.

If no exact candidate exists, if an artifact expired, or if any check differs,
the release automatically builds, signs, notarizes, and verifies fresh macOS
packages. Never weaken the matching rules to force reuse.

## Channel behavior

A Beta tag builds the Beta identity only. A Stable tag always builds Stable. If
the final Stable version is newer than the version advertised to Beta users, the
same Stable tag also publishes a separate Fraia Beta identity package. This lets
Beta users receive the latest final version when no newer Beta exists without
mixing application identities or user-data directories.

## Timing expectations

Apple notarization is external and variable. The first complete four-package
candidate run on 13 August 2026 took about 33 minutes. The earlier sequential
release path took about 74 minutes for macOS packaging. Prepare the candidate
before the release decision so this variable delay does not occur after the tag
is pushed.

Treat timings as observations, not guarantees. Use the current GitHub Actions
run as the source of truth for each release.
