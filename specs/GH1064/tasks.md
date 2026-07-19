# Task Plan

## Linked Issue

GH-1064 / #1064

## Spec Packet

- Product: `specs/GH1064/product.md`
- Tech: `specs/GH1064/tech.md`

## Implementation Tasks

- [x] `SP1064-T1` Covers: B-001, B-002, B-003, B-004, B-005. Owner: coordinator. Dependencies: merged roadmap PR #1068. Done when: the product and technical specs define #1064 as a planning umbrella, keep child implementation out of scope, and give parent closure an explicit meaning. Evidence: PR #1084 merged as `e1e2907e3e1e62da733901eaf751dca1faaa21fd`. Verify: `python3 checks/check_workflow.py --repo . --spec-dir "$PWD/specs/GH1064" && sed -n '8,17p' docs/plan/2026-07-18-best-gateway-gap-analysis.md`.
- [x] `SP1064-T2` Covers: B-002, B-003, B-005. Owner: coordinator. Dependencies: `SP1064-T1`. Done when: the roadmap records delivery, focused ownership, and closure semantics without changing a child spec, issue, or production file. Evidence: PR #1084 changed exactly the roadmap and three GH1064 packet files. Verify: `test "$(git diff --name-only e1e2907e3e1e62da733901eaf751dca1faaa21fd^1 e1e2907e3e1e62da733901eaf751dca1faaa21fd | sort | paste -sd, -)" = "docs/plan/2026-07-18-best-gateway-gap-analysis.md,specs/GH1064/product.md,specs/GH1064/tasks.md,specs/GH1064/tech.md"`.
- [ ] `SP1064-T3` Covers: B-001, B-002, B-003, B-004, B-005. Owner: coordinator and independent reviewer. Dependencies: `SP1064-T1`, `SP1064-T2`. Done when: the coordinator-authored structural reconciliation PR body contains `Refs #1068`, `Refs #1084`, and `Fixes #1064`; deterministic checks pass; a new independent review, current-head CI, resolved threads, clean merge state, and required PR gate all bind one exact head; the PR merges; then the runtime ledger's sole issue-1064 item records the same PR/head/merge SHA with `state=merged`, while a separate issue-closure audit records those same identifiers, `issue_state=CLOSED`, `closing_issue_numbers=[1064]`, and query time. Live PR and issue evidence must match every identifier, body directive, merge state, and the exact closing-issue set. The repository checkbox intentionally remains unchecked so post-merge evidence never changes the reviewed head. Verify: run the complete executable `verify_t3_closure.sh` canonical command block in `tech.md` with `$PR_EVIDENCE`, `$CHECKPOINT`, and `$CLOSURE_AUDIT` set; any nonzero exit blocks completion.

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

- PR #1068 delivered the roadmap. For PR #1084, GitHub's only submitted review
  targeted old head `88a20e8dfaa9435d573241bd7e86814faf4c3cd9`;
  separately, the current `implx` run independently reviewed final head
  `1d5502fdba342a53df8f766720d1affa4e344384` post-merge and returned `FAIL`
  for the five packet-structure findings. There is no clean/reusable `PASS`,
  and the failed exact-head review must remain recorded after repair.
- The new PR is docs-only structural reconciliation. Its coordinator-owned body
  handles `Refs #1068`, `Refs #1084`, and `Fixes #1064`; this branch does not
  modify the roadmap or child specs.
- `SP1064-T3` remains `[ ]`. After merge, completion lives in the runtime ledger
  and issue-closure audit so the exact reviewed head is never changed to tick
  its own completion box.
