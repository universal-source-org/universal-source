# Source API

Status: v0.1 draft. Requirement terms and scope are defined in the [specification index](./README.md).

## Calls and shared data

There are exactly five operations: `home`, `category`, `search`, `detail`, and `play`. Calls have one JSON object input and one JSON result. They are logical interfaces, not HTTP endpoints, native methods, or a prescribed JavaScript binding. A call MAY complete asynchronously; it MUST have exactly one terminal outcome.

A successful result is `{"ok":true,"data":...}`. A failed result is `{"ok":false,"error":{"code":"NOT_FOUND","message":"Unknown content id"}}`. Success MUST NOT include `error`; failure MUST NOT include `data`. The envelope MUST contain only these fields. `code` is machine-readable; `message` is a nonempty diagnostic string and MUST NOT expose credentials or internal stack traces. Hosts MUST branch on `code`, not message text.

Unless marked optional, fields below are required. Unknown fields in inputs, results, and declarative entries MUST be rejected in v0.1. Missing and `null` are distinct; `null` is invalid unless explicitly permitted. Strings MUST be Unicode text. IDs, names, and titles MUST be nonempty strings. Objects, arrays, strings, booleans, finite numbers, and explicit `null` are the only transferable values; native handles and functions MUST NOT appear in source results.

IDs are opaque, case-sensitive, and scoped to the manifest's source `id`. Hosts MUST NOT parse IDs as URLs or legacy delimiter-separated records. Sources SHOULD keep IDs stable when the underlying content is unchanged. A host MUST combine source identity and item ID when indexing content from different sources.

| Type | Fields |
| --- | --- |
| `Category` | `id`, `name`. |
| `Content` | `id`, `title`; optional `poster` (absolute HTTP(S) URL). |
| `Playable` | `id`, `title`. Its ID is unique across playable items in this source. |
| `Page` | `items` (array of `Content`), `page` (integer ≥ 1), `hasMore` (boolean). |
| `Detail` | `id`, `title`, `playables` (array of `Playable`); optional `poster` (absolute HTTP(S) URL), `description` (string). |
| `Resource` | `url` (absolute HTTP(S) URL); optional `headers` (string-to-string object), `mimeType` (nonempty media type string). |

Content, playable, and category IDs have separate namespaces. IDs within an array MUST be unique. A list's content IDs refer to `detail` IDs when `detail` is offered; `Detail.playables` IDs refer to `play` IDs when `play` is offered. URLs MUST NOT contain embedded username/password credentials. Resource header names MUST be valid HTTP field names, case-insensitively unique, and MUST NOT contain CR or LF in names or values. Hosts MUST treat returned text as data, never executable markup.

## Operations

| Operation | Input | Success `data` |
| --- | --- | --- |
| `home` | `{}` | `{"categories":[Category,...],"items":[Content,...]}` |
| `category` | `{"id":"category-id","page":1}` | `Page` |
| `search` | `{"query":"text","page":1}` | `Page` |
| `detail` | `{"id":"content-id"}` | `Detail` |
| `play` | `{"id":"playable-id"}` | `Resource` |

For `category` and `search`, `page` is optional and defaults to `1`. It MUST otherwise be an integer from 1 through 9007199254740991. No other input has implicit defaults. `id` and `query` MUST be nonempty strings; a query containing only spaces, tabs, carriage returns, or line feeds (U+0020, U+0009, U+000D, U+000A) is invalid. Hosts MUST pass a valid query unchanged. Matching semantics are source-defined except for the static declarative format below.

### `home`

Returns available entry categories and a source-defined content list. Both arrays MAY be empty. It does not define recommendations, personalization, or a screen layout. Category IDs MUST be usable by `category` when that operation is offered.

### `category` and `search`

Pages are one-based, with source-defined page size. The returned `page` MUST equal the requested page after defaults. `hasMore` means a later page is available. No results or a page past the end MUST return `items: []` and `hasMore: false`; an unknown category MUST instead fail with `NOT_FOUND`. A valid search with no matches is successful, not `NOT_FOUND`. Hosts MUST NOT infer totals or a fixed page size.

### `detail`

Returns one item's description and playable items. The returned `id` MUST equal the input `id`. `playables` MAY be empty. An unknown ID MUST fail with `NOT_FOUND`. Multiple playable items can express episodes or alternative resources without prescribing a player interface.

### `play`

Resolves one playable ID to one resource; it does not start playback. Unknown IDs MUST fail with `NOT_FOUND`. A host MUST NOT execute scripts, instantiate native plugins, or treat arbitrary URI schemes as media resources. The host enforces [network permissions](./host-api.md) before returning or consuming a resource. Returned headers apply only to the resolved URL's origin and MUST NOT be forwarded to another origin on redirect. A source MUST NOT supply `Host`, `Content-Length`, `Connection`, `Transfer-Encoding`, `Cookie`, or `Set-Cookie` headers (case-insensitive); transport and cookie state belong to the host. Codec and media playback support are outside source API conformance.

Example successful resolution (the URL is a placeholder):

```json
{"ok":true,"data":{"url":"https://media.example.invalid/demo.mp4","mimeType":"video/mp4"}}
```

## Errors

| Code | Meaning |
| --- | --- |
| `INVALID_ARGUMENT` | Invalid operation input or host-service argument. |
| `UNSUPPORTED_OPERATION` | The source does not declare the requested operation. |
| `NOT_FOUND` | A category, content, or playable ID is unknown. |
| `PERMISSION_DENIED` | Access is undeclared, ungranted, or blocked by host policy. |
| `NETWORK_ERROR` | A permitted network request failed at the transport level. |
| `TIMEOUT` | The host's execution or request deadline expired. |
| `CANCELLED` | The caller or host cancelled the call. |
| `RESOURCE_LIMIT` | A documented host resource limit was exceeded. |
| `INVALID_RESULT` | A source returned data that violates this contract. |
| `SOURCE_ERROR` | An otherwise unclassified source execution failure. |

These are the only source-call error codes in v0.1. Host policy rejection MUST NOT be disguised as an empty successful result. An HTTP error status alone is not a transport failure. The runtime MUST translate uncaught source failures into `SOURCE_ERROR` and malformed results into `INVALID_RESULT`. Validation and loading failures are separate [lifecycle diagnostics](./lifecycle.md), before an operation can be called.

## Minimal declarative entry format

The initial `declarative` engine reads static UTF-8 JSON data. It performs no requests, script evaluation, interpolation, selectors, or expressions. This deliberately small baseline does not yet attempt to replace scraping rule systems. A declarative manifest's `capabilities.host` MUST be empty.

The entry is an object whose keys MUST exactly match `capabilities.operations`:

| Entry key | Stored value | Dispatch behavior |
| --- | --- | --- |
| `home` | Object with `categories` and `items` arrays. | Return the object. |
| `category` | Object mapping category IDs to arrays of `Content`. | Return that array on page 1; later pages are empty. |
| `search` | Object mapping query strings to arrays of `Content`. | Exact, case-sensitive query lookup on page 1; an absent query or later page is empty. |
| `detail` | Object mapping content IDs to `Detail`. | Return the matching object; its `id` MUST equal its key. |
| `play` | Object mapping playable IDs to `Resource`. | Return the matching object. |

Stored values contain only success data, not result envelopes. The runtime supplies the envelope and, for paged calls, `page` and `hasMore: false`. Category lookup MUST occur before handling out-of-range pages, so unknown categories always fail with `NOT_FOUND`. Missing `detail` and `play` keys also fail with `NOT_FOUND`. Search-map keys MUST be valid query strings. No Unicode normalization, case folding, or trimming occurs during lookup. Map keys MUST be treated as data, including names that overlap with implementation-language object properties.

Loading MUST validate the whole entry's shapes and references. When the target operation exists, home categories MUST appear in `category`; content in home/category/search MUST have a `detail` entry; every detail playable MUST have a `play` entry. A playable ID reused across details MUST describe the same playable item. Unreferenced entries MAY exist. See the [complete minimal example](../examples/json/minimal/README.md).
