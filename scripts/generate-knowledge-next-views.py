#!/usr/bin/env python3
"""Generate markdown reader views from docs/knowledge-next typed records."""
from __future__ import annotations

import json
from pathlib import Path
from typing import Any

ROOT = Path(__file__).resolve().parents[1]
KNOWLEDGE_NEXT = ROOT / "docs" / "knowledge-next"
CARDS = KNOWLEDGE_NEXT / "cards"
ASSETS = KNOWLEDGE_NEXT / "assets"
EVALS = KNOWLEDGE_NEXT / "evals"
VIEWS = KNOWLEDGE_NEXT / "generated" / "views"


def read_json(path: Path) -> dict[str, Any]:
    return json.loads(path.read_text(encoding="utf-8"))


def json_files(root: Path) -> list[Path]:
    if not root.exists():
        return []
    return sorted(path for path in root.rglob("*.json") if path.is_file())


def rel(path: Path) -> str:
    return path.relative_to(ROOT).as_posix()


def write(path: Path, lines: list[str]) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text("\n".join(lines).rstrip() + "\n", encoding="utf-8")


def generated_header(title: str) -> list[str]:
    return [
        f"# {title}",
        "",
        "> Generated from typed records. Do not hand-edit this file; run `python3 scripts/generate-knowledge-next-views.py`.",
        "",
    ]


def render_cards(cards: list[dict[str, Any]]) -> list[str]:
    lines = generated_header("Knowledge Cards Index")
    for card in sorted(cards, key=lambda item: item["card_id"]):
        lines.extend(
            [
                f"## {card['card_id']} - {card['title']}",
                "",
                f"- Status: `{card['status']}`",
                f"- Domain: `{card['domain']}`",
                f"- Concepts: {', '.join(card['concepts'])}",
                f"- Summary: {card['summary']}",
                f"- Sources: {', '.join(link['source_id'] for link in card['source_links'])}",
            ]
        )
        if card.get("media_links"):
            lines.append(f"- Media: {', '.join(media['asset_id'] for media in card['media_links'])}")
        if card.get("relationships"):
            rels = [f"{rel['relationship_type']} {rel['target']}" for rel in card["relationships"]]
            lines.append(f"- Relationships: {', '.join(rels)}")
        lines.append("")
    return lines


def render_assets(assets: list[dict[str, Any]]) -> list[str]:
    lines = generated_header("Knowledge Assets Index")
    for asset in sorted(assets, key=lambda item: item["asset_id"]):
        lines.extend(
            [
                f"## {asset['asset_id']} - {asset['title']}",
                "",
                f"- Type: `{asset['asset_type']}`",
                f"- Source: `{asset['source_id']}`",
                f"- Embed policy: `{asset['embed_policy']}`",
                f"- Redraw status: `{asset['redraw_status']}`",
                f"- Tags: {', '.join(asset['concept_tags'])}",
                f"- Used by: {', '.join(asset.get('used_by_cards', []))}",
                f"- Target: `{asset.get('generated_safe_target', {}).get('candidate_output_path', 'none')}`",
                "",
            ]
        )
    return lines


def render_evals(evals: list[dict[str, Any]]) -> list[str]:
    lines = generated_header("Retrieval Eval Seeds")
    for eval_case in sorted(evals, key=lambda item: item["eval_id"]):
        lines.extend(
            [
                f"## {eval_case['eval_id']} - {eval_case['title']}",
                "",
                f"- Prompt: {eval_case['prompt']}",
                f"- Expected cards: {', '.join(eval_case['expected_card_ids'])}",
                f"- Expected concepts: {', '.join(eval_case['expected_concepts'])}",
                "- Unacceptable patterns:",
            ]
        )
        for pattern in eval_case["unacceptable_answer_patterns"]:
            lines.append(f"  - {pattern}")
        lines.append("")
    return lines


def main() -> int:
    cards = [read_json(path) for path in json_files(CARDS)]
    assets = [read_json(path) for path in json_files(ASSETS)]
    evals = [read_json(path) for path in json_files(EVALS)]

    write(VIEWS / "cards-index.md", render_cards(cards))
    write(VIEWS / "assets-index.md", render_assets(assets))
    write(VIEWS / "evals-index.md", render_evals(evals))

    print(f"generated {rel(VIEWS / 'cards-index.md')}")
    print(f"generated {rel(VIEWS / 'assets-index.md')}")
    print(f"generated {rel(VIEWS / 'evals-index.md')}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
