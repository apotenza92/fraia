# Fraia Knowledge Media Policy

_Status: active v0.1_
_Date: 2026-05-06_

This directory is for committed media used by compiled knowledge pages.

## Default rule

Prefer agent-created Fraia-native schematics over copied source figures.

Allowed by default:

- agent-created SVG/PNG diagrams made for Fraia
- open-license or public-domain media with attribution and license metadata

Staging-only by default:

- private textbook screenshots or page crops
- PDF contact sheets and OCR images
- website screenshots where licensing is unclear
- copied software-manual figures unless explicitly permitted
- third-party adapter image crops or screenshots unless license/permission is clear

Third-party ingestion outputs should normally inspire agent-created Fraia-native schematics, not contribute copied source artwork.

## Manifest requirement

Every committed media file must be listed in [`manifest.md`](manifest.md) with:

- file path
- source/status
- license or permission status
- source concepts/citations if derived from sources
- pages that use it

The linter checks local image links and requires media files to appear in the manifest where feasible.
