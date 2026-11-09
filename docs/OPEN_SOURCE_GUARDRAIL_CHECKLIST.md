# Open-Source and Low-Resource Guardrail Checklist

Date introduced: 2026-03-05 (UTC)

Use this lightweight checklist for every sprint, PR, and release review.

## Mission Guardrails

- [ ] `Offline-first` remains intact: no feature requires internet to function locally.
- [ ] `Local-first` remains intact: core data is stored locally and remains usable offline.
- [ ] No mandatory proprietary/cloud dependency was introduced for core workflows.
- [ ] Any sync or external integration remains optional and explicitly user-controlled.

## Emerging-Market Hardware Guardrails

- [ ] Startup/runtime overhead was not materially increased without justification.
- [ ] No new heavy background polling/jobs were added without strict throttling.
- [ ] Data access paths use indexes or bounded queries where needed.
- [ ] Large-list rendering or expensive loops were avoided in hot paths.
- [ ] Bundle/build artifact growth is justified and documented when non-trivial.

## Open-Source Guardrails

- [ ] New dependencies have compatible licenses for Apache-2.0 distribution.
- [ ] Public APIs/commands introduced in the sprint are documented.
- [ ] Changelog and sprint ledger were updated chronologically with UTC date.
- [ ] Behavior changes include migration/compatibility notes where relevant.

## Workflow Guardrails

- [ ] Work was done on a dedicated sprint branch.
- [ ] Validation commands and known blockers are documented in sprint notes.
- [ ] `npm run check:release-guardrails` passed or a scoped exception was documented.
- [ ] PR includes this checklist with explicit confirmations or exceptions.
