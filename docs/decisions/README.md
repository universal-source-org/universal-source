# Architecture decision records

ADRs preserve the rationale for consequential project and implementation choices. They complement the [current architecture](../ARCHITECTURE.md) and [roadmap](../ROADMAP.md); they do not replace the [normative specification](../../spec/README.md).

| Record | Status | Subject |
| --- | --- | --- |
| [0001](0001-initial-architecture.md) | Accepted | Initial architecture and boundaries already reflected in the v0.1 draft. |
| [0002](0002-reference-runtime-language.md) | Accepted | Rust reference implementation; completes the language choice deferred by ADR 0001. |

## When to record a decision

Use an ADR for a durable choice with meaningful alternatives or consequences, such as an implementation language, module boundary, dependency strategy, or change of project direction. Routine edits do not need an ADR. Proposed source/host contract changes use the [RFC template](../../spec/rfcs/0000-template.md); an ADR may link the accepted RFC and capture architectural consequences, but cannot silently amend the contract.

## Format and lifecycle

Create `NNNN-short-title.md` using the next unused number. Include title, status, date, context, decision, alternatives, consequences, and links to supporting evidence or related records. Keep the record proportional to the decision.

Use **Proposed**, **Accepted**, **Rejected**, or **Superseded** status. Adding a proposed file does not accept it; acceptance must be an explicit review outcome or an authorized project decision recorded in the change. ADR 0001 records the existing project direction as part of the governance foundation.

Preserve accepted rationale as history. For a later reversal, create a new record, mark the old record superseded, and cross-link them. Editorial corrections are permitted; do not rewrite history to make a new decision appear original. Update this index and current context documents when a decision changes current reality. Acceptance of a planned direction does not mean its implementation exists.
