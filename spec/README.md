# Universal Source Specification v0.1

Status: **experimental draft**. This directory defines a portable contract independently of any reference runtime. No implementation language, native ABI, application, or operating system is required.

## Reading order and authority

| Document | Defines |
| --- | --- |
| [Source API](./source-api.md) | Five operations, data types, errors, and static declarative entry format. |
| [Host API](./host-api.md) | Capability boundaries and permission enforcement. |
| [Lifecycle](./lifecycle.md) | Validation, loading, calls, cancellation, and disposal. |
| [Compatibility](./compatibility.md) | Version negotiation, adapters, and conformance boundaries. |
| [Manifest schema](./schema/manifest.schema.json) | Machine-checkable manifest structure. |
| [RFC template](./rfcs/0000-template.md) | Proposed changes; RFCs do not become normative merely by being added. |
| [RFC 0001: JavaScript execution binding](./rfcs/0001-javascript-execution-binding.md) | Proposed modules, exports, lifecycle, values/errors and capability injection; internally reviewed, not accepted or implemented. |

The uppercase terms MUST, MUST NOT, SHOULD, SHOULD NOT, and MAY express requirements as defined in [RFC 2119](https://www.rfc-editor.org/rfc/rfc2119). Lowercase uses are ordinary prose. These terms describe conformance to this draft, not a claim that a conforming runtime exists.

Normative prose and the schema MUST both be satisfied. The schema validates structure; prose adds semantic checks such as origin normalization, entry containment, and correspondence between declarations and implemented operations. A disagreement between them is a specification defect, not permission to choose either behavior. Examples are informative.

## Terms

- **Source**: a manifest and entry file providing content operations.
- **Host**: the application granting permissions and consuming results.
- **Runtime**: the implementation validating and executing a source on behalf of a host.
- **Engine**: the execution format, currently `declarative` or `javascript`.
- **Capability**: an operation offered by the source or a host service required by it.
- **Permission**: requested authority to access a resource, subject to host approval and policy.
- **Adapter**: a separate translation or execution layer for a legacy ecosystem.

## Manifest

A source MUST contain a UTF-8 JSON `manifest.json` satisfying the [Draft 2020-12 schema](./schema/manifest.schema.json). JSON documents MUST NOT contain duplicate object member names. The manifest has exactly these required fields:

| Field | Meaning |
| --- | --- |
| `specVersion` | Exact contract version; this draft accepts only `"0.1"`. |
| `id` | Stable lowercase dotted identifier, such as `org.example.minimal`; not a trust credential. |
| `name` | Nonempty display name. |
| `engine` | `declarative` or `javascript`; no inferred fallback. |
| `entry` | Relative file path within the source root. |
| `permissions` | `network` origin array and `cookies` / `storage` booleans. |
| `capabilities` | `operations` offered and `host` services required. |

`capabilities.operations` MUST be a nonempty, duplicate-free subset of `home`, `category`, `search`, `detail`, and `play`. A source MUST implement every operation it declares. It MUST NOT require callers to invoke undeclared operations. Calls to undeclared operations fail with `UNSUPPORTED_OPERATION`.

`capabilities.host` MUST be a duplicate-free subset of `http`, `cookies`, `storage`, `html`, `json`, `crypto`, `url`, and `log`. All listed services are required for loading, not optional hints. An empty list is valid. Engine-internal JSON decoding does not require the source to declare the `json` service. All manifest permission fields are explicit; empty arrays and `false` request no access.

`entry` MUST use forward-slash-separated ASCII path segments beginning with a letter, digit, or underscore, followed by letters, digits, underscores, dots, or hyphens. Absolute paths, empty segments, `.` / `..` segments, backslashes, URL schemes, and percent escapes are forbidden. The resolved file, including any symbolic links, MUST remain inside the source root and exist. It MUST end in `.json` for `declarative` and `.js` for `javascript`. File suffixes do not override `engine`.

Unknown manifest fields, engine values, operations, and host services MUST be rejected for this version. Extensions need an explicit specification change; hosts MUST NOT silently reinterpret unknown declarations.

## Scope and remaining work

v0.1 defines only the five source operations. It excludes recommendation systems, accounts, sync, DRM, subtitles, comments, danmaku, downloads, player UI, native applications, and WASM engines. Network scraping rules are not part of the initial static declarative format.

The [minimal declarative source](../examples/json/minimal/README.md) demonstrates the data contract; the [declarative harness](../conformance/declarative/README.md) now executes the static profile. [RFC 0001](./rfcs/0001-javascript-execution-binding.md) proposes the JavaScript binding without amending this draft. Normative adoption, JavaScript execution/conformance, and exact host service signatures and algorithm profiles remain open work. Their absence MUST NOT be presented as complete JavaScript or host-service interoperability. Runtime implementations may use Rust or another language; that decision does not alter this specification.
