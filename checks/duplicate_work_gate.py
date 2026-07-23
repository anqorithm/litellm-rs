#!/usr/bin/env python3
"""Evaluate duplicate implementation work evidence offline."""

from __future__ import annotations

import argparse
import json
import re
import subprocess
import sys
from datetime import datetime, timedelta, timezone
from pathlib import Path
from typing import Any

from specrail_lib import (
    PackConfig,
    SpecRailError,
    artifact_templates,
    load_pack,
    read_text,
    validate_instance,
)


SEGMENT_SPLIT_RE = re.compile(r"[/-]+")
PLACEHOLDER_RE = re.compile(r"\{[^}]+\}")
SHA_RE = re.compile(r"^[0-9a-f]{40}$")
EVIDENCE_MAX_AGE = timedelta(minutes=15)


def _positive_issue(value: int | None) -> bool:
    return isinstance(value, int) and value > 0


def _load_schema(repo: Path) -> dict[str, Any]:
    path = repo / "schemas" / "duplicate_work_evidence.schema.json"
    try:
        data = json.loads(read_text(path))
    except json.JSONDecodeError as exc:
        raise SpecRailError(f"{path.relative_to(repo)}: invalid JSON: {exc.msg}") from exc
    if not isinstance(data, dict):
        raise SpecRailError("duplicate work evidence schema must be an object")
    return data


def _legacy_validator_schema(value: Any) -> Any:
    """Strip pattern only; SHA syntax is enforced by this gate below."""
    if isinstance(value, dict):
        return {
            key: _legacy_validator_schema(item)
            for key, item in value.items()
            if key != "pattern"
        }
    if isinstance(value, list):
        return [_legacy_validator_schema(item) for item in value]
    return value


def _blocked(reason: str) -> dict[str, Any]:
    return {"decision": "blocked", "reasons": [reason], "missing": []}


def _run_git(repo: Path, args: list[str]) -> subprocess.CompletedProcess[str]:
    try:
        return subprocess.run(
            ["git", "-C", str(repo), *args],
            check=False,
            capture_output=True,
            text=True,
        )
    except FileNotFoundError as exc:
        raise SpecRailError("git executable was not found in PATH") from exc


def _git_output(repo: Path, args: list[str], label: str) -> str:
    completed = _run_git(repo, args)
    if completed.returncode != 0:
        detail = completed.stderr.strip() or completed.stdout.strip() or "no output"
        raise SpecRailError(f"{label} failed: {detail}")
    return completed.stdout.strip()


def revalidate_remote_state(
    repo: Path,
    config: PackConfig,
    issue: int,
    evidence: dict[str, Any],
) -> dict[str, Any] | None:
    token = impl_branch_token(config, issue)
    if token is None:
        return _blocked("cannot revalidate remote branches without an implementation token")
    try:
        remote_output = _git_output(
            repo,
            ["ls-remote", "--heads", "origin"],
            "gate-time git ls-remote",
        )
        remote_branches: list[dict[str, str]] = []
        for line in remote_output.splitlines():
            sha, sep, ref = line.partition("\t")
            if not sep or not ref.startswith("refs/heads/") or not SHA_RE.fullmatch(sha):
                return _blocked("gate-time remote branch output is malformed")
            remote_branches.append(
                {"name": ref.removeprefix("refs/heads/"), "head_sha": sha}
            )
        remote_heads = {branch["name"]: branch["head_sha"] for branch in remote_branches}
        if len(remote_heads) != len(remote_branches):
            return _blocked("gate-time remote branch output contains duplicate names")
        base_ref = evidence["base_ref"]
        base_sha = evidence["base_sha"]
        remote_base = remote_heads.get(base_ref)
        if remote_base is None:
            return _blocked(f"gate-time remote base branch is missing: {base_ref}")
        local_base = _git_output(
            repo,
            ["rev-parse", "--verify", f"refs/remotes/origin/{base_ref}"],
            "local remote-tracking base lookup",
        )
        if not SHA_RE.fullmatch(local_base):
            return _blocked("local remote-tracking base SHA is malformed")
        if len({base_sha, remote_base, local_base}) != 1:
            return _blocked(
                "base drift: evidence, gate-time remote, and local origin base differ"
            )
        evidence_matching = matching_contract_branches(
            evidence["remote_branches"], token
        )
        remote_matching = matching_contract_branches(remote_branches, token)
        if evidence_matching != remote_matching:
            return _blocked(
                "matching implementation branch set or head changed since evidence collection"
            )
        for branch in remote_matching:
            label = f"{branch['name']}@{branch['head_sha']}"
            exists = _run_git(repo, ["cat-file", "-e", f"{branch['head_sha']}^{{commit}}"])
            if exists.returncode != 0:
                return _blocked(f"matching branch commit object is missing locally: {label}")
            ancestor = _run_git(
                repo,
                ["merge-base", "--is-ancestor", base_sha, branch["head_sha"]],
            )
            if ancestor.returncode != 0:
                return _blocked(
                    f"matching branch is not descended from evidence base: {label}"
                )
    except (KeyError, SpecRailError) as exc:
        return _blocked(f"gate-time remote revalidation failed: {exc}")
    return None


def _load_evidence(path: Path | None) -> dict[str, Any] | None:
    if path is None or not path.is_file():
        return None
    try:
        data = json.loads(path.read_text(encoding="utf-8"))
    except json.JSONDecodeError as exc:
        raise SpecRailError(f"invalid duplicate work evidence JSON {path}: {exc.msg}") from exc
    except OSError as exc:
        raise SpecRailError(f"cannot read duplicate work evidence {path}: {exc}") from exc
    if not isinstance(data, dict):
        raise SpecRailError("duplicate work evidence JSON must be an object")
    return data


def impl_branch_token(config: PackConfig, issue: int) -> str | None:
    template = artifact_templates(config).get("impl_branch")
    if not template or "{issue_number}" not in template:
        return None
    for segment in SEGMENT_SPLIT_RE.split(template):
        if "{issue_number}" not in segment:
            continue
        token = segment.replace("{issue_number}", str(issue))
        token = PLACEHOLDER_RE.sub("", token).strip()
        if token:
            return token.lower()
    return None


def branch_segments(branch: str) -> set[str]:
    return {segment.lower() for segment in SEGMENT_SPLIT_RE.split(branch) if segment}


def matching_contract_branches(branches: list[dict[str, Any]], token: str) -> list[dict[str, str]]:
    wanted = token.lower()
    return sorted(
        [
            {"name": str(branch["name"]), "head_sha": str(branch["head_sha"])}
            for branch in branches
            if wanted in branch_segments(str(branch["name"]))
        ],
        key=lambda branch: branch["name"],
    )


def evaluate_retained_branch_dispositions(
    evidence: dict[str, Any],
    matching_branches: list[dict[str, str]],
) -> dict[str, Any] | None:
    remote_branches = evidence["remote_branches"]
    dispositions = evidence["retained_branch_dispositions"]
    if any(not SHA_RE.fullmatch(branch["head_sha"]) for branch in remote_branches):
        return {
            "decision": "blocked",
            "reasons": ["remote branch evidence contains an invalid head SHA"],
            "missing": [],
        }
    if any(not SHA_RE.fullmatch(disposition["head_sha"]) for disposition in dispositions):
        return {
            "decision": "blocked",
            "reasons": ["retained branch disposition contains an invalid head SHA"],
            "missing": [],
        }
    remote_heads = {branch["name"]: branch["head_sha"] for branch in remote_branches}
    if len(remote_heads) != len(remote_branches):
        return {
            "decision": "blocked",
            "reasons": ["duplicate remote branch names in duplicate work evidence"],
            "missing": [],
        }

    matching_names = {branch["name"] for branch in matching_branches}
    indexed: dict[str, list[dict[str, Any]]] = {}
    for disposition in dispositions:
        branch = disposition["branch"]
        if branch not in remote_heads:
            return {
                "decision": "blocked",
                "reasons": [f"retained branch disposition references absent remote branch {branch}"],
                "missing": [],
            }
        if branch not in matching_names:
            return {
                "decision": "blocked",
                "reasons": [f"retained branch disposition is outside this issue contract: {branch}"],
                "missing": [],
            }
        indexed.setdefault(branch, []).append(disposition)

    for branch in matching_branches:
        name = branch["name"]
        head_sha = branch["head_sha"]
        entries = indexed.get(name, [])
        label = f"{name}@{head_sha}"
        if not entries:
            return {
                "decision": "needs_human",
                "reasons": [f"retained implementation branch requires a human disposition: {label}"],
                "missing": [f"retained_branch_disposition:{label}"],
            }
        if len(entries) != 1:
            return {
                "decision": "blocked",
                "reasons": [f"duplicate retained branch dispositions for {label}"],
                "missing": [],
            }
        entry = entries[0]
        if entry["head_sha"] != head_sha:
            return {
                "decision": "needs_human",
                "reasons": [f"retained branch disposition SHA is stale or mismatched for {label}"],
                "missing": [f"fresh_retained_branch_disposition:{label}"],
            }
        if entry["disposition"] == "active_duplicate":
            return {
                "decision": "blocked",
                "reasons": [f"retained branch is an active duplicate: {label}"],
                "missing": [],
            }
        if entry["disposition"] != "retained_non_conflicting":
            return {
                "decision": "needs_human",
                "reasons": [f"retained branch disposition does not permit implementation: {label}"],
                "missing": [f"permitting_retained_branch_disposition:{label}"],
            }
        source = entry["source"]
        if (
            source["kind"] != "maintainer_human"
            or not source["reference"].strip()
            or not source["recorded_at"].strip()
        ):
            return {
                "decision": "needs_human",
                "reasons": [f"retained branch disposition lacks maintainer source evidence: {label}"],
                "missing": [f"maintainer_source:{label}"],
            }
    if matching_branches:
        return {
            "decision": "needs_human",
            "reasons": [
                "maintainer_human retained-branch dispositions are audit evidence "
                "and do not authorize implementation"
            ],
            "missing": ["independent_implementation_authorization"],
        }
    return None


def evaluate_duplicate_work_gate(
    config: PackConfig,
    issue: int | None,
    evidence: dict[str, Any] | None,
) -> dict[str, Any]:
    reasons: list[str] = []
    satisfied: list[str] = []
    missing: list[str] = []

    if not _positive_issue(issue):
        return {
            "decision": "blocked",
            "issue": issue,
            "reasons": ["duplicate work gate requires a positive issue number"],
            "satisfied": [],
            "missing": ["issue"],
            "blocked_actions": ["implement"],
            "verification_commands": ["python3 checks/duplicate_work_gate.py --repo . --issue <issue> --evidence <evidence.json>"],
        }

    if evidence is None:
        return {
            "decision": "needs_human",
            "issue": issue,
            "reasons": ["duplicate work evidence is missing"],
            "satisfied": [],
            "missing": ["duplicate_evidence"],
            "blocked_actions": ["implement"],
            "verification_commands": ["python3 checks/github_duplicate_evidence.py --github-repo OWNER/REPO --issue <issue> --json"],
        }

    try:
        validate_instance(_legacy_validator_schema(_load_schema(config.repo)), evidence)
    except SpecRailError as exc:
        return {
            "decision": "blocked",
            "issue": issue,
            "reasons": [f"duplicate work evidence schema validation failed: {exc}"],
            "satisfied": [],
            "missing": [],
            "blocked_actions": ["implement"],
            "verification_commands": ["python3 checks/duplicate_work_gate.py --repo . --issue <issue> --evidence <evidence.json>"],
        }

    if evidence.get("issue") != issue:
        return {
            "decision": "blocked",
            "issue": issue,
            "reasons": [f"duplicate work evidence issue mismatch: expected {issue}, got {evidence.get('issue')}"],
            "satisfied": [],
            "missing": [],
            "blocked_actions": ["implement"],
            "verification_commands": ["python3 checks/duplicate_work_gate.py --repo . --issue <issue> --evidence <evidence.json>"],
        }

    for field in ("base_sha",):
        if not SHA_RE.fullmatch(str(evidence.get(field, ""))):
            return {
                "decision": "blocked",
                "issue": issue,
                "reasons": [f"duplicate work evidence contains invalid {field}"],
                "satisfied": [],
                "missing": [],
                "blocked_actions": ["implement"],
                "verification_commands": ["python3 checks/duplicate_work_gate.py --repo . --issue <issue> --evidence <evidence.json>"],
            }
    for collection, field in (
        ("remote_branches", "head_sha"),
        ("retained_branch_dispositions", "head_sha"),
    ):
        if any(
            not SHA_RE.fullmatch(str(item.get(field, "")))
            for item in evidence[collection]
        ):
            return {
                "decision": "blocked",
                "issue": issue,
                "reasons": [f"duplicate work evidence contains invalid {collection} SHA"],
                "satisfied": [],
                "missing": [],
                "blocked_actions": ["implement"],
                "verification_commands": ["python3 checks/duplicate_work_gate.py --repo . --issue <issue> --evidence <evidence.json>"],
            }
    try:
        collected_at = datetime.fromisoformat(
            str(evidence["collected_at"]).replace("Z", "+00:00")
        )
    except (TypeError, ValueError):
        return {
            "decision": "blocked",
            "issue": issue,
            "reasons": ["duplicate work evidence collected_at is invalid"],
            "satisfied": [],
            "missing": [],
            "blocked_actions": ["implement"],
            "verification_commands": ["python3 checks/github_duplicate_evidence.py --github-repo OWNER/REPO --issue <issue> --json"],
        }
    now = datetime.now(timezone.utc)
    if collected_at.tzinfo is None or collected_at > now or now - collected_at > EVIDENCE_MAX_AGE:
        return {
            "decision": "blocked",
            "issue": issue,
            "reasons": ["duplicate work evidence is stale or future-dated"],
            "satisfied": [],
            "missing": [],
            "blocked_actions": ["implement"],
            "verification_commands": ["python3 checks/github_duplicate_evidence.py --github-repo OWNER/REPO --issue <issue> --json"],
        }

    duplicate_prs = [
        item["number"]
        for item in evidence["open_prs"]
        if item.get("references_issue") is True
    ]
    if duplicate_prs:
        joined = ", ".join(f"#{number}" for number in sorted(duplicate_prs))
        reasons.append(f"open PRs already reference GH-{issue}: {joined}")
        return {
            "decision": "blocked",
            "issue": issue,
            "reasons": reasons,
            "satisfied": satisfied,
            "missing": missing,
            "blocked_actions": ["implement"],
            "verification_commands": ["python3 checks/duplicate_work_gate.py --repo . --issue <issue> --evidence <evidence.json>"],
        }
    satisfied.append(f"no open PR references GH-{issue}")

    if evidence.get("open_prs_complete") is not True:
        limit = evidence.get("open_pr_limit")
        reasons.append(
            "open PR evidence may be incomplete"
            + (f" at collection limit {limit}" if isinstance(limit, int) else "")
        )
        return {
            "decision": "needs_human",
            "issue": issue,
            "reasons": reasons,
            "satisfied": satisfied,
            "missing": ["complete_open_pr_evidence"],
            "blocked_actions": ["implement"],
            "verification_commands": ["python3 checks/github_duplicate_evidence.py --github-repo OWNER/REPO --issue <issue> --pr-limit <larger-limit> --json"],
        }

    token = impl_branch_token(config, issue)
    if token is None:
        return {
            "decision": "needs_human",
            "issue": issue,
            "reasons": ["workflow.yaml artifacts.impl_branch is missing or lacks {issue_number}"],
            "satisfied": satisfied,
            "missing": ["artifacts.impl_branch"],
            "blocked_actions": ["implement"],
            "verification_commands": ["python3 checks/check_workflow.py --repo ."],
        }

    branches = matching_contract_branches(evidence["remote_branches"], token)
    if branches:
        disposition_result = evaluate_retained_branch_dispositions(evidence, branches)
        if disposition_result is not None:
            return {
                "decision": disposition_result["decision"],
                "issue": issue,
                "reasons": reasons + disposition_result["reasons"],
                "satisfied": satisfied,
                "missing": disposition_result["missing"],
                "blocked_actions": ["implement"],
                "verification_commands": ["python3 checks/duplicate_work_gate.py --repo . --issue <issue> --evidence <evidence.json>"],
            }
        satisfied.append(
            "all retained implementation branches have exact maintainer dispositions"
        )

    else:
        satisfied.append(f"no remote branch matches implementation token {token}")
    return {
        "decision": "allowed",
        "issue": issue,
        "reasons": [f"duplicate work gate passed for GH-{issue}"],
        "satisfied": satisfied,
        "missing": [],
        "blocked_actions": [],
        "verification_commands": ["python3 checks/duplicate_work_gate.py --repo . --issue <issue> --evidence <evidence.json>"],
    }


def evaluate_duplicate_work_gate_path(
    repo: Path,
    issue: int | None,
    evidence_path: Path | None,
) -> dict[str, Any]:
    config = load_pack(repo)
    try:
        evidence = _load_evidence(evidence_path)
    except SpecRailError as exc:
        return {
            "decision": "blocked",
            "issue": issue,
            "reasons": [str(exc)],
            "satisfied": [],
            "missing": [],
            "blocked_actions": ["implement"],
            "verification_commands": ["python3 checks/duplicate_work_gate.py --repo . --issue <issue> --evidence <evidence.json>"],
        }
    if evidence is not None and _positive_issue(issue):
        remote_result = revalidate_remote_state(repo, config, issue, evidence)
        if remote_result is not None:
            return {
                "decision": remote_result["decision"],
                "issue": issue,
                "reasons": remote_result["reasons"],
                "satisfied": [],
                "missing": remote_result["missing"],
                "blocked_actions": ["implement"],
                "verification_commands": ["python3 checks/github_duplicate_evidence.py --github-repo OWNER/REPO --issue <issue> --json"],
            }
    return evaluate_duplicate_work_gate(config, issue, evidence)


def print_human(result: dict[str, Any]) -> None:
    print(f"decision: {result['decision']}")
    if result.get("issue"):
        print(f"issue: GH-{result['issue']}")
    if result.get("reasons"):
        print("reasons:")
        for reason in result["reasons"]:
            print(f"- {reason}")
    if result.get("missing"):
        print("missing:")
        for item in result["missing"]:
            print(f"- {item}")


def main() -> int:
    parser = argparse.ArgumentParser(
        description="Evaluate duplicate implementation work evidence offline."
    )
    parser.add_argument("--repo", default=".", help="SpecRail pack or adopted repo root")
    parser.add_argument("--issue", type=int, required=True, help="Linked GitHub issue number")
    parser.add_argument("--evidence", help="Duplicate work evidence JSON file")
    parser.add_argument("--json", action="store_true", help="Print JSON output")
    args = parser.parse_args()

    result = evaluate_duplicate_work_gate_path(
        Path(args.repo).resolve(),
        args.issue,
        Path(args.evidence) if args.evidence else None,
    )

    if args.json:
        print(json.dumps(result, indent=2, sort_keys=True))
    else:
        print_human(result)

    if result["decision"] == "blocked":
        return 1
    return 0


if __name__ == "__main__":
    sys.exit(main())
