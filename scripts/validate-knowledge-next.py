#!/usr/bin/env python3
"""Validate Fraia knowledge-next typed records without external dependencies."""
from __future__ import annotations

import json
import re
from pathlib import Path
from typing import Any

ROOT = Path(__file__).resolve().parents[1]
KNOWLEDGE_NEXT = ROOT / "docs" / "knowledge-next"
CARDS = KNOWLEDGE_NEXT / "cards"
ASSETS = KNOWLEDGE_NEXT / "assets"
EVALS = KNOWLEDGE_NEXT / "evals"
INVENTORY = KNOWLEDGE_NEXT / "source-inventory.json"

CARD_ID_RE = re.compile(r"^KC-[a-z0-9][a-z0-9-]*$")
ASSET_ID_RE = re.compile(r"^KA-[a-z0-9][a-z0-9-]*$")
SOURCE_ID_RE = re.compile(r"^SRC-[a-f0-9]{10}$")
BANNED_MEDIA_SUFFIXES = {".gif", ".jpeg", ".jpg", ".pdf", ".png", ".tif", ".tiff", ".webp"}

CARD_REQUIRED = {
    "card_id", "schema_version", "title", "status", "domain", "summary", "concepts",
    "claims", "source_links", "applicability", "limitations", "relationships", "media_links",
}
ASSET_REQUIRED = {
    "asset_id", "schema_version", "title", "asset_type", "source_id", "original_locator",
    "copyright_status", "embed_policy", "caption", "concept_tags", "alt_text", "redraw_status",
}
EVAL_REQUIRED = {
    "eval_id", "schema_version", "title", "prompt", "expected_card_ids",
    "expected_concepts", "unacceptable_answer_patterns",
}


def rel(path: Path) -> str:
    return path.relative_to(ROOT).as_posix()


def read_json(path: Path) -> Any:
    return json.loads(path.read_text(encoding="utf-8"))


def load_inventory() -> dict[str, dict[str, Any]]:
    inventory = read_json(INVENTORY)
    return {source["source_id"]: source for source in inventory["sources"]}


def json_files(root: Path) -> list[Path]:
    if not root.exists():
        return []
    return sorted(path for path in root.rglob("*.json") if path.is_file())


def validate_card(path: Path, card: dict[str, Any], sources: dict[str, dict[str, Any]], asset_ids: set[str]) -> list[str]:
    errors: list[str] = []
    missing = sorted(CARD_REQUIRED - set(card))
    if missing:
        errors.append(f"{rel(path)}: missing required fields {', '.join(missing)}")
        return errors
    card_id = card["card_id"]
    if not CARD_ID_RE.match(card_id):
        errors.append(f"{rel(path)}: invalid card_id {card_id!r}")
    if card["schema_version"] != "knowledge-card.v0":
        errors.append(f"{rel(path)}: schema_version must be knowledge-card.v0")
    if card["status"] not in {"seed", "draft", "reviewed", "deprecated"}:
        errors.append(f"{rel(path)}: invalid status {card['status']!r}")
    if not isinstance(card["claims"], list) or not card["claims"]:
        errors.append(f"{rel(path)}: claims must be a non-empty list")
    if not isinstance(card["source_links"], list) or not card["source_links"]:
        errors.append(f"{rel(path)}: source_links must be a non-empty list")

    source_link_ids = set()
    for link in card.get("source_links", []):
        source_id = link.get("source_id")
        if not source_id or not SOURCE_ID_RE.match(source_id):
            errors.append(f"{rel(path)}: invalid source_links source_id {source_id!r}")
            continue
        source_link_ids.add(source_id)
        source = sources.get(source_id)
        if not source:
            errors.append(f"{rel(path)}: source {source_id} is not in source-inventory.json")
        elif not source.get("public_rebuild_eligible"):
            errors.append(f"{rel(path)}: source {source_id} is not public rebuild-eligible")

    claim_ids = set()
    for claim in card.get("claims", []):
        claim_id = claim.get("claim_id")
        if claim_id in claim_ids:
            errors.append(f"{rel(path)}: duplicate claim_id {claim_id}")
        claim_ids.add(claim_id)
        refs = claim.get("source_refs")
        if not isinstance(refs, list) or not refs:
            errors.append(f"{rel(path)}: claim {claim_id} has no source_refs")
            continue
        for ref in refs:
            source_id = ref.get("source_id")
            if source_id not in source_link_ids:
                errors.append(f"{rel(path)}: claim {claim_id} references source {source_id} not listed in source_links")
            source = sources.get(source_id)
            if source and not source.get("public_rebuild_eligible"):
                errors.append(f"{rel(path)}: claim {claim_id} references non-public source {source_id}")
            if not ref.get("locator"):
                errors.append(f"{rel(path)}: claim {claim_id} has a source ref without locator")

    for media in card.get("media_links", []):
        asset_id = media.get("asset_id")
        if asset_id not in asset_ids:
            errors.append(f"{rel(path)}: media link asset {asset_id} does not exist")
    return errors


def validate_asset(path: Path, asset: dict[str, Any], sources: dict[str, dict[str, Any]]) -> list[str]:
    errors: list[str] = []
    missing = sorted(ASSET_REQUIRED - set(asset))
    if missing:
        errors.append(f"{rel(path)}: missing required fields {', '.join(missing)}")
        return errors
    asset_id = asset["asset_id"]
    if not ASSET_ID_RE.match(asset_id):
        errors.append(f"{rel(path)}: invalid asset_id {asset_id!r}")
    if asset["schema_version"] != "knowledge-asset.v0":
        errors.append(f"{rel(path)}: schema_version must be knowledge-asset.v0")
    source_id = asset["source_id"]
    if source_id not in sources:
        errors.append(f"{rel(path)}: source {source_id} is not in source-inventory.json")
    elif not sources[source_id].get("public_rebuild_eligible"):
        errors.append(f"{rel(path)}: source {source_id} is not public rebuild-eligible")
    if asset["embed_policy"] in {"embed_allowed", "link_only"} and asset["copyright_status"] in {"permission_required", "private_reference_only", "unknown"}:
        errors.append(f"{rel(path)}: embed policy is too permissive for copyright_status {asset['copyright_status']!r}")
    target = asset.get("generated_safe_target", {}).get("candidate_output_path")
    if target and Path(target).suffix.lower() in BANNED_MEDIA_SUFFIXES and not target.startswith("docs/knowledge-next/generated/"):
        errors.append(f"{rel(path)}: generated media output must live under docs/knowledge-next/generated/")
    return errors


def validate_eval(path: Path, eval_case: dict[str, Any], card_ids: set[str], card_concepts: set[str]) -> list[str]:
    errors: list[str] = []
    missing = sorted(EVAL_REQUIRED - set(eval_case))
    if missing:
        errors.append(f"{rel(path)}: missing required fields {', '.join(missing)}")
        return errors
    eval_id = eval_case["eval_id"]
    if not re.match(r"^KE-[a-z0-9][a-z0-9-]*$", eval_id):
        errors.append(f"{rel(path)}: invalid eval_id {eval_id!r}")
    if eval_case["schema_version"] != "knowledge-eval.v0":
        errors.append(f"{rel(path)}: schema_version must be knowledge-eval.v0")
    expected_cards = eval_case.get("expected_card_ids")
    if not isinstance(expected_cards, list) or not expected_cards:
        errors.append(f"{rel(path)}: expected_card_ids must be a non-empty list")
    else:
        for card_id in expected_cards:
            if card_id not in card_ids:
                errors.append(f"{rel(path)}: expected card {card_id} does not exist")
    expected_concepts = eval_case.get("expected_concepts")
    if not isinstance(expected_concepts, list) or not expected_concepts:
        errors.append(f"{rel(path)}: expected_concepts must be a non-empty list")
    else:
        missing_concepts = [concept for concept in expected_concepts if concept not in card_concepts]
        if missing_concepts:
            errors.append(f"{rel(path)}: expected concepts not present in card concepts: {', '.join(missing_concepts)}")
    bad_patterns = eval_case.get("unacceptable_answer_patterns")
    if not isinstance(bad_patterns, list) or not bad_patterns:
        errors.append(f"{rel(path)}: unacceptable_answer_patterns must be a non-empty list")
    return errors


def validate_banned_media() -> list[str]:
    errors = []
    for path in KNOWLEDGE_NEXT.rglob("*"):
        if path.is_file() and path.suffix.lower() in BANNED_MEDIA_SUFFIXES:
            errors.append(f"{rel(path)}: copied/source media files are not allowed under knowledge-next")
    return errors


def main() -> int:
    errors: list[str] = []
    sources = load_inventory()
    asset_files = json_files(ASSETS)
    card_files = json_files(CARDS)
    eval_files = json_files(EVALS)
    assets = {}
    card_ids = set()
    card_concepts = set()

    for path in asset_files:
        asset = read_json(path)
        asset_id = asset.get("asset_id")
        if asset_id in assets:
            errors.append(f"{rel(path)}: duplicate asset_id {asset_id}")
        assets[asset_id] = asset
        errors.extend(validate_asset(path, asset, sources))

    for path in card_files:
        card = read_json(path)
        card_ids.add(card.get("card_id"))
        card_concepts.update(card.get("concepts", []))
        errors.extend(validate_card(path, card, sources, set(assets)))

    for path in eval_files:
        eval_case = read_json(path)
        errors.extend(validate_eval(path, eval_case, card_ids, card_concepts))

    errors.extend(validate_banned_media())

    if errors:
        print("knowledge-next validation failed")
        for error in errors:
            print(f"- {error}")
        return 1
    print(f"knowledge-next validation passed ({len(card_files)} cards, {len(asset_files)} assets, {len(eval_files)} evals)")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
