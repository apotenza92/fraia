#!/usr/bin/env python3
"""Deterministic lint for docs/knowledge."""
from __future__ import annotations

import re
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
KNOWLEDGE = ROOT / "docs" / "knowledge"
INDEX = KNOWLEDGE / "index.md"
TOPIC_MAP = KNOWLEDGE / "topic-map.md"
WIKI = KNOWLEDGE / "wiki"
REQUIRED = {
    "title", "status", "trust_level", "domain", "applies_to", "not_applicable_to",
    "jurisdiction_or_standard_context", "last_compiled", "source_count", "citation_policy", "owner",
}
REQUIRED_SECTIONS = [
    "## Summary", "## Scope / non-scope", "## Key concepts",
    "## Engineering guidance for Fraia agents", "## Tradeoffs / cautions",
    "## Source-backed claims", "## Open questions / weak evidence",
    "## Related pages", "## Sources",
]
ALLOWED_STATUS = {"draft", "compiled", "needs-review", "deprecated"}
ALLOWED_TRUST = {"raw", "compiled", "reviewed"}
ALLOWED_CITATION = {"required", "none"}
ALLOWED_DOMAIN = {"structural-steel", "steel", "loads", "analysis", "modeling", "stability", "diagnostics", "materials", "systems", "product", "other"}
LINK_RE = re.compile(r"(?<!!)\[[^\]]+\]\(([^)]+)\)")
IMAGE_RE = re.compile(r"!\[[^\]]*\]\(([^)]+)\)")
SOURCE_ENTRY_RE = re.compile(r"^- \[S\d+\].+$", re.MULTILINE)
FORBIDDEN_KNOWLEDGE_PARTS = {".staging", ".cache"}
FORBIDDEN_KNOWLEDGE_PREFIXES = {"media/private/"}
MEDIA = KNOWLEDGE / "media"
MEDIA_MANIFEST = MEDIA / "manifest.md"


def split_front_matter(text: str):
    if not text.startswith("---\n"):
        return None, text
    end = text.find("\n---\n", 4)
    if end == -1:
        return None, text
    return text[4:end], text[end + 5 :]


def front_keys(front: str) -> set[str]:
    return {line.split(":", 1)[0].strip() for line in front.splitlines() if line and not line.startswith(" ") and ":" in line}


def scalar(front: str, key: str) -> str | None:
    for line in front.splitlines():
        if line.startswith(f"{key}:"):
            return line.split(":", 1)[1].strip()
    return None


def is_external(target: str) -> bool:
    return target.startswith(("http://", "https://", "mailto:", "#"))


def resolve_local(path: Path, target: str) -> Path | None:
    target = target.split("#", 1)[0].strip()
    if not target or is_external(target):
        return None
    return (path.parent / target).resolve()


def check_links(path: Path, text: str, errors: list[str]) -> None:
    for match in LINK_RE.finditer(text):
        target = match.group(1)
        resolved = resolve_local(path, target)
        if resolved is None:
            continue
        try:
            resolved.relative_to(ROOT.resolve())
        except ValueError:
            errors.append(f"{path}: local link escapes repo: {target}")
            continue
        if not resolved.exists():
            errors.append(f"{path}: broken local link: {target}")


def check_images(path: Path, text: str, media_manifest: str, errors: list[str]) -> None:
    for match in IMAGE_RE.finditer(text):
        target = match.group(1)
        resolved = resolve_local(path, target)
        if resolved is None:
            continue
        try:
            rel_repo = resolved.relative_to(ROOT.resolve()).as_posix()
        except ValueError:
            errors.append(f"{path}: local image escapes repo: {target}")
            continue
        if not resolved.exists():
            errors.append(f"{path}: broken local image link: {target}")
            continue
        try:
            rel_media = resolved.relative_to(MEDIA.resolve()).as_posix()
        except ValueError:
            continue
        if resolved.suffix.lower() in {".md", ".txt"}:
            continue
        if rel_media not in media_manifest and f"media/{rel_media}" not in media_manifest:
            errors.append(f"{path}: media file not listed in docs/knowledge/media/manifest.md: {rel_repo}")


def git_tracked_knowledge_files() -> set[Path]:
    """Return git-tracked files under docs/knowledge.

    Staging/cache directories may legitimately contain local temporary files during
    a wiki-maintenance run. Lint should reject only committed/tracked leaks, not
    normal ignored local working files.
    """
    try:
        result = subprocess.run(
            ["git", "ls-files", "docs/knowledge"],
            cwd=ROOT,
            check=True,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            text=True,
        )
    except (OSError, subprocess.CalledProcessError):
        return set()
    return {ROOT / line for line in result.stdout.splitlines() if line.strip()}


def check_forbidden_tracked_paths(path: Path, errors: list[str]) -> None:
    rel = path.relative_to(KNOWLEDGE).as_posix()
    parts = set(Path(rel).parts)
    if parts & FORBIDDEN_KNOWLEDGE_PARTS:
        errors.append(f"{path}: forbidden tracked knowledge staging/cache path")
    if any(rel.startswith(prefix) for prefix in FORBIDDEN_KNOWLEDGE_PREFIXES):
        errors.append(f"{path}: forbidden tracked private media path")


def check_tracked_media_manifest(path: Path, media_manifest: str, errors: list[str]) -> None:
    try:
        rel_media = path.relative_to(MEDIA).as_posix()
    except ValueError:
        return
    if path.name in {"README.md", "manifest.md"} or path.suffix.lower() in {".md", ".txt"}:
        return
    if rel_media not in media_manifest and f"media/{rel_media}" not in media_manifest:
        errors.append(f"{path}: tracked media file not listed in docs/knowledge/media/manifest.md")


def source_entries(body: str) -> list[str]:
    if "## Sources" not in body:
        return []
    sources = body.split("## Sources", 1)[1]
    next_section = re.search(r"\n## ", sources)
    if next_section:
        sources = sources[: next_section.start()]
    return SOURCE_ENTRY_RE.findall(sources)


def check_compiled(path: Path, front: str, body: str, index_text: str, topic_text: str, errors: list[str]) -> None:
    missing = REQUIRED - front_keys(front)
    if missing:
        errors.append(f"{path}: missing required front matter: {', '.join(sorted(missing))}")
    status = scalar(front, "status")
    trust = scalar(front, "trust_level")
    domain = scalar(front, "domain")
    citation = scalar(front, "citation_policy")
    if status not in ALLOWED_STATUS:
        errors.append(f"{path}: invalid status `{status}`")
    if trust not in ALLOWED_TRUST:
        errors.append(f"{path}: invalid/disallowed trust_level `{trust}`")
    if domain not in ALLOWED_DOMAIN:
        errors.append(f"{path}: invalid domain `{domain}`")
    if citation not in ALLOWED_CITATION:
        errors.append(f"{path}: invalid citation_policy `{citation}`")
    if trust == "reviewed" and status != "compiled":
        errors.append(f"{path}: trust_level reviewed requires status compiled")
    if citation != "required":
        errors.append(f"{path}: compiled pages must use citation_policy: required")
    for section in REQUIRED_SECTIONS:
        if section not in body:
            errors.append(f"{path}: compiled page missing required section {section}")
    try:
        count = int(scalar(front, "source_count") or "0")
    except ValueError:
        count = -1
    entries = source_entries(body)
    if count <= 0:
        errors.append(f"{path}: compiled page source_count must be > 0")
    if count != len(entries):
        errors.append(f"{path}: source_count {count} does not equal [S#] entries {len(entries)}")
    for entry in entries:
        if not any(label in entry for label in ("URL:", "Path:", "Local source:")):
            errors.append(f"{path}: source entry missing URL:, Path:, or Local source:: {entry[:80]}")
        if not any(label in entry for label in ("Retrieved:", "Consulted:")):
            errors.append(f"{path}: source entry missing Retrieved: or Consulted:: {entry[:80]}")
        if "Source type:" not in entry:
            errors.append(f"{path}: source entry missing Source type:: {entry[:80]}")
        if "Reliability/limits:" not in entry:
            errors.append(f"{path}: source entry missing Reliability/limits:: {entry[:80]}")
    rel = str(path.relative_to(KNOWLEDGE))
    if rel not in index_text:
        errors.append(f"{path}: compiled page not listed in docs/knowledge/index.md")
    if TOPIC_MAP.exists() and rel not in topic_text:
        errors.append(f"{path}: compiled page not listed in docs/knowledge/topic-map.md")


def main() -> int:
    errors: list[str] = []
    if not KNOWLEDGE.exists():
        print("docs/knowledge does not exist", file=sys.stderr)
        return 1
    index_text = INDEX.read_text() if INDEX.exists() else ""
    topic_text = TOPIC_MAP.read_text() if TOPIC_MAP.exists() else ""
    media_manifest = MEDIA_MANIFEST.read_text() if MEDIA_MANIFEST.exists() else ""
    titles: dict[str, Path] = {}
    slugs: dict[str, Path] = {}

    tracked_files = git_tracked_knowledge_files()
    for path in sorted(tracked_files):
        if path.exists() and path.is_file():
            check_forbidden_tracked_paths(path, errors)
            check_tracked_media_manifest(path, media_manifest, errors)

    for path in sorted(KNOWLEDGE.rglob("*.md")):
        text = path.read_text()
        check_links(path, text, errors)
        check_images(path, text, media_manifest, errors)
        if not path.is_relative_to(WIKI) or path.name in {"README.md", "log.md"}:
            continue
        front, body = split_front_matter(text)
        if front is None:
            errors.append(f"{path}: missing YAML front matter")
            continue
        title = scalar(front, "title")
        if title:
            if title in titles:
                errors.append(f"duplicate title `{title}`: {titles[title]} and {path}")
            titles[title] = path
        slug = str(path.relative_to(WIKI).with_suffix(""))
        if slug in slugs:
            errors.append(f"duplicate slug `{slug}`: {slugs[slug]} and {path}")
        slugs[slug] = path
        if scalar(front, "trust_level") == "canonical":
            errors.append(f"{path}: trust_level canonical is disallowed")
        if scalar(front, "status") == "compiled":
            check_compiled(path, front, body, index_text, topic_text, errors)

    if errors:
        for error in errors:
            print(error, file=sys.stderr)
        return 1
    print("Knowledge wiki lint passed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
