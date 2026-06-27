# Product Spec

## Linked Issue

GH-713

## User Problem

The public provider contract is ambiguous. `LLMProvider` is documented as the
core provider abstraction, while router deployments store the closed
`Provider` enum. A third-party `LLMProvider` implementation cannot be routed
unless the crate also adds an enum variant and dispatch wiring.

`ProviderHandle` makes the confusion worse because it is public and described as
a type-erased routing wrapper, but its core methods return optimistic stub data:
all models and tools are supported, health is healthy, cost is zero, latency is
fixed, and success rate is 100%.

## Goals

- Make the router provider contract explicit: deployments route through the
  built-in `Provider` enum.
- Keep `LLMProvider` documented as the implementation trait for providers wired
  into this crate, not as a standalone router plugin boundary.
- Keep `ProviderHandle` compatible as a legacy wrapper, but stop presenting it
  as a real routing abstraction.
- Ensure `ProviderHandle` no longer returns optimistic capability, health, cost,
  latency, or success-rate data.
- Record the decision in SpecRail so #714 can focus on registry and declaration
  source-of-truth work.

## Non-Goals

- Do not introduce a `dyn LLMProvider` router path in this issue.
- Do not add `Provider::Custom` or external custom provider routing.
- Do not solve the #714 provider registry/catalog drift in this PR.
- Do not split provider retry/error/HTTP mapping responsibilities from #715.
- Do not change provider factory feature gates or generated docs matrices.

## User-Visible Behavior

The router continues to use built-in provider deployments only. Public docs no
longer imply that implementing `LLMProvider` alone makes a provider routeable.
Any code still using `ProviderHandle` receives explicit unsupported/unknown
results instead of fabricated healthy or successful routing data.

## Acceptance Criteria

- [x] `LLMProvider` docs state that router dispatch currently requires a
  built-in `Provider` enum variant and dispatch wiring.
- [x] `Provider`/`Deployment` docs identify the built-in enum as the router
  deployment contract.
- [x] `ProviderHandle` docs no longer advertise it as an active router dispatch
  wrapper.
- [x] `ProviderHandle::supports_model` and `supports_tools` do not return
  optimistic `true` by default.
- [x] `ProviderHandle::health_check` does not report `Healthy` without evidence.
- [x] `ProviderHandle` cost, latency, and success-rate methods return explicit
  errors instead of fabricated values.
- [x] Tests cover the non-optimistic `ProviderHandle` behavior.

## Follow-Up

#714 should add registry/source-of-truth conformance around the built-in provider
set. A future custom-provider issue can revisit an object-safe adapter or hybrid
`Provider::Custom` design after the closed contract is stable.
