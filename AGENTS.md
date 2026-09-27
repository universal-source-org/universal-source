# Agent guide

Use repository files as durable project context; do not rely on conversation history.

## Read before substantial work

1. [Project charter](docs/PROJECT_CHARTER.md): purpose, scope, and non-goals.
2. [Design principles](docs/DESIGN_PRINCIPLES.md): constraints and tradeoffs.
3. [Architecture](docs/ARCHITECTURE.md): what exists and what does not.
4. [Roadmap](docs/ROADMAP.md): active phase and exit criteria.
5. [Specification](spec/README.md): authoritative source/host contracts and schema.
6. [Decisions](docs/decisions/README.md): accepted rationale and superseded choices.
7. [Contributing](CONTRIBUTING.md): workflow, validation, plans, and commits.

Inspect relevant files, Git status, and any [active plans](docs/plans/active/) before editing. Preserve existing work. The specification governs contract behavior; context documents and plans do not silently amend it. Surface conflicts and resolve them in the relevant authoritative document within the task's scope.

## Work and delivery

- Follow the active roadmap phase and the workflow in CONTRIBUTING.md; do not treat future phases as authorization to implement them.
- Complete one coherent unit, run relevant tests, update affected documentation, and review the complete diff.
- For substantial tasks, keep a short repository plan and move it to [completed plans](docs/plans/completed/) with results and remaining gaps.
- Create a local Git commit using Conventional Commits after validation. Report changes, checks, limitations, and the commit hash. Do not include unrelated work without authorization.
- Agents MUST NOT automatically push, force-push, create tags, create releases, or modify repository settings. Each requires explicit user authorization; permission to commit is not permission to publish.
