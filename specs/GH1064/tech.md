# Tech Spec

## Linked Issue

GH-1064 / #1064

## Product Spec

See `specs/GH1064/product.md`.

## Codebase Context

Anchors below were verified against `origin/main@bdc06ba364e8b9e095bb91f1c75572a624a954f5`.

| Area | Verified anchor | Current behavior | Why relevant |
| --- | --- | --- | --- |
| Roadmap method | `docs/plan/2026-07-18-best-gateway-gap-analysis.md:3-6,140-159` | The roadmap names its evidence baseline and separates facts, inference, and recommendations. | Grounds `B-001` without re-running the survey. |
| Roadmap lifecycle | `docs/plan/2026-07-18-best-gateway-gap-analysis.md:8-17` | PR #1068 delivery, focused ownership, parent closure semantics, and future focused-issue routing are recorded. | Grounds `B-002` through `B-004`. |
| Gap ownership | `docs/plan/2026-07-18-best-gateway-gap-analysis.md:102-110` | New gaps map to #1065-#1067 and existing gaps remain with #519/#837/#838/#965. | Prevents umbrella implementation scope. |
| Product packet | `specs/GH1064/product.md:33-55` at the baseline | Five invariants and acceptance criteria exist, but the invariants lack stable `B-xxx` IDs and the ten-category boundary checklist. | Structural reconciliation only. |
| Tech packet | `specs/GH1064/tech.md:32-40` at the baseline | Mapping uses unstable legacy labels and no planned-changes manifest exists. | Requires stable refs and one fail-closed manifest. |
| Task packet | `specs/GH1064/tasks.md:14-16` at the baseline | Stable T1-T3 exist, with T1/T2 complete and T3 open, but fields and verification are not machine-precise. | IDs and meanings must be preserved. |

## Historical and Current Evidence

- PR #1084 merged at final head
  `1d5502fdba342a53df8f766720d1affa4e344384` as merge commit
  `e1e2907e3e1e62da733901eaf751dca1faaa21fd`.
- Its exact changed paths were the roadmap and the three GH1064 packet files;
  no production or child-spec file changed.
- The two GitHub inline comments were resolved and outdated on the final head.
  GitHub's only submitted review was on old head
  `88a20e8dfaa9435d573241bd7e86814faf4c3cd9`. Separately, the current `implx`
  run performed an independent post-merge exact-head review of
  `1d5502fdba342a53df8f766720d1affa4e344384`; its verdict was `FAIL` with
  structural findings: missing stable `B-xxx` IDs, missing ten-category boundary
  coverage, missing single complete planned-changes manifest, missing per-B
  mapping, and missing task `Covers:` fields. PR #1084 therefore has no clean,
  reusable exact-head `PASS`; the failed review remains durable evidence and is
  not erased by this reconciliation.
- Fresh issue evidence on 2026-07-19 reports #1064 as `OPEN`. Historical merge
  and resolved comments do not satisfy the new PR's review, gate, merge, or
  closure requirements.

## Planned Changes

This is the complete GH1064 lifecycle path set. The structural reconciliation
PR is narrower and may change only the three packet files; the roadmap is
listed because it is the durable artifact already delivered by PR #1068 and
updated by PR #1084.

```specrail-planned-changes
{
  "issue": 1064,
  "complete": true,
  "paths": [
    "docs/plan/2026-07-18-best-gateway-gap-analysis.md",
    "specs/GH1064/product.md",
    "specs/GH1064/tech.md",
    "specs/GH1064/tasks.md"
  ],
  "spec_refs": [
    "B-001",
    "B-002",
    "B-003",
    "B-004",
    "B-005"
  ]
}
```

## Proposed Design

Reconcile the existing packet without changing product scope:

1. Assign stable `B-001` through `B-005` to the five published invariants and
   record every boundary category as covered or N/A.
2. Bind the technical mapping and task coverage to those same IDs.
3. Preserve `SP1064-T1` through `SP1064-T3`, their meanings, and their truth:
   T1/T2 remain complete from PR #1084; T3 remains open.
4. Treat this new PR as docs-only structural reconciliation. Its body owns
   `Refs`/`Fixes`; this commit does not modify the roadmap or child packets.
5. After the new PR head is frozen, collect a new independent exact-head review,
   required PR-gate evidence, and merge evidence. After merge, record T3 in the
   runtime ledger and a separate issue-closure audit without another head edit.

## Product-to-Test Mapping

| Product invariant | Implementation area | Verification |
| --- | --- | --- |
| `B-001` durable classified roadmap | Roadmap method and sections | `rg -n '^> 方法：|^### 事实|^### 推断$|^### 建议' docs/plan/2026-07-18-best-gateway-gap-analysis.md` |
| `B-002` focused ownership | Roadmap lifecycle and ownership table | `sed -n '8,17p;102,110p' docs/plan/2026-07-18-best-gateway-gap-analysis.md` plus manual confirmation that each implementation-sized row names an issue. |
| `B-003` parent closure semantics | Roadmap lifecycle, new PR body, post-merge audit | `sed -n '8,17p' docs/plan/2026-07-18-best-gateway-gap-analysis.md`; after merge, run the complete `verify_t3_closure.sh` command block below. |
| `B-004` future discoveries use focused work | Roadmap lifecycle | `rg -n 'Future implementation-sized discoveries' docs/plan/2026-07-18-best-gateway-gap-analysis.md` |
| `B-005` documentation-only change | Reconciliation diff scope | `base="$(git merge-base origin/main HEAD)"; test -z "$(git diff --name-only "$base"..HEAD -- . ':(exclude)specs/GH1064/product.md' ':(exclude)specs/GH1064/tech.md' ':(exclude)specs/GH1064/tasks.md')"` |

## Data Flow

There is no gateway runtime data flow. Repository roadmap and packet content
define the planning contract; the new PR carries linkage and exact-head review
evidence; after merge, the runtime checkpoint records T3 completion and a
read-only issue audit records remote closure. Those post-merge artifacts do not
modify the merged head.

## Alternatives Considered

- Mark T3 complete in this packet: rejected because review, PR gate, merge, and
  closure have not happened for the new head and changing the checkbox after
  them would create a self-referential head.
- Reuse PR #1084 review: rejected because GitHub's submitted review targeted an
  older head and the independent post-merge exact-head review returned `FAIL`.
  Resolved/outdated threads and this structural repair do not convert or erase
  that failure.
- Modify the roadmap or child specs again: rejected because reconciliation is
  structural and their content remains owned by prior/focused work.

## Risks

- Security: none; no runtime or security behavior changes.
- Compatibility: parent closure can be misread as child completion; stable
  invariants and the closure audit prevent that claim.
- Performance: none.
- Maintenance: external ledger/audit evidence can drift from repository prose;
  evidence must bind the new PR exact head and merge SHA.

## Test Plan

- [x] Deterministic packet check:
      `python3 checks/check_workflow.py --repo . --spec-dir "$PWD/specs/GH1064"`
      passed in the reconciliation session.
- [x] Diff syntax and reconciliation path scope:
      `git diff --check` and the `B-005` command passed in the reconciliation
      session.
- [x] Historical path evidence: PR #1084 merge commit
      `e1e2907e3e1e62da733901eaf751dca1faaa21fd` changes exactly the roadmap and
      three GH1064 packet files.
- [x] Historical review-comment resolution: both PR #1084 inline threads are
      resolved and outdated at final head `1d5502fdba342a53df8f766720d1affa4e344384`.
- [x] Historical exact-head review truth: the current `implx` run's independent
      post-merge review of that final head returned `FAIL` with the five packet
      structure findings listed above; no clean/reusable `PASS` exists.
- [ ] A new independent reviewer has returned a clean verdict bound to the
      structural reconciliation PR exact head.
- [ ] Fresh required `pr_gate.py` evidence for that exact head is allowed.
- [ ] The new PR is merged and post-merge runtime-ledger plus issue-closure
      audit evidence is complete.

### `verify_t3_closure.sh` canonical command block

Run this complete block from the repository root after merge. `PR_EVIDENCE`,
`CHECKPOINT`, and `CLOSURE_AUDIT` are coordinator-owned external JSON artifacts;
any missing field, duplicate issue-1064 checkpoint item, mismatch, API failure,
or nonzero exit blocks T3.

```bash
set -euo pipefail

: "${PR_EVIDENCE:?set PR_EVIDENCE to post-merge PR evidence JSON}"
: "${CHECKPOINT:?set CHECKPOINT to the runtime checkpoint JSON}"
: "${CLOSURE_AUDIT:?set CLOSURE_AUDIT to the post-merge issue audit JSON}"

expected_pr="$(jq -er '.pr | select(type == "number" and . > 0)' "$CLOSURE_AUDIT")"
expected_head="$(jq -er '.head_sha | select(type == "string" and test("^[0-9a-f]{40}$"))' "$CLOSURE_AUDIT")"
expected_merge="$(jq -er '.merge_sha | select(type == "string" and test("^[0-9a-f]{40}$"))' "$CLOSURE_AUDIT")"

jq -e --argjson pr "$expected_pr" --arg head "$expected_head" --arg merge "$expected_merge" '
  .issue == 1064
  and .pr == $pr
  and .head_sha == $head
  and .merge_sha == $merge
  and .pr_state == "MERGED"
  and .issue_state == "CLOSED"
  and .closing_issue_numbers == [1064]
  and ((.queried_at | type) == "string")
  and ((.queried_at | length) > 0)
' "$CLOSURE_AUDIT" >/dev/null

python3 checks/pr_gate.py --repo . --evidence "$PR_EVIDENCE" --mode required --json
python3 checks/runtime_ledger_gate.py --checkpoint "$CHECKPOINT" --json

jq -e --argjson pr "$expected_pr" --arg head "$expected_head" --arg merge "$expected_merge" '
  .pr == $pr
  and .head_sha == $head
  and (
    .merge_record
    | if . then
        if type == "object" then
          (.remote_confirmed == true and .merge_commit_sha == $merge)
        else
          false
        end
      else
        false
      end
  )
' "$PR_EVIDENCE" >/dev/null

jq -e --argjson pr "$expected_pr" --arg head "$expected_head" --arg merge "$expected_merge" '
  (if .items then
    [
      .items[]?
      | select(if type == "object" then .issue == 1064 else false end)
    ]
  else
    []
  end) as $matches
  | ($matches[0]? // {}) as $match
  | ($matches | length) == 1
    and $match.pr == $pr
    and $match.head_sha == $head
    and $match.merge_commit == $merge
    and $match.state == "merged"
' "$CHECKPOINT" >/dev/null

live_pr="$(
  gh pr view "$expected_pr" \
    --repo majiayu000/litellm-rs \
    --json number,state,headRefOid,mergeCommit,body,closingIssuesReferences
)"
jq -e --argjson pr "$expected_pr" --arg head "$expected_head" --arg merge "$expected_merge" '
  .number == $pr
  and .state == "MERGED"
  and .headRefOid == $head
  and (
    .mergeCommit
    | if type == "object" then .oid == $merge else false end
  )
  and (
    .body
    | if type == "string" then
        (
          contains("Refs #1068")
          and contains("Refs #1084")
          and contains("Fixes #1064")
        )
      else
        false
      end
  )
  and (
    .closingIssuesReferences
    | if type == "array" then
        (
          [
            .[]?
            | if type == "object" then
                (.number | select(type == "number"))
              else
                empty
              end
          ]
          | sort
        ) == [1064]
      else
        false
      end
  )
' <<<"$live_pr" >/dev/null

live_issue="$(
  gh issue view 1064 \
    --repo majiayu000/litellm-rs \
    --json number,state
)"
jq -e '.number == 1064 and .state == "CLOSED"' <<<"$live_issue" >/dev/null
```

## Rollback Plan

Revert only the structural reconciliation commit if the packet format is
incorrect. The roadmap and PR #1084 history remain intact; reopening or closing
#1064 is a separate maintainer action backed by the closure audit.
