#!/usr/bin/env python3
"""Reject confirmed disposable or obsolete files when they are tracked."""
from __future__ import annotations

import fnmatch
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]

FORBIDDEN_EXACT = {
    "MEMORY.md",
    "PLAN.md",
    "NOW.md",
    "WORKLOG.md",
    "BACKLOG.md",
    "ROADMAP.md",
    "HANDOFF.md",
    "context.md",
    "progress.md",
    "research.md",
    "docs/active-implementation-log.md",
    "docs/engineering-core-plan.md",
    "docs/fraia-system-graphs.md",
    "docs/knowledge/viewer.html",
    "docs/ontology-and-type-hierarchy.md",
}

FORBIDDEN_PREFIXES = (
    "apps/fraia-electron/dist/",
    "artifacts/",
    "docs/graphs/",
    "plans/.subagent-",
)

FORBIDDEN_GLOBS = (
    "**/__pycache__/*",
    "**/*.pyc",
)

ACTIVE_RUNTIME_FILES = (
    "apps/fraia-appd/src/main.rs",
    "apps/fraia-electron/main.js",
    "apps/fraia-electron/preload.js",
    "apps/fraia-electron/package.json",
)

FORBIDDEN_ACTIVE_RUNTIME_TEXT = (
    'Command::new("codex")',
    "spawn('codex'",
    'spawn("codex"',
    "codex exec",
    "codex login",
    "codex debug models",
    "auth.json",
)


def tracked_files() -> list[str]:
    result = subprocess.run(
        ["git", "ls-files", "-z"],
        cwd=ROOT,
        check=True,
        stdout=subprocess.PIPE,
    )
    return [
        entry.decode()
        for entry in result.stdout.split(b"\0")
        if entry and (ROOT / entry.decode()).exists()
    ]


def is_forbidden(path: str) -> bool:
    return (
        path in FORBIDDEN_EXACT
        or path.startswith(FORBIDDEN_PREFIXES)
        or any(fnmatch.fnmatch(path, pattern) for pattern in FORBIDDEN_GLOBS)
    )


def main() -> int:
    violations = sorted(path for path in tracked_files() if is_forbidden(path))
    if violations:
        print("Repository hygiene check failed; disposable or obsolete files are tracked:")
        for path in violations:
            print(f"- {path}")
        return 1

    runtime_violations: list[str] = []
    for relative_path in ACTIVE_RUNTIME_FILES:
        path = ROOT / relative_path
        if not path.exists():
            continue
        text = path.read_text(encoding="utf-8")
        for forbidden in FORBIDDEN_ACTIVE_RUNTIME_TEXT:
            if forbidden in text:
                runtime_violations.append(f"{relative_path}: {forbidden}")
    if runtime_violations:
        print("Repository hygiene check failed; obsolete Codex CLI runtime text returned:")
        for violation in runtime_violations:
            print(f"- {violation}")
        return 1

    print("Repository hygiene check passed.")
    return 0


if __name__ == "__main__":
    sys.exit(main())
