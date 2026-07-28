#!/usr/bin/env python3
"""Check that a Fraia knowledge update has recorded Steward review evidence.

This is a deterministic evidence gate. It does not perform the Steward judgment.
For compiled wiki promotion, it verifies that a reviewer/steward decision was
recorded and that the decision is promotable.
"""
from __future__ import annotations

import argparse
import re
import sys
from dataclasses import dataclass
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]

ALL_DECISIONS = {
    "accept",
    "accept-with-edits",
    "needs-more-source",
    "downgrade-to-draft",
    "veto",
}
PROMOTABLE_DECISIONS = {"accept", "accept-with-edits"}

REQUIRED_CHECK_PHRASES = [
    "Fraia product relevance",
    "Architecture fit",
    "Authored structural state",
    "Structural vocabulary",
    "Vendor/software",
    "Source/confidence",
]

DECISION_LINE_RE = re.compile(
    r"(?:Decision|Steward decision|Classification)\s*(?::|was)\s*(.+?)\s*$",
    re.IGNORECASE,
)


@dataclass(frozen=True)
class EvidenceResult:
    path: Path
    decisions: list[str]
    missing_check_phrases: list[str]


def repo_path(path: str) -> Path:
    candidate = Path(path)
    if not candidate.is_absolute():
        candidate = ROOT / candidate
    return candidate


def display_path(path: Path) -> str:
    try:
        return str(path.relative_to(ROOT))
    except ValueError:
        return str(path)


def normalize_decision(decision: str) -> str:
    return decision.lower().strip("` .;")


def decision_from_line(line: str) -> str | None:
    match = DECISION_LINE_RE.search(line.strip())
    if not match:
        return None
    value = match.group(1).strip()
    if "|" in value:
        return None
    if value.startswith("`"):
        end = value.find("`", 1)
        if end == -1:
            return None
        value = value[1:end]
    else:
        value = value.split(None, 1)[0]
    decision = normalize_decision(value)
    return decision if decision in ALL_DECISIONS else None


def inspect_evidence(path: Path) -> EvidenceResult:
    text = path.read_text()
    decisions = [
        decision
        for line in text.splitlines()
        if (decision := decision_from_line(line)) is not None
    ]
    missing = [phrase for phrase in REQUIRED_CHECK_PHRASES if phrase not in text]
    return EvidenceResult(path=path, decisions=decisions, missing_check_phrases=missing)


def parse_args(argv: list[str]) -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description=(
            "Verify recorded Fraia Knowledge Steward review evidence. "
            "Use --evidence for a wiki update proposal, PR body export, or maintenance log."
        )
    )
    parser.add_argument(
        "--evidence",
        action="append",
        default=[],
        help="Markdown file containing Steward checklist and decision evidence. May be repeated.",
    )
    parser.add_argument(
        "--allow-non-promotable",
        action="store_true",
        help="Allow needs-more-source, downgrade-to-draft, or veto decisions. Use for audit-only checks.",
    )
    parser.add_argument(
        "--require-checklist",
        action="store_true",
        help="Require the evidence file to mention all Steward checklist categories.",
    )
    return parser.parse_args(argv)


def main(argv: list[str] | None = None) -> int:
    args = parse_args(sys.argv[1:] if argv is None else argv)
    errors: list[str] = []

    if not args.evidence:
        errors.append("provide at least one --evidence markdown file")

    results: list[EvidenceResult] = []
    for raw_path in args.evidence:
        path = repo_path(raw_path)
        if not path.exists():
            errors.append(f"{raw_path}: evidence file does not exist")
            continue
        if not path.is_file():
            errors.append(f"{raw_path}: evidence path is not a file")
            continue
        results.append(inspect_evidence(path))

    found_decisions: list[tuple[Path, str]] = []
    for result in results:
        for decision in result.decisions:
            if decision not in ALL_DECISIONS:
                errors.append(f"{result.path}: invalid Steward decision `{decision}`")
                continue
            found_decisions.append((result.path, decision))
        if args.require_checklist and result.missing_check_phrases:
            missing = ", ".join(result.missing_check_phrases)
            errors.append(f"{result.path}: missing Steward checklist categories: {missing}")

    if results and not found_decisions:
        paths = ", ".join(display_path(result.path) for result in results)
        errors.append(f"{paths}: no Steward decision found")

    if found_decisions and not args.allow_non_promotable:
        non_promotable = [
            (path, decision)
            for path, decision in found_decisions
            if decision not in PROMOTABLE_DECISIONS
        ]
        if non_promotable:
            formatted = ", ".join(
                f"{display_path(path)}={decision}" for path, decision in non_promotable
            )
            errors.append(f"non-promotable Steward decision for compiled update: {formatted}")

    if errors:
        for error in errors:
            print(error, file=sys.stderr)
        return 1

    formatted = ", ".join(
        f"{display_path(path)}={decision}" for path, decision in found_decisions
    )
    print(f"Fraia Knowledge Steward evidence passed: {formatted}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
