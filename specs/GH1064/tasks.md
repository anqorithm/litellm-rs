# Task Plan

## Linked Issue

GH-1064 / #1064

## Spec Packet

- Product: `specs/GH1064/product.md`
- Tech: `specs/GH1064/tech.md`

## Implementation Tasks

- [x] `SP1064-T1` Covers: B-001, B-002, B-003, B-004, B-005. Owner: coordinator. Dependencies: merged roadmap PR #1068. Done when: the product and technical specs define #1064 as a planning umbrella, keep child implementation out of scope, and give parent closure an explicit meaning. Evidence: PR #1084 merged as `e1e2907e3e1e62da733901eaf751dca1faaa21fd`. Verify: `python3 checks/check_workflow.py --repo . --spec-dir "$PWD/specs/GH1064" && sed -n '8,17p' docs/plan/2026-07-18-best-gateway-gap-analysis.md`.
- [x] `SP1064-T2` Covers: B-002, B-003, B-005. Owner: coordinator. Dependencies: `SP1064-T1`. Done when: the roadmap records delivery, focused ownership, and closure semantics without changing a child spec, issue, or production file. Evidence: PR #1084 changed exactly the roadmap and three GH1064 packet files. Verify: `test "$(git diff --name-only e1e2907e3e1e62da733901eaf751dca1faaa21fd^1 e1e2907e3e1e62da733901eaf751dca1faaa21fd | sort | paste -sd, -)" = "docs/plan/2026-07-18-best-gateway-gap-analysis.md,specs/GH1064/product.md,specs/GH1064/tasks.md,specs/GH1064/tech.md"`.
- [ ] `SP1064-T3` Covers: B-001, B-002, B-003, B-004, B-005. Owner: coordinator and independent reviewer. Dependencies: `SP1064-T1`, `SP1064-T2`. Done when: the coordinator-authored structural reconciliation PR body references #1068/#1084 and uses `Fixes #1064`; deterministic checks pass; a new independent review, current-head CI, resolved threads, clean merge state, and required PR gate all bind the same exact head; the PR merges; then the runtime ledger records T3 complete and a separate issue-closure audit records `issue=1064`, the exact PR/head/merge SHA, `pr_state=MERGED`, `issue_state=CLOSED`, query time, and an empty `linked_issue_mutations` list without modifying the merged head. The repository checkbox intentionally remains unchecked to avoid self-reference. Verify: `test -n "$PR_EVIDENCE" && test -n "$CHECKPOINT" && test -n "$CLOSURE_AUDIT" && python3 checks/pr_gate.py --repo . --evidence "$PR_EVIDENCE" --mode required --json && python3 checks/runtime_ledger_gate.py --checkpoint "$CHECKPOINT" --json && jq -e '.issue == 1064 and .pr_state == "MERGED" and .issue_state == "CLOSED" and ((.pr | type) == "number") and (.head_sha | test("^[0-9a-f]{40}$")) and (.merge_sha | test("^[0-9a-f]{40}$")) and ((.queried_at | type) == "string") and ((.queried_at | length) > 0) and .linked_issue_mutations == []' "$CLOSURE_AUDIT" && test "$(gh issue view 1064 --repo majiayu000/litellm-rs --json state --jq .state)" = "CLOSED"`.

## Parallelization

The three packet edits are one coordinator-owned serial lane. The independent
reviewer is read-only and starts only after the new PR head is stable. PR-body
linkage, runtime checkpoint updates, and the post-merge issue audit are
coordinator-owned external evidence; they do not edit the repository head.

## Verification

- Product invariant set: `B-001..B-005`.
- Task `Covers:` union: `B-001..B-005`; no missing or unknown ID.
- The tech spec contains exactly one complete issue-1064 manifest with four
  paths and refs `B-001..B-005`.
- Run `python3 checks/check_workflow.py --repo . --spec-dir "$PWD/specs/GH1064"`.
- Run `git diff --check`.
- Run the `B-005` changed-path command before opening the docs-only PR.

## Handoff Notes

- PR #1068 delivered the roadmap; PR #1084 delivered lifecycle wording and
  resolved its two inline comments, but did not obtain independent final-head
  review. Its review evidence must not be reused.
- The new PR is docs-only structural reconciliation. Its coordinator-owned body
  handles `Refs #1068`, `Refs #1084`, and `Fixes #1064`; this branch does not
  modify the roadmap or child specs.
- `SP1064-T3` remains `[ ]`. After merge, completion lives in the runtime ledger
  and issue-closure audit so the exact reviewed head is never changed to tick
  its own completion box.
