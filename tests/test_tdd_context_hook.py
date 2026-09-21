"""Contract tests for the repository-local Codex TDD hook."""

import json
from pathlib import Path
import subprocess
import sys
import unittest


ROOT = Path(__file__).resolve().parents[1]
HOOK = ROOT / ".codex" / "hooks" / "tdd_context.py"


class TddContextHookTests(unittest.TestCase):
    def invoke(self, event_name: str) -> dict:
        result = subprocess.run(
            [sys.executable, str(HOOK)],
            input=json.dumps({"hook_event_name": event_name}),
            text=True,
            capture_output=True,
            check=True,
        )
        return json.loads(result.stdout)

    def test_user_prompt_injects_red_green_evidence_protocol(self) -> None:
        payload = self.invoke("UserPromptSubmit")
        context = payload["hookSpecificOutput"]["additionalContext"]
        self.assertEqual(payload["hookSpecificOutput"]["hookEventName"], "UserPromptSubmit")
        self.assertIn("Red", context)
        self.assertIn("Green", context)
        self.assertIn("docs/evidence/issue-<number>.json", context)

    def test_subagent_start_injects_handoff_requirements(self) -> None:
        payload = self.invoke("SubagentStart")
        context = payload["hookSpecificOutput"]["additionalContext"]
        self.assertEqual(payload["hookSpecificOutput"]["hookEventName"], "SubagentStart")
        self.assertIn("summary", context)
        self.assertIn("Red -> recorded evidence -> Green", context)

    def test_subagent_stop_requests_evidence_reconciliation(self) -> None:
        payload = self.invoke("SubagentStop")
        context = payload["hookSpecificOutput"]["additionalContext"]
        self.assertEqual(payload["hookSpecificOutput"]["hookEventName"], "SubagentStop")
        self.assertIn("exact command", context)
        self.assertIn("acceptance criteria", context)

    def test_unknown_event_is_silent(self) -> None:
        result = subprocess.run(
            [sys.executable, str(HOOK)],
            input=json.dumps({"hook_event_name": "Unknown"}),
            text=True,
            capture_output=True,
            check=True,
        )
        self.assertEqual(result.stdout, "")


if __name__ == "__main__":
    unittest.main()
