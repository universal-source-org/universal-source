# QuickJS complete patch-set maintenance acceptance

Status: completed. Baseline: `6fe23ddefc3274d8986f87164cfbca0730e33679`,
`main`, clean tree; no pre-existing active plan.

## Scope

Audit the actual terminal-failure artifacts and maintenance capability, accepting
terminal-OOM feasibility as a premise. Locate any terminal-OOM patch/evidence in
repository files, local refs and relevant generated experiment artifacts; do not
invent or recreate it. Inventory the inspectable async patch, exact provenance,
site/order/ownership boundaries and limits on any combined claim. Define an
operational update/security/build qualification model and distinguish proposed
procedures from accepted human responsibility or executed qualification.

Use CURRENT_STATE's candidate/ownership route: candidate-cost review, async patch
audit and public-API boundary; follow linked drivers and integrated evidence for
specific provenance/cleanup/sanitizer questions. Read governance and applicable
RFC sections. Do not preload unrelated historical reviews.

## Work

1. Search and inspect artifacts; verify hashes and existing source differences
   read-only where needed. No engine patch creation/application/modification.
2. Decide Pass, Not yet accepted or Fail using actual provenance and ownership.
   A missing full artifact is a failed gate, not a new feasibility experiment.
3. Write the maintenance review; update only affected CURRENT_STATE, ROADMAP and
   ARCHITECTURE; identify one next task without executing it.
4. Validate Markdown/links/anchors/whitespace and status consistency; review the
   complete diff, complete this plan and commit locally with Conventional Commits.

## Boundaries and validation

No engine edits or new spike, no module/parser/platform/alternative-engine/IPC
implementation or selection work, no production JS, ADR, RFC acceptance or
portable-contract change. No upstream submission or publication. Reuse dated
execution evidence; run technical checks only to identify existing artifacts,
not full Phase 3 reruns. Preserve ignored outputs as well as tracked work.

## Results

- **Fail — maintenance acceptance not established.** The async subset is
  concentrated and auditable, but no terminal-OOM artifact or combined successful
  configuration was found. Technical feasibility remains the task premise.
- Actual subset: one changed C file, eleven functions, twelve sites; eleven
  ordered recipe rules, 43 added / 4 removed lines. Not a full-set count or cost
  estimate. Exact hashes and per-rule original anchor lines are in the
  [review](../../reviews/2026-09-30-quickjs-maintenance-acceptance.md).
- Required owner/reviewer/security responsibilities and rebase/site-audit,
  regression, upstream/rejection, rollback/suspend and single-engine procedures
  are defined. They have no accepted personnel/capacity or completed operational
  qualification. SECURITY.md is empty. Rust integrated sanitizer gap persists.
- QuickJS loses default candidate status and remains a comparison option. Only
  next task: end-to-end candidate architecture cost comparison under the same RFC;
  no next task executed, no alternate selected or implementation authorized.
- Updated only CURRENT_STATE, ROADMAP, ARCHITECTURE, new review and this plan.

## Artifact audit and validation

Read-only artifact checks performed (no build, driver run or patch application):

- Local branch/worktree and reachable experiment history plus eleven distinct app
  snapshot tree inventories; no terminal-OOM artifact found. Existing patch/build
  drivers have one blob version each among those snapshots and HEAD.
- SHA-256 of all eighteen experiment `quickjs.c` copies: only baseline and
  async-only hashes. All thirty baseline C/header files match pinned registry
  files; generated baseline/patched trees differ only in `quickjs.c`.
- Python AST reads of the recipe checked every baseline anchor occurrence and
  matching patched replacement; `difflib` and full `git diff --no-index` inspection
  confirmed twelve hunks and exact counts. The expected no-index exit 1 means
  different inputs, not an engine-test failure.
- Baseline/patched provenance records: six source/archive/executable hash checks
  passed. Read-only `nm` checks found one definition of each of the three engine
  witness symbols in each binary. This does not prove absence of all hidden
  copies or create a fresh successful-run attestation.
- Recorded driver/harness/lock/header/license hashes and installed sys VCS
  metadata. No fresh upstream, dependency-security or platform validation.
- Five changed Markdown files: 179 relative links, 15 anchors, heading/table/
  fence structure, final newlines and external URL syntax passed. External sites
  were not fetched. Temporary check script lives outside the repository.
- Full diff including both new files reviewed. Outcome, missing-artifact boundary,
  non-operational ownership model, unchanged RFC/ADR status and unique next task
  agree across all affected documents. Git whitespace/staged-whitespace and
  five-file scope checks passed; active plans contain only `.gitkeep`.

Existing runtime/Phase 3 tests and sanitizer suites were not rerun: this task
changes documentation only, and hash/diff/symbol inspection sufficed to identify
the available artifacts. Dated execution results remain explicitly inherited.
All tracked engine/experiment/contract files and ignored generated output were
preserved. No push, tag, release or repository-setting change.
