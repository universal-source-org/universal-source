# QuickJS candidate cost and selection-readiness review

Status: completed. Baseline: `06b8ea646d028cda7f869ff7f3875b8acbb7146e`,
`main`, clean tree; active directory contained only `.gitkeep`.

## Scope and decision

Produce one new evidence-linked candidate review: whether the full QuickJS-NG
patch, policy, binding, parser and platform burden warrants further selection
investment. Treat engine-level terminal allocation/heap failure as feasible per
the task premise; do not repeat the feasibility question. The checked-in evidence
currently ends at public-API Outcome B, so distinguish that premise from an
inspectable terminal-OOM patch and its unknown size/provenance. A clarification
request asked for the missing evidence location; no additional artifact was
available during the review. The task premise was retained explicitly.

Read CURRENT_STATE routing, the prior consolidated review and latest failure
reviews first; follow only relevant component evidence for cost/audit questions.
Preserve historical reviews. Update CURRENT_STATE, ROADMAP and ARCHITECTURE to
record the new outcome, honest evidence boundary and exactly one next task.

No engine edits/spikes, parser/module/platform implementation, alternative-engine
experiments, IPC design, production runtime, selection ADR, RFC acceptance,
contract changes or publication. This plan does not authorize any of them.

## Steps

1. Inventory evidenced patches, permanent policies, binding/dependency costs,
   unresolved gates and plausible risks; compare with the near-upstream ideal.
2. Decide Continue, Conditional continue or Pause/reconsider with explicit
   decision criteria and stopping conditions; name exactly one next task.
3. Update affected context documents; check links/anchors, Markdown structure,
   whitespace and contract/status consistency; review the entire diff.
4. Record results here, move to completed and create a local Conventional Commit.

## Validation boundary

Documentation-only assessment reuses dated experimental results without claiming
fresh execution or upstream status. Follow CONTRIBUTING's documentation checks;
no shared runtime/patch/policy changes or new experimental closure are made, so
experiment rebuilds and full Phase 3 reruns are not evidence for this decision.

## Results

- **Conditional continue:** retain QuickJS only for the complete patch-set
  maintenance acceptance gate before investing in other selection work.
- New [candidate review](../../reviews/2026-09-30-quickjs-candidate-viability.md)
  inventories engine changes, permanent host policy, parser dependency/consistency,
  binding/unsafe ownership, platform gaps and upgrade/security responsibilities.
- Accepted terminal-OOM feasibility as the task premise; did not invent patch
  size/provenance or close the integrated gate without an inspectable artifact.
- Updated CURRENT_STATE, ROADMAP and ARCHITECTURE; preserved historical reviews,
  experiments, pins, contracts, ADRs and production files. No next gate executed.
- Remaining gaps: full patch artifact/site audit and maintenance acceptance;
  resource composition/coverage, modules, parser and platform proofs. No owner or
  upstream acceptance fabricated; no engine/fork/parser selected.

## Validation results

- Scoped check: five Markdown files, 185 relative links and 14 anchors passed;
  heading spacing, table columns, closed fences, final newlines and external URL
  syntax checked. Relative targets/anchors were checked on disk; external sites
  were not fetched. The temporary checker is outside the repository.
- Full tracked diff and both new files reviewed for scope, evidence labels,
  permanent costs versus gates, unique next task and unchanged contract/RFC/ADR
  status. Whitespace and staged whitespace checks passed.
- Changed-file allowlist contains exactly the new review, this completed plan,
  CURRENT_STATE, ROADMAP and ARCHITECTURE. Active plans retain only `.gitkeep`.
This candidate-cost assessment is documentation-only under CONTRIBUTING and
CURRENT_STATE's documentation-only rule. It is not experimental closure or a
fresh technical feasibility consolidation: no full Phase 3 corpus/runtime rebuild
was run, and no inherited test result is labeled as newly executed. External
links remain historical references; no current upstream/security/platform claim
was researched. No JSON example or contract was changed.
