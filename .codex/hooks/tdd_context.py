#!/usr/bin/env python3
"""Inject a concise, non-blocking TDD protocol into Codex lifecycle events."""

import json
import sys


CONTEXT = {
    "UserPromptSubmit": """[tmrch TDD workflow]
For GitHub Issue implementation, first fetch the Issue and turn each observable acceptance criterion into an ID (AC-1, AC-2, ...). Work one criterion at a time: write a focused failing test (Red), run it and record the expected assertion failure, then make the smallest change and rerun it (Green). Keep Red and Green as separate jj changes when this repository's existing changes permit it. Record evidence in docs/evidence/issue-<number>.json and validate it with the issue-driven-tdd check_evidence.py script. Ask the tdd_test_designer or tdd_reviewer SubAgent for bounded, read-only work when it would improve confidence; use at most one writing SubAgent per Issue.""",
    "SubagentStart": """[tmrch SubAgent protocol]
State your assigned scope and return a concise summary with files inspected, proposed test IDs or findings, and commands run. Do not edit files if you are a read-only agent. A writing SubAgent owns only one Issue and must not overwrite existing user changes. For implementation, follow Red -> recorded evidence -> Green for one acceptance criterion at a time.""",
    "SubagentStop": """[tmrch SubAgent handoff]
The parent agent must reconcile this SubAgent's summary before proceeding. Treat a test result as evidence only when it includes the exact command, exit status, and whether it was the intended Red or Green result. Do not claim an Issue complete until its acceptance criteria and evidence are accounted for.""",
}


def main() -> None:
    try:
        event = json.load(sys.stdin)
    except json.JSONDecodeError:
        return

    event_name = event.get("hook_event_name")
    context = CONTEXT.get(event_name)
    if not context:
        return

    print(json.dumps({"hookSpecificOutput": {
        "hookEventName": event_name,
        "additionalContext": context,
    }}, ensure_ascii=False))


if __name__ == "__main__":
    main()
