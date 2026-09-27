# Minimal declarative source

This static example illustrates all five operations in the [v0.1 Source API](../../../spec/source-api.md). It is data, not a runtime implementation. No dependencies, scripts, external requests, or player are needed to inspect it.

- [`manifest.json`](./manifest.json) declares the version, identity, engine, entry, operations, and permissions.
- [`source.json`](./source.json) contains the static operation data.
- The [manifest schema](../../../spec/schema/manifest.schema.json) validates manifest structure; the specification defines semantic and entry checks.

`https://media.example.invalid/demo.mp4` is deliberately a placeholder, not a working media stream. The network permission illustrates the authority a host would need before returning or consuming that resource. It grants nothing until approved by a host. The source requires no host services because its operations only look up static JSON; the `http` service is not needed to return a URL.

## Expected calls

Assuming a future conforming declarative runtime has loaded the source and granted its requested media origin:

| Operation | Input | Expected behavior |
| --- | --- | --- |
| `home` | `{}` | Category `all` and content item `demo`. |
| `category` | `{"id":"all"}` | `demo` on page 1, with `hasMore: false`. |
| `search` | `{"query":"demo","page":1}` | `demo` on page 1, with `hasMore: false`. |
| `detail` | `{"id":"demo"}` | Detail with playable ID `demo-main`. |
| `play` | `{"id":"demo-main"}` | The placeholder MP4 resource. |

For example, `category({"id":"all"})` logically yields:

```json
{
  "ok": true,
  "data": {
    "items": [{ "id": "demo", "title": "Demo video" }],
    "page": 1,
    "hasMore": false
  }
}
```

The static search is an exact, case-sensitive lookup. `{"query":"Demo"}` or `{"query":"missing"}` returns an empty page. Page 2 for a known category or any valid query is also empty. An unknown category, content ID, or playable ID returns `NOT_FOUND`; `page: 0` returns `INVALID_ARGUMENT`. If the media origin is denied, `play` returns `PERMISSION_DENIED`, while operations that return no URLs can still succeed.

To point this example at a real resource you control, change the URL in `source.json` and the requested origin in `manifest.json` together. There is no runtime command or playback test in this repository yet.
