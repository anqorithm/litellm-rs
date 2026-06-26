# Task Plan

## Linked Issue

GH-711

## Spec Packet

- Product: `product.md`
- Tech: `tech.md`

## 实现任务

- [x] `SP711-T1` Owner: coordinator. Done when: fixed-point `BudgetAmount` rejects negative, NaN, and infinite inputs at budget boundaries. Verify: focused unit tests for valid and invalid conversions.
- [x] `SP711-T2` Owner: coordinator. Done when: `BudgetTracker::reserve_spend` returns a reservation that settle/cancel/drop adjusts `current_spend` correctly. Verify: tracker reservation unit tests.
- [x] `SP711-T3` Owner: coordinator. Done when: concurrent reservations competing for the last budget permit allow at most one success. Verify: multi-threaded budget tracker test.
- [x] `SP711-T4` Owner: coordinator. Done when: provider/model managers and `UnifiedBudgetLimits` expose matching reservation semantics. Verify: provider/model/unified reservation tests.
- [x] `SP711-T5` Owner: coordinator. Done when: estimated-cost budget routing and chat/Responses streaming paths reserve before upstream work; completed spend path keeps priced/unpriced semantics and records actual over-reservation spend. Verify: budget router, spend route, and responses stream tests.
- [x] `SP711-T6` Owner: coordinator. Done when: PR body records SpecRail readiness/review/merge gates and distributed follow-up. Verify: PR template checklist is completed with evidence.

## 并行拆分

- Read-only planner/reviewer lanes may inspect provider/model budget routing while the coordinator edits core budget code.
- Writable implementation is serial for this issue because `src/core/budget/**` ownership overlaps across tracker, provider limits, and tests.

## 验证

- `python3 /Users/apple/Desktop/code/AI/tool/specrail/checks/check_workflow.py --repo /Users/apple/Desktop/code/AI/tool/specrail --spec-dir /private/tmp/litellm-rs-issue711/specs/GH711`
- `cargo test budget --lib`
- `cargo test spend --lib`
- `cargo test responses_stream --lib`
- `cargo check --all-features --locked`

## Handoff Notes

SpecRail gate for `implement` is advisory/warn until this packet exists and validates. `threads` remains responsible for queue truth, reviewThreads, CI, merge gate, and closure audit.
