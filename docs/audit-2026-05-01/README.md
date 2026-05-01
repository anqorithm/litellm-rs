# Audit 2026-05-01 evidence

This directory preserves the evidence used by the audit remediation campaign.

## Counts

| Source | Critical | High | Medium | Total |
|--------|----------|------|--------|-------|
| Raw audit findings | 20 | 27 | 30 | 77 |
| Deduplicated execution tracker | 20 | 22 | 30 | 72 |

The raw audit contained 77 findings from four parallel agents. During planning, five High findings were merged into already-tracked Critical/High remediation items, leaving 72 executable rows in `PLAN_AUDIT_REMEDIATION.md`.

## Files

- `raw-consolidated-findings.md` is the durable copy of the consolidated raw audit list provided in the remediation thread.
- `agent-1-api-data-integrity.md`, `agent-2-error-security.md`, `agent-3-architecture.md`, and `agent-4-config-persistence.md` preserve the available per-agent provenance counts. The full per-agent transcripts were not present in this checkout when Step A1 ran.

## Linked plans

- `../../PLAN_AUDIT_REMEDIATION.md`
- `../../PLAN_AUDIT_EXECUTION.md`
- `../plan/audit-remediation-complete-plan.md`
