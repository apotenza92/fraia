#!/usr/bin/env python3
"""Build a normalized source inventory for the Fraia knowledge rebuild."""
from __future__ import annotations

import argparse
import hashlib
import json
import re
from collections import defaultdict
from datetime import date
from pathlib import Path
from typing import Any

ROOT = Path(__file__).resolve().parents[1]
KNOWLEDGE = ROOT / "docs" / "knowledge"
WIKI = KNOWLEDGE / "wiki"
RAW = KNOWLEDGE / "raw"
KNOWLEDGE_NEXT = ROOT / "docs" / "knowledge-next"
BANNED_KNOWLEDGE_NEXT_SUFFIXES = {
    ".gif", ".jpeg", ".jpg", ".pdf", ".png", ".tif", ".tiff", ".webp",
}

SOURCE_LINE_RE = re.compile(r"^- \[(S\d+)\]\s*(.+)$")
URL_RE = re.compile(r"\bURL:\s*(https?://\S+|[^.;\s]+)", re.IGNORECASE)
DOI_RE = re.compile(r"\bDOI:\s*([^.;\s]+)", re.IGNORECASE)
PATH_RE = re.compile(
    r"\bPath:\s*`?(.+?)`?(?=\.?\s+(?:Source type|Retrieved|Consulted|Reliability/limits|Reliability|$))",
    re.IGNORECASE,
)
LOCAL_RE = re.compile(
    r"\bLocal source:\s*`?(.+?)`?(?=\.?\s+(?:Source type|Retrieved|Consulted|Reliability/limits|Reliability|$))",
    re.IGNORECASE,
)
RETRIEVED_RE = re.compile(r"\bRetrieved:\s*([0-9]{4}-[0-9]{2}-[0-9]{2}|YYYY-MM-DD)", re.IGNORECASE)
CONSULTED_RE = re.compile(r"\bConsulted:\s*([0-9]{4}-[0-9]{2}-[0-9]{2}|YYYY-MM-DD)", re.IGNORECASE)
SOURCE_TYPE_RE = re.compile(r"\bSource type:\s*([^.;\n]+)", re.IGNORECASE)
RELIABILITY_RE = re.compile(r"\bReliability/limits:\s*([^\n]+)", re.IGNORECASE)
PAGES_USED_RE = re.compile(r"\bPages used:\s*([^\n]+)", re.IGNORECASE)
TITLE_RE = re.compile(r"\*([^*]+)\*")
RAW_URL_LINE_RE = re.compile(r"https?://[^\s;)]+")
RAW_SOURCE_HEADING_RE = re.compile(r"^##\s+(Sources|Source list|Sources retained)\b", re.IGNORECASE)
MARKDOWN_HEADING_RE = re.compile(r"^##\s+")
RAW_BLOCK_HEADING_RE = re.compile(r"^###\s+(.+)")
REGISTRY_ID_RE = re.compile(r"^- Source id:\s*(.+)$")


def repo_rel(path: Path) -> str:
    return path.relative_to(ROOT).as_posix()


def read_text(path: Path) -> str:
    return path.read_text(encoding="utf-8")


def source_section(text: str) -> str:
    if "## Sources" not in text:
        return ""
    section = text.split("## Sources", 1)[1]
    next_section = re.search(r"\n## ", section)
    if next_section:
        section = section[: next_section.start()]
    return section


def extract_labeled(match_re: re.Pattern[str], text: str) -> str | None:
    match = match_re.search(text)
    if not match:
        return None
    return match.group(1).strip().strip("`").rstrip(".")


def clean_url(url: str | None) -> str | None:
    if not url:
        return None
    return url.strip().rstrip(".,);")


def title_from_entry(entry: str) -> str | None:
    match = TITLE_RE.search(entry)
    if match:
        return match.group(1).strip()
    before_locator = re.split(r"\b(?:URL|DOI|Path|Local source|Source type):", entry, maxsplit=1)[0]
    before_locator = re.sub(r"^Fraia compiled wiki,\s*", "", before_locator).strip(" .")
    return before_locator or None


def author_from_entry(entry: str, title: str | None) -> str | None:
    if not title:
        return None
    prefix = entry.split(f"*{title}*", 1)[0].strip(" ,.")
    return prefix or None


def classify_source(source_type: str | None, locator: str | None, raw: str) -> str:
    text = f"{source_type or ''} {locator or ''} {raw}".lower()
    if "fraia compiled wiki" in text or "docs/knowledge/wiki" in text or "docs/" in (locator or "").lower():
        return "internal_fraia"
    if any(term in text for term in ["software", "manual", "solver", "pynite", "scia", "lusas", "anastruct", "saf", "strand7"]):
        return "software_manual"
    if "local source" in raw.lower() or "private" in text or "textbook reference" in text:
        return "textbook_private_reference"
    if any(term in text for term in ["discovery", "wikipedia"]):
        return "discovery_only"
    if any(term in text for term in ["seo", "marketing", "anonymous", "weak", "untraceable"]):
        return "weak_replace"
    if locator and any(term in text for term in [
        "professional", "academic", "university", "public agency", "government", "open educational",
        "textbook", "standard", "official", "peer-reviewed", "steel industry", "design guide",
        "research report", "conference", "chapter", "institution", "public",
    ]):
        return "public_professional"
    if locator:
        return "public_professional"
    return "weak_replace"


def inventory_key(entry: dict[str, Any]) -> str:
    for key in ["url", "doi", "path", "local_source"]:
        value = entry.get(key)
        if value:
            return f"{key}:{value.lower()}"
    if entry.get("title"):
        return f"title:{entry['title'].lower()}"
    return f"raw:{entry['raw_entry'].lower()[:160]}"


def source_id_for(key: str) -> str:
    digest = hashlib.sha1(key.encode("utf-8")).hexdigest()[:10]
    return f"SRC-{digest}"


def parse_source_entry(page: Path, marker: str, entry: str) -> dict[str, Any]:
    url = clean_url(extract_labeled(URL_RE, entry))
    doi = extract_labeled(DOI_RE, entry)
    path = extract_labeled(PATH_RE, entry)
    local_source = extract_labeled(LOCAL_RE, entry)
    retrieved = extract_labeled(RETRIEVED_RE, entry)
    consulted = extract_labeled(CONSULTED_RE, entry)
    source_type = extract_labeled(SOURCE_TYPE_RE, entry)
    reliability = extract_labeled(RELIABILITY_RE, entry)
    pages_used = extract_labeled(PAGES_USED_RE, entry)
    title = title_from_entry(entry)
    author = author_from_entry(entry, title)
    locator = url or doi or path or local_source
    bucket = classify_source(source_type, locator, entry)
    return {
        "page_source_marker": marker,
        "title": title,
        "author_or_organization": author,
        "url": url,
        "doi": doi,
        "path": path,
        "local_source": local_source,
        "retrieved": retrieved,
        "consulted": consulted,
        "source_type": source_type,
        "reliability_limits": reliability,
        "pages_used": pages_used,
        "bucket": bucket,
        "raw_entry": entry.strip(),
        "used_by": [repo_rel(page)],
        "wiki_pages": [repo_rel(page)] if page.is_relative_to(WIKI) else [],
        "quality_flags": quality_flags(locator, source_type, retrieved or consulted, reliability, bucket, local_source),
    }


def quality_flags(
    locator: str | None,
    source_type: str | None,
    date_value: str | None,
    reliability: str | None,
    bucket: str,
    local_source: str | None = None,
) -> list[str]:
    flags = []
    if not locator:
        flags.append("missing_locator")
    if not source_type:
        flags.append("missing_source_type")
    if not date_value or date_value == "YYYY-MM-DD":
        flags.append("missing_or_placeholder_date")
    if not reliability:
        flags.append("missing_reliability_limits")
    if bucket == "internal_fraia":
        flags.append("internal_source_not_original")
    if bucket == "weak_replace":
        flags.append("replace_or_corroborate")
    if local_source or bucket == "textbook_private_reference":
        flags.append("private_local_deferred")
    return flags


def public_rebuild_eligible(source: dict[str, Any]) -> bool:
    has_public_locator = bool(source.get("url") or source.get("doi"))
    return has_public_locator and source["bucket"] in {"public_professional", "software_manual"}


def rebuild_action(source: dict[str, Any]) -> str:
    if public_rebuild_eligible(source):
        return "eligible_for_public_rebuild_seed"
    if source.get("local_source") or source["bucket"] == "textbook_private_reference":
        return "defer_private_local_source"
    if source["bucket"] == "internal_fraia":
        return "trace_to_original_public_source"
    if source["bucket"] == "discovery_only":
        return "use_for_discovery_only"
    if source["bucket"] == "weak_replace":
        return "replace_with_stronger_public_source"
    return "review_before_use"


def source_entries_from_compiled_page(page: Path) -> list[dict[str, Any]]:
    section = source_section(read_text(page))
    entries = []
    for line in section.splitlines():
        match = SOURCE_LINE_RE.match(line.strip())
        if match:
            entries.append(parse_source_entry(page, match.group(1), match.group(2)))
    return entries


def source_entries_from_raw_note(page: Path) -> list[dict[str, Any]]:
    text = read_text(page)
    section = raw_source_section(text)
    if not section:
        return []
    entries = []
    default_retrieved = raw_note_default_retrieval_date(text)
    current_heading: str | None = None
    block_lines: list[str] = []
    block_start = 0

    def flush_block() -> None:
        nonlocal block_lines, block_start, current_heading
        if not block_lines:
            return
        raw = "\n".join(block_lines).strip()
        url = clean_url(extract_labeled(URL_RE, raw)) or first_url(raw)
        if not url:
            block_lines = []
            return
        title = current_heading or raw_bullet_title(raw)
        entry = raw_entry_from_text(page, f"RAW-L{block_start}", raw, title, url, default_retrieved)
        entries.append(entry)
        block_lines = []

    for index, line in enumerate(section.splitlines(), start=1):
        heading = RAW_BLOCK_HEADING_RE.match(line.strip())
        if heading:
            flush_block()
            current_heading = heading.group(1).strip()
            block_start = index
            block_lines = [current_heading]
            continue
        stripped = line.strip()
        if stripped.startswith("- ") and first_url(stripped):
            flush_block()
            current_heading = None
            block_start = index
            block_lines = [stripped]
            flush_block()
            continue
        if stripped.startswith("- ") and block_lines:
            block_lines.append(stripped)
    flush_block()
    return entries


def raw_source_section(text: str) -> str:
    lines = text.splitlines()
    start = None
    for index, line in enumerate(lines):
        if RAW_SOURCE_HEADING_RE.match(line.strip()):
            start = index + 1
            break
    if start is None:
        return ""
    end = len(lines)
    for index in range(start, len(lines)):
        if MARKDOWN_HEADING_RE.match(lines[index].strip()):
            end = index
            break
    return "\n".join(lines[start:end])


def first_url(raw: str) -> str | None:
    match = RAW_URL_LINE_RE.search(raw)
    return clean_url(match.group(0)) if match else None


def raw_note_default_retrieval_date(text: str) -> str | None:
    match = re.search(r"Retrieval date(?: for all (?:web )?sources)?:\s*([0-9]{4}-[0-9]{2}-[0-9]{2})", text, re.IGNORECASE)
    if match:
        return match.group(1)
    match = re.search(r"Retrieved date(?: for all (?:web )?sources)?:\s*([0-9]{4}-[0-9]{2}-[0-9]{2})", text, re.IGNORECASE)
    return match.group(1) if match else None


def raw_bullet_title(raw: str) -> str | None:
    clean = raw.strip().lstrip("- ").strip()
    clean = re.sub(r"\*\*([^*]+)\*\*", r"\1", clean)
    return clean.split(" — ", 1)[0].strip(" -*") or None


def raw_entry_from_text(page: Path, marker: str, raw: str, title: str | None, url: str, default_retrieved: str | None) -> dict[str, Any]:
    source_type = extract_raw_source_type(raw) or infer_raw_source_type(raw, url)
    retrieved = extract_labeled(RETRIEVED_RE, raw) or extract_raw_retrieved(raw) or default_retrieved
    consulted = extract_labeled(CONSULTED_RE, raw)
    reliability = extract_labeled(RELIABILITY_RE, raw) or extract_raw_limits(raw) or infer_raw_reliability(raw, url)
    bucket = classify_source(source_type, url, raw)
    entry = {
        "page_source_marker": marker,
        "title": title,
        "author_or_organization": None,
        "url": url,
        "doi": None,
        "path": None,
        "local_source": None,
        "retrieved": retrieved,
        "consulted": consulted,
        "source_type": source_type,
        "reliability_limits": reliability,
        "pages_used": extract_labeled(PAGES_USED_RE, raw),
        "bucket": bucket,
        "raw_entry": raw.strip(),
        "used_by": [repo_rel(page)],
        "wiki_pages": [repo_rel(page)] if page.is_relative_to(WIKI) else [],
        "quality_flags": [],
    }
    entry["quality_flags"] = quality_flags(
        entry["url"],
        entry["source_type"],
        entry["retrieved"] or entry["consulted"],
        entry["reliability_limits"],
        entry["bucket"],
        entry["local_source"],
    )
    return entry


def extract_raw_retrieved(raw: str) -> str | None:
    match = re.search(r"retrieved:?\s+([0-9]{4}-[0-9]{2}-[0-9]{2})", raw, re.IGNORECASE)
    return match.group(1) if match else None


def extract_raw_source_type(raw: str) -> str | None:
    match = re.search(r"source type:\s*([^;]+)", raw, re.IGNORECASE)
    if match:
        return match.group(1).strip()
    match = re.search(r"—\s*([^;]+?);", raw)
    return match.group(1).strip() if match else None


def infer_raw_source_type(raw: str, url: str) -> str | None:
    text = f"{raw} {url}".lower()
    if "libretexts" in text or "openstax" in text or "teachbooks" in text:
        return "open educational text"
    if "steelconstruction.info" in text:
        return "professional steel construction guidance"
    if any(term in text for term in ["fema.gov", "fhwa.dot.gov", "nist", "nehrp", "basc.pnnl.gov", "jrc.ec.europa.eu", "wbdg.org"]):
        return "public agency guidance"
    if any(term in text for term in ["pynite", "scia", "risa", "lusas", "dlubal", "oasys-software", "opensees", "freecad", "edubeam"]):
        return "public software documentation"
    if "first principle engineering" in text or "fppengineering" in text:
        return "public engineering explainer"
    if "aisc.org" in text:
        return "professional steel industry guidance"
    if "asce.org" in text:
        return "official standards overview"
    return "public web source" if url else None


def extract_raw_limits(raw: str) -> str | None:
    match = re.search(r"(?:limits?|Reliability/limits):\s*(.+?)(?:\.\s*URL:|; retrieved|$)", raw, re.IGNORECASE | re.DOTALL)
    return match.group(1).strip() if match else None


def infer_raw_reliability(raw: str, url: str) -> str | None:
    text = re.sub(r"https?://\S+", "", raw)
    text = re.sub(r"^\s*-\s*", "", text).strip()
    if "—" in text:
        text = text.split("—", 1)[1]
    text = re.sub(r"\bRetrieved\s+[0-9]{4}-[0-9]{2}-[0-9]{2}\.?", "", text, flags=re.IGNORECASE)
    text = re.sub(r"\bretrieved\s+[0-9]{4}-[0-9]{2}-[0-9]{2}\.?", "", text, flags=re.IGNORECASE)
    text = text.strip(" .;")
    if text:
        return text
    inferred = infer_raw_source_type(raw, url)
    if inferred:
        return f"{inferred}; verify scope before promotion"
    return None


def source_entries_from_registry(page: Path) -> list[dict[str, Any]]:
    text = read_text(page)
    entries = []
    current: list[str] = []
    current_id: str | None = None
    for line in text.splitlines():
        match = REGISTRY_ID_RE.match(line)
        if match:
            if current:
                entry = registry_block_entry(page, current_id, current)
                if entry:
                    entries.append(entry)
            current_id = match.group(1).strip()
            current = [line]
            continue
        if current and (line.startswith("  ") or not line.strip()):
            current.append(line)
        elif current:
            entry = registry_block_entry(page, current_id, current)
            if entry:
                entries.append(entry)
            current = []
            current_id = None
    if current:
        entry = registry_block_entry(page, current_id, current)
        if entry:
            entries.append(entry)
    return entries


def registry_block_entry(page: Path, source_id: str | None, lines: list[str]) -> dict[str, Any] | None:
    raw = "\n".join(lines)
    if "..." in raw or "YYYY-MM-DD" in raw:
        return None
    url = clean_url(extract_labeled(URL_RE, raw))
    doi = extract_labeled(DOI_RE, raw)
    path = extract_labeled(PATH_RE, raw)
    local_source = extract_labeled(LOCAL_RE, raw)
    locator = url or doi or path or local_source
    if not locator:
        return None
    title = registry_field(raw, "Title")
    author = registry_field(raw, "Author/organization") or registry_field(raw, "Organization") or registry_field(raw, "Author")
    source_type = extract_labeled(SOURCE_TYPE_RE, raw)
    retrieved = extract_labeled(RETRIEVED_RE, raw)
    consulted = extract_labeled(CONSULTED_RE, raw)
    reliability = extract_labeled(RELIABILITY_RE, raw)
    pages_used = extract_labeled(PAGES_USED_RE, raw)
    bucket = classify_source(source_type, locator, raw)
    entry = {
        "page_source_marker": source_id or "REGISTRY",
        "title": title,
        "author_or_organization": author,
        "url": url,
        "doi": doi,
        "path": path,
        "local_source": local_source,
        "retrieved": retrieved,
        "consulted": consulted,
        "source_type": source_type,
        "reliability_limits": reliability,
        "pages_used": pages_used,
        "bucket": bucket,
        "raw_entry": raw.strip(),
        "used_by": [repo_rel(page)],
        "wiki_pages": [repo_rel(page)] if page.is_relative_to(WIKI) else [],
        "quality_flags": [],
    }
    entry["quality_flags"] = quality_flags(locator, source_type, retrieved or consulted, reliability, bucket, local_source)
    return entry


def registry_field(raw: str, name: str) -> str | None:
    match = re.search(rf"^\s*{re.escape(name)}:\s*(.+)$", raw, re.MULTILINE)
    return match.group(1).strip() if match else None


def merge_entries(entries: list[dict[str, Any]]) -> list[dict[str, Any]]:
    merged: dict[str, dict[str, Any]] = {}
    for entry in entries:
        key = inventory_key(entry)
        source_id = source_id_for(key)
        if source_id not in merged:
            merged[source_id] = {"source_id": source_id, **entry}
            continue
        current = merged[source_id]
        for field in [
            "title", "author_or_organization", "url", "doi", "path", "local_source",
            "retrieved", "consulted", "source_type", "reliability_limits", "pages_used",
        ]:
            if not current.get(field) and entry.get(field):
                current[field] = entry[field]
        current["used_by"] = sorted(set(current["used_by"]) | set(entry["used_by"]))
        current["wiki_pages"] = sorted(set(current.get("wiki_pages", [])) | set(entry.get("wiki_pages", [])))
        current["quality_flags"] = sorted(set(current["quality_flags"]) | set(entry["quality_flags"]))
        current["raw_entry"] = current["raw_entry"]
        if current["bucket"] == "weak_replace" and entry["bucket"] != "weak_replace":
            current["bucket"] = entry["bucket"]
    for current in merged.values():
        locator = current.get("url") or current.get("doi") or current.get("path") or current.get("local_source")
        current["quality_flags"] = quality_flags(
            locator,
            current.get("source_type"),
            current.get("retrieved") or current.get("consulted"),
            current.get("reliability_limits"),
            current["bucket"],
            current.get("local_source"),
        )
        current["wiki_pages"] = sorted(page for page in current["used_by"] if page.startswith("docs/knowledge/wiki/"))
        current["public_rebuild_eligible"] = public_rebuild_eligible(current)
        current["rebuild_action"] = rebuild_action(current)
    return sorted(merged.values(), key=lambda item: (item["bucket"], item.get("title") or item["source_id"]))


def wiki_pages() -> list[Path]:
    return sorted(path for path in WIKI.rglob("*.md") if path.name not in {"README.md", "index.md", "log.md"})


def raw_notes() -> list[Path]:
    return sorted(RAW.glob("*.md"))


def registry_and_plan_files() -> list[Path]:
    files = [KNOWLEDGE / "sources.md"]
    files.extend(sorted((ROOT / "plans").glob("knowledge*.md")))
    return [path for path in files if path.exists()]


def page_audit(compiled_pages: list[Path], sources: list[dict[str, Any]]) -> list[dict[str, Any]]:
    by_page: dict[str, list[dict[str, Any]]] = defaultdict(list)
    for source in sources:
        for page in source["used_by"]:
            by_page[page].append(source)
    audit = []
    for page in compiled_pages:
        rel = repo_rel(page)
        page_sources = by_page.get(rel, [])
        original = [source for source in page_sources if source["bucket"] != "internal_fraia"]
        internal = [source for source in page_sources if source["bucket"] == "internal_fraia"]
        audit.append({
            "page": rel,
            "source_count": len(page_sources),
            "original_source_count": len(original),
            "internal_source_count": len(internal),
            "status": "has_original_sources" if original else "needs_original_source_rebuild",
            "flags": page_flags(page_sources),
            "source_ids": [source["source_id"] for source in page_sources],
        })
    return audit


def page_flags(page_sources: list[dict[str, Any]]) -> list[str]:
    flags = []
    if not page_sources:
        flags.append("no_sources_found")
    if any(source["bucket"] == "internal_fraia" for source in page_sources):
        flags.append("cites_internal_fraia_pages")
    if not any(source["bucket"] != "internal_fraia" for source in page_sources):
        flags.append("no_original_sources")
    if any(source["quality_flags"] for source in page_sources):
        flags.append("source_metadata_needs_cleanup")
    return flags


def build_inventory() -> dict[str, Any]:
    compiled_pages = wiki_pages()
    raw_note_pages = raw_notes()
    registry_plan_pages = registry_and_plan_files()
    entries: list[dict[str, Any]] = []
    for page in compiled_pages:
        entries.extend(source_entries_from_compiled_page(page))
    for page in raw_note_pages:
        entries.extend(source_entries_from_raw_note(page))
    for page in registry_plan_pages:
        entries.extend(source_entries_from_registry(page))
    sources = merge_entries(entries)
    return {
        "schema_version": "knowledge-source-inventory.v0",
        "generated_at": date.today().isoformat(),
        "summary": {
            "compiled_pages_audited": len(compiled_pages),
            "raw_notes_audited": len(raw_note_pages),
            "registry_and_plan_files_audited": len(registry_plan_pages),
            "unique_sources": len(sources),
            "bucket_counts": bucket_counts(sources),
            "public_rebuild_eligible_sources": sum(1 for source in sources if source["public_rebuild_eligible"]),
            "deferred_or_replacement_sources": sum(1 for source in sources if not source["public_rebuild_eligible"]),
            "pages_needing_original_source_rebuild": sum(
                1 for page in page_audit(compiled_pages, sources) if page["status"] == "needs_original_source_rebuild"
            ),
        },
        "sources": sources,
        "page_audit": page_audit(compiled_pages, sources),
    }


def bucket_counts(sources: list[dict[str, Any]]) -> dict[str, int]:
    counts: dict[str, int] = defaultdict(int)
    for source in sources:
        counts[source["bucket"]] += 1
    return dict(sorted(counts.items()))


def markdown_report(inventory: dict[str, Any]) -> str:
    summary = inventory["summary"]
    lines = [
        "# Fraia Knowledge Source Inventory",
        "",
        "_Status: rebuild seed inventory_",
        f"_Generated: {inventory['generated_at']}_",
        "",
        "This inventory is generated from the current `docs/knowledge/` wiki, raw notes, and source registry inputs. It is a rebuild seed, not a new source of engineering truth.",
        "",
        "## Summary",
        "",
        f"- Compiled pages audited: {summary['compiled_pages_audited']}",
        f"- Raw notes audited: {summary['raw_notes_audited']}",
        f"- Registry/knowledge plan files audited: {summary['registry_and_plan_files_audited']}",
        f"- Unique normalized sources: {summary['unique_sources']}",
        f"- Public rebuild-eligible sources: {summary['public_rebuild_eligible_sources']}",
        f"- Deferred/replacement sources: {summary['deferred_or_replacement_sources']}",
        f"- Pages needing original-source rebuild: {summary['pages_needing_original_source_rebuild']}",
        "",
        "## Source Buckets",
        "",
    ]
    for bucket, count in summary["bucket_counts"].items():
        lines.append(f"- `{bucket}`: {count}")
    lines.extend([
        "",
        "## Rebuild Flags",
        "",
        "- `internal_source_not_original`: the page cites another Fraia wiki/doc page and should be traced back to original references during rebuild.",
        "- `missing_locator`, `missing_source_type`, `missing_or_placeholder_date`, `missing_reliability_limits`: source metadata needs cleanup before promotion.",
        "- `replace_or_corroborate`: weak or incomplete source; replace or corroborate with stronger references.",
        "- `private_local_deferred`: private/local source is inventoried only and is not eligible for the public-source rebuild seed.",
        "",
        "## Pages Needing Original-Source Rebuild",
        "",
    ])
    rebuild_pages = [page for page in inventory["page_audit"] if page["status"] == "needs_original_source_rebuild"]
    if rebuild_pages:
        for page in rebuild_pages:
            lines.append(f"- `{page['page']}`")
    else:
        lines.append("- None.")
    lines.extend([
        "",
        "## Source List",
        "",
    ])
    for source in inventory["sources"]:
        locator = source.get("url") or source.get("doi") or source.get("path") or source.get("local_source") or "missing locator"
        title = source.get("title") or "(untitled source)"
        flags = ", ".join(source.get("quality_flags", [])) or "none"
        lines.extend([
            f"### {source['source_id']} — {title}",
            "",
            f"- Bucket: `{source['bucket']}`",
            f"- Locator: {locator}",
            f"- Source type: {source.get('source_type') or 'missing'}",
            f"- Date: {source.get('retrieved') or source.get('consulted') or 'missing'}",
            f"- Reliability/limits: {source.get('reliability_limits') or 'missing'}",
            f"- Pages used: {source.get('pages_used') or 'not yet recorded'}",
            f"- Public rebuild eligible: {str(source.get('public_rebuild_eligible', False)).lower()}",
            f"- Rebuild action: `{source.get('rebuild_action') or 'review_before_use'}`",
            f"- Flags: {flags}",
            f"- Current wiki pages: {', '.join(f'`{page}`' for page in source.get('wiki_pages', [])) or 'none'}",
            f"- Used by: {', '.join(f'`{page}`' for page in source['used_by'])}",
            "",
        ])
    return "\n".join(lines).rstrip() + "\n"


def write_outputs(inventory: dict[str, Any]) -> None:
    KNOWLEDGE_NEXT.mkdir(parents=True, exist_ok=True)
    (KNOWLEDGE_NEXT / "source-inventory.json").write_text(
        json.dumps(inventory, indent=2, sort_keys=True) + "\n",
        encoding="utf-8",
    )
    (KNOWLEDGE_NEXT / "source-inventory.md").write_text(markdown_report(inventory), encoding="utf-8")


def validation_errors(inventory: dict[str, Any]) -> list[str]:
    errors: list[str] = []
    for source in inventory["sources"]:
        locator = source.get("url") or source.get("doi") or source.get("path") or source.get("local_source")
        flags = set(source.get("quality_flags", []))
        source_id = source["source_id"]
        if not locator and "missing_locator" not in flags:
            errors.append(f"{source_id}: missing locator is not flagged")
        if not source.get("source_type") and "missing_source_type" not in flags:
            errors.append(f"{source_id}: missing source type is not flagged")
        if not (source.get("retrieved") or source.get("consulted")) and "missing_or_placeholder_date" not in flags:
            errors.append(f"{source_id}: missing date is not flagged")
        if not source.get("reliability_limits") and "missing_reliability_limits" not in flags:
            errors.append(f"{source_id}: missing reliability limits are not flagged")
        if source.get("public_rebuild_eligible") != public_rebuild_eligible(source):
            errors.append(f"{source_id}: public rebuild eligibility is stale")
        if source.get("rebuild_action") != rebuild_action(source):
            errors.append(f"{source_id}: rebuild action is stale")
        for field in ["path", "local_source"]:
            value = source.get(field)
            if value and (value.startswith("/") or "/Users/" in value):
                errors.append(f"{source_id}: {field} must be logical, not an absolute local path")

    for page in inventory["page_audit"]:
        if page["original_source_count"] == 0 and page["status"] != "needs_original_source_rebuild":
            errors.append(f"{page['page']}: page without original sources is not flagged for rebuild")
        if page["original_source_count"] > 0 and page["status"] != "has_original_sources":
            errors.append(f"{page['page']}: page with original sources has wrong audit status")

    if KNOWLEDGE_NEXT.exists():
        for path in KNOWLEDGE_NEXT.rglob("*"):
            if path.is_file() and path.suffix.lower() in BANNED_KNOWLEDGE_NEXT_SUFFIXES:
                errors.append(f"{repo_rel(path)}: binary/private media should not be committed in knowledge-next")
    return errors


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--check", action="store_true", help="Verify generated inventory files are current")
    args = parser.parse_args()
    inventory = build_inventory()
    if args.check:
        expected_json = json.dumps(inventory, indent=2, sort_keys=True) + "\n"
        expected_md = markdown_report(inventory)
        json_path = KNOWLEDGE_NEXT / "source-inventory.json"
        md_path = KNOWLEDGE_NEXT / "source-inventory.md"
        if not json_path.exists() or not md_path.exists():
            print("source inventory files are missing")
            return 1
        if json_path.read_text(encoding="utf-8") != expected_json:
            print(f"{repo_rel(json_path)} is stale")
            return 1
        if md_path.read_text(encoding="utf-8") != expected_md:
            print(f"{repo_rel(md_path)} is stale")
            return 1
        errors = validation_errors(inventory)
        if errors:
            print("source inventory validation failed")
            for error in errors:
                print(f"- {error}")
            return 1
        print("source inventory is current")
        return 0
    write_outputs(inventory)
    print(f"wrote {repo_rel(KNOWLEDGE_NEXT / 'source-inventory.json')}")
    print(f"wrote {repo_rel(KNOWLEDGE_NEXT / 'source-inventory.md')}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
