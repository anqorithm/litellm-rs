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
- The two inline comments were resolved and outdated on the final head. The only
  submitted review was on old head `88a20e8dfaa9435d573241bd7e86814faf4c3cd9`,
  and the Codex reviewer lane reported a usage-limit failure. Therefore PR #1084
  supplies no reusable independent exact-head PASS.
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
| `B-003` parent closure semantics | Roadmap lifecycle, new PR body, post-merge audit | `sed -n '8,17p' docs/plan/2026-07-18-best-gateway-gap-analysis.md`; after merge, verify `$CLOSURE_AUDIT` records `issue_state=CLOSED` and no linked-issue mutation. |
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
- Reuse PR #1084 review: rejected because its only review targeted an older head
  and the independent Codex reviewer lane failed.
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
- [ ] A new independent reviewer has returned a clean verdict bound to the
      structural reconciliation PR exact head.
- [ ] Fresh required `pr_gate.py` evidence for that exact head is allowed.
- [ ] The new PR is merged and post-merge runtime-ledger plus issue-closure
      audit evidence is complete.

## Rollback Plan

Revert only the structural reconciliation commit if the packet format is
incorrect. The roadmap and PR #1084 history remain intact; reopening or closing
#1064 is a separate maintainer action backed by the closure audit.
