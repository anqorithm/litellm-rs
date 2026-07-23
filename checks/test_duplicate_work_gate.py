"""Fail-closed retained-branch disposition tests."""

from __future__ import annotations

import unittest
from datetime import datetime, timedelta, timezone
from pathlib import Path
from types import SimpleNamespace
from unittest.mock import patch
import subprocess

from duplicate_work_gate import (
    _legacy_validator_schema,
    _load_schema,
    evaluate_duplicate_work_gate,
    revalidate_remote_state,
)
from github_duplicate_evidence import (
    EvidenceError,
    bind_retained_branch_dispositions,
    build_evidence,
    collect_open_prs,
)
from route_gate import evaluate_route
from specrail_lib import SpecRailError, load_pack, validate_instance


ISSUE = 1107
BRANCH = "codex/gh1107-codex-responses-compat"
HEAD = "a" * 40
BASE = "b" * 40
SOURCE = {
    "kind": "maintainer_human",
    "reference": "https://github.com/majiayu000/litellm-rs/issues/1107#retained-branch",
    "recorded_at": "2026-07-24T00:00:00Z",
}


def disposition(
    *,
    head_sha: str = HEAD,
    value: str = "retained_non_conflicting",
    branch: str = BRANCH,
) -> dict[str, object]:
    return {
        "branch": branch,
        "head_sha": head_sha,
        "disposition": value,
        "source": dict(SOURCE),
    }


def evidence(dispositions: list[dict[str, object]] | None = None) -> dict[str, object]:
    return build_evidence(
        ISSUE,
        [],
        [{"name": "main", "head_sha": BASE}, {"name": BRANCH, "head_sha": HEAD}],
        100,
        dispositions or [],
    )


class DuplicateWorkGateTest(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.repo = Path(__file__).resolve().parents[1]
        cls.config = load_pack(cls.repo)

    def result(self, payload: dict[str, object]) -> dict[str, object]:
        return evaluate_duplicate_work_gate(self.config, ISSUE, payload)

    def test_exact_human_disposition_is_audit_only(self) -> None:
        result = self.result(evidence([disposition()]))
        self.assertEqual(result["decision"], "needs_human")
        self.assertIn("audit evidence", result["reasons"][0])
        self.assertNotIn(
            "no remote branch matches implementation token gh1107",
            result["satisfied"],
        )

    def test_missing_disposition_needs_human(self) -> None:
        result = self.result(evidence())
        self.assertEqual(result["decision"], "needs_human")
        self.assertIn("retained_branch_disposition", result["missing"][0])

    def test_sha_drift_needs_human(self) -> None:
        payload = evidence([disposition()])
        payload["retained_branch_dispositions"][0]["head_sha"] = "b" * 40
        result = self.result(payload)
        self.assertEqual(result["decision"], "needs_human")

    def test_active_duplicate_blocks(self) -> None:
        result = self.result(evidence([disposition(value="active_duplicate")]))
        self.assertEqual(result["decision"], "blocked")

    def test_duplicate_dispositions_block(self) -> None:
        payload = evidence([disposition()])
        payload["retained_branch_dispositions"].append(disposition())
        result = self.result(payload)
        self.assertEqual(result["decision"], "blocked")

    def test_orphan_disposition_blocks(self) -> None:
        payload = evidence([disposition()])
        payload["retained_branch_dispositions"][0]["branch"] = "codex/gh1107-old"
        result = self.result(payload)
        self.assertEqual(result["decision"], "blocked")

    def test_open_pr_blocks_even_with_disposition(self) -> None:
        payload = evidence([disposition()])
        payload["open_prs"] = [{"number": 42, "head_ref": BRANCH, "references_issue": True}]
        result = self.result(payload)
        self.assertEqual(result["decision"], "blocked")

    def test_collector_rejects_unbound_disposition(self) -> None:
        with self.assertRaises(EvidenceError):
            bind_retained_branch_dispositions(
                [{"name": BRANCH, "head_sha": HEAD}],
                [disposition(head_sha="b" * 40)],
            )

    def test_schema_rejects_legacy_name_only_branch_evidence(self) -> None:
        payload = evidence([disposition()])
        payload["remote_branches"] = [BRANCH]
        with self.assertRaises(SpecRailError):
            validate_instance(_legacy_validator_schema(_load_schema(self.repo)), payload)

    def test_invalid_sha_blocks_even_when_schema_length_matches(self) -> None:
        payload = evidence([disposition()])
        payload["remote_branches"][0]["head_sha"] = "z" * 40
        result = self.result(payload)
        self.assertEqual(result["decision"], "blocked")

    def test_blank_source_timestamp_needs_human(self) -> None:
        payload = evidence([disposition()])
        payload["retained_branch_dispositions"][0]["source"]["recorded_at"] = " "
        result = self.result(payload)
        self.assertEqual(result["decision"], "needs_human")

    def test_stale_evidence_blocks(self) -> None:
        payload = evidence([disposition()])
        payload["collected_at"] = (
            datetime.now(timezone.utc) - timedelta(hours=1)
        ).isoformat().replace("+00:00", "Z")
        self.assertEqual(self.result(payload)["decision"], "blocked")

    def test_schema_fallback_and_gate_declare_sha_contract(self) -> None:
        schema = _load_schema(self.repo)
        self.assertEqual(schema["properties"]["base_sha"]["minLength"], 40)
        branch_sha = schema["properties"]["remote_branches"]["items"]["properties"]["head_sha"]
        disposition_sha = schema["properties"]["retained_branch_dispositions"]["items"]["properties"]["head_sha"]
        self.assertEqual(branch_sha["minLength"], 40)
        self.assertEqual(disposition_sha["minLength"], 40)
        self.assertIsNotNone(__import__("duplicate_work_gate").SHA_RE.fullmatch("a" * 40))
        self.assertIsNone(__import__("duplicate_work_gate").SHA_RE.fullmatch("A" * 40))

    def test_uppercase_sha_blocks_with_legacy_validator(self) -> None:
        payload = evidence([disposition()])
        payload["base_sha"] = "B" * 40
        self.assertEqual(self.result(payload)["decision"], "blocked")

    def test_invalid_sha_blocks_without_matching_contract_branch(self) -> None:
        payload = evidence()
        payload["remote_branches"] = [
            {"name": "main", "head_sha": BASE},
            {"name": "unrelated", "head_sha": "Z" * 40},
        ]
        self.assertEqual(self.result(payload)["decision"], "blocked")

    def test_collector_auth_failure_is_fail_closed(self) -> None:
        failed = subprocess.CompletedProcess(["gh"], 1, "", "authentication required")
        with patch("github_duplicate_evidence._run_command", return_value=failed):
            with self.assertRaises(EvidenceError):
                collect_open_prs("owner/repo", 100)

    def remote_result(
        self,
        *,
        remote_base: str = BASE,
        remote_head: str = HEAD,
        extra_branch: tuple[str, str] | None = None,
        object_exists: bool = True,
        ancestor: bool = True,
    ) -> dict[str, object] | None:
        lines = [
            f"{remote_base}\trefs/heads/main",
            f"{remote_head}\trefs/heads/{BRANCH}",
        ]
        if extra_branch:
            lines.append(f"{extra_branch[1]}\trefs/heads/{extra_branch[0]}")

        def fake_run(_repo: Path, args: list[str]) -> subprocess.CompletedProcess[str]:
            if args[:2] == ["ls-remote", "--heads"]:
                return subprocess.CompletedProcess(args, 0, "\n".join(lines) + "\n", "")
            if args[0] == "rev-parse":
                return subprocess.CompletedProcess(args, 0, BASE + "\n", "")
            if args[0] == "cat-file":
                return subprocess.CompletedProcess(args, 0 if object_exists else 1, "", "")
            if args[0] == "merge-base":
                return subprocess.CompletedProcess(args, 0 if ancestor else 1, "", "")
            raise AssertionError(args)

        with patch("duplicate_work_gate._run_git", side_effect=fake_run):
            return revalidate_remote_state(
                self.repo, self.config, ISSUE, evidence([disposition()])
            )

    def test_gate_time_remote_revalidation_accepts_consistent_objects(self) -> None:
        self.assertIsNone(self.remote_result())

    def test_gate_time_base_drift_blocks(self) -> None:
        self.assertEqual(self.remote_result(remote_base="c" * 40)["decision"], "blocked")

    def test_gate_time_matching_branch_drift_blocks(self) -> None:
        self.assertEqual(self.remote_result(remote_head="c" * 40)["decision"], "blocked")

    def test_gate_time_all_matching_branches_must_be_in_evidence(self) -> None:
        result = self.remote_result(
            extra_branch=("agent/gh1107-second", "c" * 40)
        )
        self.assertEqual(result["decision"], "blocked")

    def test_gate_time_missing_object_blocks(self) -> None:
        self.assertEqual(self.remote_result(object_exists=False)["decision"], "blocked")

    def test_gate_time_non_ancestor_blocks(self) -> None:
        self.assertEqual(self.remote_result(ancestor=False)["decision"], "blocked")

    def test_required_route_preserves_duplicate_gate_block(self) -> None:
        args = SimpleNamespace(
            repo=str(self.repo),
            route="implement",
            issue=ISSUE,
            pr=None,
            state="ready_to_implement",
            label=[],
            artifact=[],
            evidence=None,
            duplicate_evidence="unused.json",
            mode="required",
        )
        blocked = {
            "decision": "blocked",
            "reasons": ["base drift"],
            "satisfied": [],
            "missing": [],
        }
        with patch("route_gate.evaluate_duplicate_work_gate_path", return_value=blocked):
            result = evaluate_route(args)
        self.assertEqual(result["decision"], "blocked")
        self.assertIn("duplicate_work: base drift", result["reasons"])


if __name__ == "__main__":
    unittest.main()
