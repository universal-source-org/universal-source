# Host API

Status: v0.1 draft. This document defines **service responsibilities and security boundaries**, independently of an implementation language. Exact callable signatures, byte representations, HTML selector coverage, cookie algorithms, and cryptographic algorithm profiles remain future RFC work. The service names below are manifest vocabulary, not a completed JavaScript SDK. A host MUST NOT claim interoperable service support solely because it recognizes a name.

## Capabilities and permission grants

The manifest separates services needed to execute a source (`capabilities.host`) from authority requested (`permissions`). A runtime MUST check required services before executing an entry. Missing required services prevent loading; they MUST NOT be replaced with silent no-ops. An undeclared service MUST NOT be accessible to the source.

The host MAY deny any requested permission or narrow an origin grant. It MUST NOT grant authority beyond what the manifest requests. Denied permissions do not necessarily prevent loading: a source MAY perform work that does not need them. An attempted denied action MUST fail with `PERMISSION_DENIED`, without performing the action. The runtime MUST enforce grants on every use, including after revocation.

| Permission | Requested authority |
| --- | --- |
| `network` | Array of exact HTTP(S) origins that source-driven traffic may contact. `[]` permits none. |
| `cookies` | Access to a cookie jar isolated to this source, restricted to granted network origins. |
| `storage` | Persistent key/value storage isolated to this source. |

An origin is `scheme://host[:port]`, with no path (including a trailing slash), query, fragment, user information, or wildcard. Schemes and DNS names MUST be lowercase; international DNS names MUST use ASCII encoding. Default ports MUST be omitted; explicit ports MUST be integers from 1 through 65535 without leading zeros. IPv6 literals MUST use brackets. Hosts MUST parse and compare normalized origins, not URL string prefixes. Semantically duplicate origins MUST be rejected.

The schema checks a coarse origin shape. Full URL parsing, valid host/port syntax, canonicalization, and duplicate-origin checks are required semantic validation. HTTPS SHOULD be used. HTTP requires its own explicit origin grant. A grant to `https://example.org` does not authorize subdomains or a different scheme or port.

Declaring `http` or `cookies` requires a nonempty `network` request. Declaring `cookies` or `storage` also requires its corresponding permission to be `true`. A requested cookie permission requires nonempty `network` and the `cookies` service; a requested storage permission requires the `storage` service. Network permission can be requested without `http` because a static source may return poster or media URLs.

## Host service boundaries

| Service | Required behavior |
| --- | --- |
| `http` | Make bounded HTTP(S) requests; return status, headers, and body. Check origin grants before every request and redirect. Non-2xx responses MUST remain inspectable responses. Transport failures use `NETWORK_ERROR`. |
| `cookies` | Read, update, and clear source-isolated cookie state for permitted origins. HTTP and subsequent resource consumption MUST use only this source's granted jar. Cookies MUST NOT come from the host's browser session or another source. |
| `storage` | Read, write, and remove source-isolated string keys with JSON values. Reads MUST distinguish a missing key from a stored JSON `null`. Values MUST be copied across the boundary, not shared as mutable host objects. |
| `html` | Parse inert HTML and query its structure. Parsing MUST NOT execute scripts, fetch subresources, navigate, or expose a browser window. Parser objects MUST remain inside the execution instance. |
| `json` | Parse JSON text and serialize JSON-compatible values. Invalid input, duplicate member names, and non-finite numbers MUST be rejected with `INVALID_ARGUMENT`. JSON processing MUST NOT evaluate code. |
| `crypto` | Provide explicitly specified cryptographic primitives through the host. Unsupported algorithms MUST fail explicitly; hosts MUST NOT substitute an algorithm. Access to host keychains or platform credentials is not implied. |
| `url` | Parse and resolve URLs without network access. Utility acceptance of a URL MUST NOT be treated as permission to fetch it. |
| `log` | Accept diagnostic messages at `debug`, `info`, `warn`, or `error` level. Hosts MAY filter or discard logs. Logs MUST NOT serve as an operation's result channel. |

The eventual bindings MUST preserve these rules and the [source error vocabulary](./source-api.md). Sources MUST NOT rely on browser globals, Node.js APIs, filesystem access, process execution, native bridges, or a runtime's implementation language as part of the portable contract. Recognition of the `javascript` engine does not grant ambient capabilities.

## Network and returned resources

The same network boundary applies to HTTP requests, cookie access, poster fetches, and resolved media resources. The runtime MUST reject source results containing URLs outside requested and granted origins with `PERMISSION_DENIED`. It MUST NOT fetch URLs merely to validate their syntax. Hosts consuming results MUST recheck grants before access and on every redirect, including requests initiated by a media subsystem. If a host cannot enforce this boundary in that subsystem, it MUST refuse the access.

The host owns transport headers and cookie attachment. Source HTTP requests MUST obey the same header restrictions as [play resources](./source-api.md). With cookies denied or undeclared, requests MUST NOT attach or retain cookies, including `Set-Cookie` from responses. Cookie operations and response metadata MUST NOT bypass this rule. With cookies granted, the host MUST still apply origin grants and cookie scope rules; the precise cookie profile is pending.

Hosts MAY apply stricter destination policy, including blocking loopback, private-network, or local service addresses. Such policy MUST apply after DNS resolution and redirects as well as before connection. Sources MUST NOT assume a declared origin overrides host policy. Permission checks MUST NOT be bypassed by alternative host services or legacy adapters.

## Isolation and limits

Storage and cookie namespaces MUST separate sources and host user contexts; a source ID alone MUST NOT establish publisher trust or entitlement to another installation's state. Replacing a source with the same ID requires a host-controlled update decision. Storage SHOULD survive ordinary disposal and reload; deletion and eviction policies MUST be documented. Cookie persistence is host policy until a portable profile is defined.

Hosts MUST bound execution time, memory, response sizes, and persistent storage. Exact quotas are host policy and MUST be documented. Sources MUST tolerate `TIMEOUT` and `RESOURCE_LIMIT`. Hosts SHOULD redact secrets from diagnostics; sources MUST NOT deliberately log cookies, authorization headers, or stored secrets. No host-service call may outlive its owning operation's cancellation or instance disposal in a way that grants further source access.
