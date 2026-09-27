# Contributing

Start with [AGENTS.md](AGENTS.md), the concise map to the project's authoritative context. Read the relevant charter, principles, current architecture, roadmap phase, specification sections, and accepted decisions before changing behavior. Repository documents must carry enough context for both people and agents to continue without conversation history.

## A coherent unit of work

For substantial tasks:

1. Inspect Git status, relevant files, and [active plans](docs/plans/active/). Identify existing work and preserve it; do not reset, overwrite, or commit unrelated changes without authorization.
2. Choose a bounded result within the [active roadmap phase](docs/ROADMAP.md). A future phase is not an instruction to start it. Record scope, non-goals, affected contracts, and validation in a short active plan before implementation.
3. Implement the coherent unit. Follow existing specification semantics; use an RFC for contract proposals and an ADR for consequential architectural choices. Do not add speculative layers as incidental cleanup.
4. Run relevant existing tests and checks. Add tests for changed behavior where a testable implementation exists. For documentation or data changes, validate links, formatting, JSON, and applicable schema/example consistency. Report missing tooling and unrun checks explicitly; do not claim runtime conformance from documentation checks.
5. Update affected docs, examples, decisions, and roadmap status when facts or exit criteria change. Move the plan to [completed plans](docs/plans/completed/) with outcomes, validation evidence, and remaining gaps. Leave genuinely unfinished work in active plans with a useful continuation note.
6. Review the complete diff, including new files and staged changes. Check scope, consistency, unintended changes, and sensitive data; run `git diff --check` and `git diff --cached --check` as applicable.
7. Create a local Git commit for the completed work using Conventional Commits. Report the commit hash, changed files, checks, limitations, current phase, and the next coherent task. If a required check or commit fails, report it honestly rather than claiming completion.

Tiny corrections can skip a standalone plan; the scope, validation, and diff-review responsibilities still apply. Plans are task records, not a second specification. Use a descriptive filename such as `YYYY-MM-DD-short-task.md`; keep the existing `.gitkeep` files in both plan directories.

## Validation in the current repository

The repository contains a specification draft, JSON Schema, static example, and [conformance workflow](conformance/README.md). After the documented development-environment setup, run from the repository root:

```sh
python -m unittest discover -s conformance -p 'test_*.py' -v
```

This uses a real Draft 2020-12 validator to check the schema, positive/negative manifest fixtures, and the existing example manifest. It also validates the [declarative operation fixtures](conformance/declarative/README.md), their source references, effective grants, and stored success snapshots. There is no executable source runtime or operation-level execution harness yet. Do not describe schema checks as tested source execution.

For documentation work, check relative links (including anchors when used), Markdown structure, whitespace, and consistency with the specification. Parse JSON examples when touched. Distinguish structural manifest validation from the prose's semantic requirements, as listed in the conformance guide. The [roadmap](docs/ROADMAP.md) records remaining validation work; keep these instructions aligned with available commands.

## Commits and publication

Use `type(scope): summary` or `type: summary`, with an imperative summary; for example `docs: establish repository governance`, `test(spec): add manifest conformance fixtures`, or `feat(runtime): load static declarative sources`. Scope is optional. Keep each commit reviewable and aligned with its actual changes. Do not rewrite existing commits merely to tidy history.

Agents MUST NOT automatically push, force-push, create tags, create releases, or modify repository settings. Each action requires explicit user authorization. A request to implement or commit work does not authorize publishing it. Human contributors should follow the same separation between local preparation and deliberate publication.

## Proposals and durable context

Use the [RFC template](spec/rfcs/0000-template.md) for source/host contract changes and the [ADR process](docs/decisions/README.md) for durable architectural decisions. Include concrete examples, alternatives, portability and permission effects, compatibility implications, and validation evidence. An RFC's presence is not acceptance.

The charter defines purpose, the principles guide tradeoffs, the specification and schema define contracts, the architecture describes current reality, and the roadmap sequences work. ADRs explain decisions; plans track execution. If these conflict, surface the discrepancy and correct the appropriate document within the authorized task instead of silently choosing a different contract.
