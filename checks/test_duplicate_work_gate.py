"""Fail-closed retained-branch disposition tests."""

from __future__ import annotations

import unittest
from pathlib import Path

from duplicate_work_gate import _load_schema, evaluate_duplicate_work_gate
from github_duplicate_evidence import EvidenceError, bind_retained_branch_dispositions, build_evidence
from specrail_lib import SpecRailError, load_pack, validate_instance


ISSUE = 1107
BRANCH = "codex/gh1107-codex-responses-compat"
HEAD = "a" * 40
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
        [{"name": BRANCH, "head_sha": HEAD}],
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

    def test_exact_human_disposition_allows_retained_branch(self) -> None:
        result = self.result(evidence([disposition()]))
        self.assertEqual(result["decision"], "allowed")
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
            validate_instance(_load_schema(self.repo), payload)

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


if __name__ == "__main__":
    unittest.main()
