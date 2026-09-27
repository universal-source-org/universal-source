# Manifest schema conformance

This workflow checks the existing [v0.1 manifest schema](../spec/schema/manifest.schema.json), seven schema-valid manifests, 34 schema-invalid manifests, and the [minimal example manifest](../examples/json/minimal/manifest.json). It does not load or execute sources.

## Setup

Use Python 3.10 or newer. From the repository root, create an isolated development environment and install the pinned validator:

```sh
python3 -m venv .venv
.venv/bin/python -m pip install -r conformance/requirements.txt
source .venv/bin/activate
```

On Windows PowerShell, the equivalent setup is:

```powershell
py -3 -m venv .venv
.venv\Scripts\python.exe -m pip install -r conformance/requirements.txt
.venv\Scripts\Activate.ps1
```

After setup, the single validation command from the repository root is:

```sh
python -m unittest discover -s conformance -p 'test_*.py' -v
```

If activation is unavailable, use `.venv/bin/python` on macOS/Linux or `.venv\Scripts\python.exe` on Windows in place of `python` in that command. Installation may require package-index access; validation uses the installed metaschemas and local files and does not require network access.

[`requirements.txt`](requirements.txt) pins `jsonschema==4.26.0` as a development-only dependency. Python is a tooling choice, not a source engine or reference-runtime decision. Transitive dependencies are resolved by pip; this is not a complete environment lock.

## What the command proves

[`test_manifests.py`](test_manifests.py) uses the library's [Draft202012Validator](https://python-jsonschema.readthedocs.io/en/stable/validate/) directly. It checks the declared dialect and validates the schema against its Draft 2020-12 metaschema before checking any manifests. It then checks that every valid fixture and the existing example pass, and every invalid fixture is rejected.

Success ends with `Ran 3 tests` and `OK`: those three test groups cover all 41 fixtures and the example. Each fixture is a named subtest, so a failure identifies its file. A mismatch, missing example, empty/missing fixture group, invalid schema, or malformed JSON exits nonzero. A malformed negative fixture does not count as successful schema rejection. JSON decoding also rejects duplicate object members and non-JSON constants such as `NaN`.

The workflow checks structural schema conformance, not full source validity. The fixtures are standalone manifests with illustrative entries; no corresponding source files are provided or executed. A schema-valid JavaScript manifest does not demonstrate JavaScript engine support. Negative fixtures are valid JSON intended to fail schema validation, and their expected constraints are listed below.

## Valid fixtures

| Fixture | Coverage |
| --- | --- |
| [declarative-minimal](valid/manifests/declarative-minimal.json) | One operation, no host services, no requested access. |
| [declarative-resource-origin](valid/manifests/declarative-resource-origin.json) | Static `play` source may request a media origin without the `http` service. |
| [javascript-minimal](valid/manifests/javascript-minimal.json) | JavaScript engine, nested `.js` entry, one operation, no services. |
| [javascript-http](valid/manifests/javascript-http.json) | HTTP requires a network origin, but not cookie or storage permission. |
| [javascript-cookies](valid/manifests/javascript-cookies.json) | Cookie service and permission with an origin; `http` is not a mandatory declaration. |
| [javascript-storage](valid/manifests/javascript-storage.json) | Storage service and permission without network access. |
| [javascript-all-services](valid/manifests/javascript-all-services.json) | All five operations and eight services, matching permissions, HTTPS, HTTP with a nondefault port, and bracketed IPv6. |

## Invalid fixtures

| Fixture | Expected schema constraint |
| --- | --- |
| [missing-name](invalid/manifests/missing-name.json) | Required top-level field. |
| [unsupported-version](invalid/manifests/unsupported-version.json) | Exact `specVersion: "0.1"`. |
| [numeric-version](invalid/manifests/numeric-version.json) | Version is a string constant, not a number. |
| [unsupported-engine](invalid/manifests/unsupported-engine.json) | `wasm` is not an allowed engine. |
| [invalid-id](invalid/manifests/invalid-id.json) | Lowercase dotted ID pattern. |
| [empty-name](invalid/manifests/empty-name.json) | Nonempty name. |
| [unknown-field](invalid/manifests/unknown-field.json) | Closed top-level object. |
| [unknown-permission](invalid/manifests/unknown-permission.json) | Closed permissions object. |
| [missing-permission](invalid/manifests/missing-permission.json) | All permission fields must be explicit. |
| [invalid-permission-type](invalid/manifests/invalid-permission-type.json) | Permission flag must be a boolean. |
| [unknown-capability-field](invalid/manifests/unknown-capability-field.json) | Closed capabilities object. |
| [missing-host-list](invalid/manifests/missing-host-list.json) | Host-service array is required even when empty. |
| [empty-operations](invalid/manifests/empty-operations.json) | At least one operation. |
| [unknown-operation](invalid/manifests/unknown-operation.json) | Only the five initial operations. |
| [duplicate-operations](invalid/manifests/duplicate-operations.json) | Unique operation names. |
| [unknown-host-service](invalid/manifests/unknown-host-service.json) | Only the specified host-service vocabulary. |
| [duplicate-host-services](invalid/manifests/duplicate-host-services.json) | Unique host-service names. |
| [absolute-entry](invalid/manifests/absolute-entry.json) | Entry must be relative. |
| [traversal-entry](invalid/manifests/traversal-entry.json) | No `..` path segment. |
| [backslash-entry](invalid/manifests/backslash-entry.json) | Forward slashes only. |
| [declarative-wrong-suffix](invalid/manifests/declarative-wrong-suffix.json) | Declarative entry ends in `.json`. |
| [javascript-wrong-suffix](invalid/manifests/javascript-wrong-suffix.json) | JavaScript entry ends in `.js`. |
| [declarative-host-service](invalid/manifests/declarative-host-service.json) | Static declarative profile has no host-service declarations. |
| [wildcard-origin](invalid/manifests/wildcard-origin.json) | No wildcard origins. |
| [origin-with-path](invalid/manifests/origin-with-path.json) | Origin contains no path. |
| [duplicate-origins](invalid/manifests/duplicate-origins.json) | Unique network entries. |
| [http-without-origin](invalid/manifests/http-without-origin.json) | HTTP service requires a nonempty network list. |
| [cookies-without-origin](invalid/manifests/cookies-without-origin.json) | Cookie service requires a nonempty network list, independently of HTTP. |
| [cookies-without-permission](invalid/manifests/cookies-without-permission.json) | Cookie service requires cookie permission. |
| [cookie-permission-without-service](invalid/manifests/cookie-permission-without-service.json) | Cookie permission requires the cookie service. |
| [storage-without-permission](invalid/manifests/storage-without-permission.json) | Storage service requires storage permission. |
| [storage-permission-without-service](invalid/manifests/storage-permission-without-service.json) | Storage permission requires the storage service. |
| [entry-trailing-newline](invalid/manifests/entry-trailing-newline.json) | Entry pattern matches the entire string. |
| [id-trailing-newline](invalid/manifests/id-trailing-newline.json) | ID pattern matches the entire string. |

## Semantic checks still outside this workflow

The [specification index](../spec/README.md), [Host API](../spec/host-api.md), and [Source API](../spec/source-api.md) require additional checks that the manifest schema does not claim to perform:

- Entry-file existence and containment after resolving symbolic links.
- Full origin parsing, valid hosts and ports, normalization, default-port omission, and semantic duplicate detection. For example, the coarse schema pattern can accept port `99999` or an explicitly written default port even though normative prose rejects them. Such schema acceptance is not a contract contradiction: the specification explicitly requires additional semantic checks.
- Source operation implementation, declarative entry shapes, and cross-references.
- Host-service availability, effective permission grants, redirects, isolation, and lifecycle enforcement.

No origin is contacted, and no manifest permission is granted by this command. It does not verify the example's runtime outputs, code execution, codec support, or platform interoperability. Declarative input/expected-result cases and a future execution harness remain separate work.

To add a fixture, put a plain UTF-8 JSON file directly in `valid/manifests/` or `invalid/manifests/`, describe its purpose here, and run the command. Keep schema-negative cases focused on one requirement where possible. If a genuine schema/prose contradiction is found, report it for explicit resolution instead of weakening the test or silently changing the contract.
