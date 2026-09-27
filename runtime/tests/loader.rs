use serde_json::{Value, json};
use std::{
    fs,
    path::{Path, PathBuf},
};
use tempfile::TempDir;
use universal_source_runtime::{LoadError as E, LoadLimits, load};

fn repository() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("..")
}
fn example(name: &str) -> Value {
    serde_json::from_slice(
        &fs::read(repository().join("examples/json/minimal").join(name)).unwrap(),
    )
    .unwrap()
}
struct Package {
    dir: TempDir,
    manifest: Value,
    entry: Value,
}
impl Package {
    fn new() -> Self {
        Self {
            dir: tempfile::tempdir().unwrap(),
            manifest: example("manifest.json"),
            entry: example("source.json"),
        }
    }
    fn write(&self) {
        fs::write(
            self.dir.path().join("manifest.json"),
            self.manifest.to_string(),
        )
        .unwrap();
        fs::write(self.dir.path().join("source.json"), self.entry.to_string()).unwrap();
    }
    fn check(&self) -> Result<universal_source_runtime::LoadedSource, E> {
        self.write();
        load(self.dir.path(), LoadLimits::default())
    }
}
fn fails(result: Result<universal_source_runtime::LoadedSource, E>, expected: E) {
    assert_eq!(result.unwrap_err(), expected);
}

#[test]
fn loads_repository_contexts_without_executing_operations() {
    for relative in [
        "examples/json/minimal",
        "conformance/declarative/sources/empty-home",
    ] {
        let source = load(repository().join(relative), LoadLimits::default()).unwrap();
        assert_eq!(source.manifest()["engine"], "declarative");
        assert!(source.entry().is_object());
    }
}

#[test]
fn malformed_manifest_json_unicode_and_duplicates_are_rejected() {
    let package = Package::new();
    for bytes in [
        b"{".as_slice(),
        b"[]",
        b"null",
        b"{} {}",
        b"{\"a\":1,\"a\":2}",
        br#"{"a":{"x":1,"\u0078":2}}"#,
        b"{\"a\":NaN}",
        b"{\"a\":1e999}",
        b"{\"a\":\"\xff\"}",
        br#"{"a":"\ud800"}"#,
    ] {
        fs::write(package.dir.path().join("manifest.json"), bytes).unwrap();
        fails(
            load(package.dir.path(), LoadLimits::default()),
            E::InvalidManifest,
        );
    }
}

#[test]
fn manifest_schema_negative_corpus_is_rejected_before_entry_resolution() {
    let directory = repository().join("conformance/invalid/manifests");
    let package = Package::new();
    let mut count = 0;
    for item in fs::read_dir(directory).unwrap() {
        let path = item.unwrap().path();
        if path.extension().and_then(|s| s.to_str()) != Some("json") {
            continue;
        }
        fs::copy(&path, package.dir.path().join("manifest.json")).unwrap();
        let error = load(package.dir.path(), LoadLimits::default()).unwrap_err();
        assert!(
            matches!(
                error,
                E::InvalidManifest | E::UnsupportedVersion | E::UnsupportedEngine
            ),
            "{}: {error}",
            path.display()
        );
        count += 1;
    }
    assert_eq!(count, 34);
}

#[test]
fn version_engine_and_schema_diagnostics_have_documented_precedence() {
    let mut package = Package::new();
    package.manifest["specVersion"] = json!("0.2");
    package.manifest["engine"] = json!("wasm");
    package.manifest["extra"] = json!(true);
    fails(package.check(), E::UnsupportedVersion);
    package.manifest["specVersion"] = json!("0.1");
    fails(package.check(), E::UnsupportedEngine);
    package.manifest["engine"] = json!("javascript");
    fails(package.check(), E::UnsupportedEngine);
    package.manifest["engine"] = json!("declarative");
    fails(package.check(), E::InvalidManifest);
    package.manifest.as_object_mut().unwrap().remove("extra");
    package.manifest["specVersion"] = json!(0.1);
    fails(package.check(), E::InvalidManifest);
}

#[test]
fn required_services_are_invalid_for_static_profile_not_silent_noops() {
    let mut package = Package::new();
    package.manifest["capabilities"]["host"] = json!(["log"]);
    fails(package.check(), E::InvalidManifest);
    // UnavailableService is a distinct host diagnostic category, but cannot be
    // reached by a schema-valid declarative manifest, whose host list is empty.
}

#[test]
fn semantic_origin_errors_rejected_even_when_schema_accepts() {
    let schema: Value = serde_json::from_slice(
        &fs::read(repository().join("spec/schema/manifest.schema.json")).unwrap(),
    )
    .unwrap();
    let validator = jsonschema::draft202012::new(&schema).unwrap();
    for origin in [
        "https://example.org:443",
        "http://example.org:80",
        "https://example.org:99999",
        "https://a..example.org",
        "https://a.-bad.example.org",
        "https://[:::]",
    ] {
        let mut package = Package::new();
        package.manifest["permissions"]["network"] = json!([origin]);
        assert!(validator.is_valid(&package.manifest), "{origin}");
        fails(package.check(), E::InvalidManifest);
    }
}

#[test]
fn forbidden_origin_components_are_rejected() {
    for origin in [
        "https://example.org/",
        "https://example.org.",
        "https://example.org?q",
        "https://example.org#f",
        "https://u:p@example.org",
        "https://*.example.org",
        "HTTPS://example.org",
        "https://EXAMPLE.org",
        "https://例子.test",
        "https://example.org:0444",
        "https://example.org:0",
    ] {
        let mut package = Package::new();
        package.manifest["permissions"]["network"] = json!([origin]);
        fails(package.check(), E::InvalidManifest);
    }
}

#[test]
fn normalizes_dns_ipv4_ipv6_and_nondefault_ports() {
    let mut package = Package::new();
    package.manifest["permissions"]["network"] = json!([
        "https://example.org",
        "http://example.org",
        "https://example.org:8443",
        "https://[0:0:0:0:0:0:0:1]",
        "https://127.0.0.1",
        "https://xn--fsqu00a.test"
    ]);
    let source = package.check().unwrap();
    assert_eq!(
        source.requested_origins(),
        [
            "https://example.org",
            "http://example.org",
            "https://example.org:8443",
            "https://[::1]",
            "https://127.0.0.1",
            "https://xn--fsqu00a.test"
        ]
    );
}

#[test]
fn semantic_duplicate_origins_are_rejected() {
    let schema: Value = serde_json::from_slice(
        &fs::read(repository().join("spec/schema/manifest.schema.json")).unwrap(),
    )
    .unwrap();
    let validator = jsonschema::draft202012::new(&schema).unwrap();
    for origins in [
        json!(["https://[::1]", "https://[0:0:0:0:0:0:0:1]"]),
        json!(["https://127.0.0.1", "https://0x7f000001"]),
        json!(["https://127.0.0.1", "https://127.1"]),
    ] {
        let mut package = Package::new();
        package.manifest["permissions"]["network"] = origins;
        assert!(validator.is_valid(&package.manifest));
        fails(package.check(), E::InvalidManifest);
    }
}

#[test]
fn invalid_and_missing_roots_manifest_and_entry_have_distinct_categories() {
    let package = Package::new();
    fails(
        load(package.dir.path().join("missing"), LoadLimits::default()),
        E::InvalidManifest,
    );
    fails(
        load(package.dir.path(), LoadLimits::default()),
        E::InvalidManifest,
    );
    package.write();
    fs::remove_file(package.dir.path().join("source.json")).unwrap();
    fails(
        load(package.dir.path(), LoadLimits::default()),
        E::InvalidEntry,
    );
    fs::create_dir(package.dir.path().join("source.json")).unwrap();
    fails(
        load(package.dir.path(), LoadLimits::default()),
        E::InvalidEntry,
    );
}

#[test]
fn rejects_lexical_escapes_and_accepts_contained_nested_entry() {
    let mut package = Package::new();
    for entry in [
        "../outside.json",
        "/outside.json",
        "sub/../../outside.json",
        "sub\\outside.json",
        "sub/%2e%2e/outside.json",
    ] {
        package.manifest["entry"] = json!(entry);
        fails(package.check(), E::InvalidManifest);
    }
    package.manifest["entry"] = json!("nested/data.json");
    fs::create_dir(package.dir.path().join("nested")).unwrap();
    fs::write(
        package.dir.path().join("nested/data.json"),
        package.entry.to_string(),
    )
    .unwrap();
    package.check().unwrap();
}

#[cfg(unix)]
#[test]
fn symlink_containment_checks_components_for_manifest_entry_and_directories() {
    use std::os::unix::fs::symlink;
    let parent = tempfile::tempdir().unwrap();
    let root = parent.path().join("source");
    let sibling = parent.path().join("source-outside");
    fs::create_dir(&root).unwrap();
    fs::create_dir(&sibling).unwrap();
    let mut manifest = example("manifest.json");
    fs::write(root.join("manifest.json"), manifest.to_string()).unwrap();
    fs::write(
        sibling.join("source.json"),
        example("source.json").to_string(),
    )
    .unwrap();
    symlink(sibling.join("source.json"), root.join("source.json")).unwrap();
    fails(load(&root, LoadLimits::default()), E::InvalidEntry);
    fs::remove_file(root.join("source.json")).unwrap();
    symlink(&sibling, root.join("nested")).unwrap();
    manifest["entry"] = json!("nested/source.json");
    fs::write(root.join("manifest.json"), manifest.to_string()).unwrap();
    fails(load(&root, LoadLimits::default()), E::InvalidEntry);
    fs::remove_file(root.join("nested")).unwrap();
    fs::create_dir(root.join("data")).unwrap();
    fs::write(
        root.join("data/source.json"),
        example("source.json").to_string(),
    )
    .unwrap();
    symlink(root.join("data"), root.join("nested")).unwrap();
    load(&root, LoadLimits::default()).unwrap();
    fs::remove_file(root.join("manifest.json")).unwrap();
    fs::write(sibling.join("manifest.json"), manifest.to_string()).unwrap();
    symlink(sibling.join("manifest.json"), root.join("manifest.json")).unwrap();
    fails(load(&root, LoadLimits::default()), E::InvalidManifest);
}

#[test]
fn entry_json_rejects_duplicates_malformed_unicode_and_nonfinite_values() {
    let package = Package::new();
    package.write();
    for raw in [
        b"{".as_slice(),
        b"{} {}",
        br#"{"home":{"items":[],"items":[],"categories":[]}}"#,
        br#"{"home":{"items":[],"\u0069tems":[],"categories":[]}}"#,
        br#"{"x":"\udfff"}"#,
        b"{\"x\":\"\xff\"}",
        b"{\"x\":Infinity}",
    ] {
        fs::write(package.dir.path().join("source.json"), raw).unwrap();
        fails(
            load(package.dir.path(), LoadLimits::default()),
            E::InvalidEntry,
        );
    }
}

#[test]
fn rejects_incorrect_operation_declarations_and_unknown_fields() {
    let mut package = Package::new();
    for entry in [
        json!({}),
        json!(null),
        json!([]),
        json!({"home": {"categories": [], "items": []}}),
    ] {
        package.entry = entry;
        fails(package.check(), E::InvalidEntry);
    }
    package.entry = example("source.json");
    package.entry["extra"] = json!({});
    fails(package.check(), E::InvalidEntry);
    package.entry = example("source.json");
    package.entry["home"]["extra"] = json!(true);
    fails(package.check(), E::InvalidEntry);
}

#[test]
fn rejects_shape_violations_in_every_stored_type() {
    let mutations = [
        ("/home/categories", json!(null)),
        ("/home/items", json!({})),
        ("/home/categories/0/name", json!("")),
        ("/home/items/0/title", json!(5)),
        ("/category/all", json!({"items": []})),
        ("/search/demo", json!(null)),
        ("/detail/demo/id", json!("other")),
        ("/detail/demo/description", json!(null)),
        ("/detail/demo/playables", json!({})),
        ("/detail/demo/playables/0/title", json!("")),
        ("/play/demo-main/url", json!(null)),
        ("/play/demo-main/mimeType", json!("not-a-type")),
    ];
    for (pointer, value) in mutations {
        let mut package = Package::new();
        *package.entry.pointer_mut(pointer).unwrap() = value;
        fails(package.check(), E::InvalidEntry);
    }
}

#[test]
fn broken_references_from_every_operation_are_rejected() {
    for pointer in [
        "/home/categories/0/id",
        "/home/items/0/id",
        "/category/all/0/id",
        "/search/demo/0/id",
        "/detail/demo/playables/0/id",
    ] {
        let mut package = Package::new();
        *package.entry.pointer_mut(pointer).unwrap() = json!("missing");
        fails(package.check(), E::InvalidEntry);
    }
}

#[test]
fn duplicate_array_ids_and_inconsistent_reused_playables_are_rejected() {
    for pointer in [
        "/home/categories",
        "/home/items",
        "/category/all",
        "/search/demo",
        "/detail/demo/playables",
    ] {
        let mut package = Package::new();
        let list = package
            .entry
            .pointer_mut(pointer)
            .unwrap()
            .as_array_mut()
            .unwrap();
        list.push(list[0].clone());
        fails(package.check(), E::InvalidEntry);
    }
    let mut package = Package::new();
    package.entry["detail"]["second"] = package.entry["detail"]["demo"].clone();
    package.entry["detail"]["second"]["id"] = json!("second");
    package.check().unwrap(); // Same playable item reused across details is valid.
    package.entry["detail"]["second"]["playables"][0]["title"] = json!("different");
    fails(package.check(), E::InvalidEntry);
}

#[test]
fn absent_target_operations_unreferenced_records_and_opaque_keys_are_valid() {
    let mut package = Package::new();
    package.manifest["capabilities"]["operations"] = json!(["home", "search"]);
    package.entry = json!({"home": {"categories": [{"id": "url", "name": "Category"}],
        "items": [{"id": "__proto__", "title": "Unresolved without detail"}]},
        "search": {"constructor": [], " demo ": [], "\u{00a0}": []}});
    let source = package.check().unwrap();
    assert!(source.entry()["search"].get(" demo ").is_some());
    for key in ["", " \t\r\n"] {
        package.entry["search"][key] = json!([]);
        fails(package.check(), E::InvalidEntry);
        package.entry["search"].as_object_mut().unwrap().remove(key);
    }
}

#[test]
fn a1_loads_unrequested_resources_and_posters_without_grants_or_network() {
    let mut package = Package::new();
    package.manifest["permissions"]["network"] = json!([]);
    package.entry["home"]["items"][0]["poster"] =
        json!("https://unknown.example.invalid/poster.jpg");
    package.entry["detail"]["demo"]["poster"] = json!("https://other.example.invalid/poster.jpg");
    package.check().unwrap();
}

#[test]
fn validates_all_stored_urls_including_unreferenced_data() {
    for raw in [
        "/relative.mp4",
        "ftp://example.org/a",
        "https://u:p@example.org/a",
        "https://@example.org/a",
        "https://[:::]/a",
        "https://example.org:99999/a",
        "https:///a",
        "https://bad..example.org/a",
        "https://example.org/a\n",
        "https://example.org\\a",
    ] {
        for pointer in [
            "/play/demo-main/url",
            "/home/items/0/poster",
            "/detail/demo/poster",
        ] {
            let mut package = Package::new();
            package.entry["home"]["items"][0]["poster"] = json!("https://example.org/p");
            package.entry["detail"]["demo"]["poster"] = json!("https://example.org/p");
            *package.entry.pointer_mut(pointer).unwrap() = json!(raw);
            fails(package.check(), E::InvalidEntry);
        }
    }
    let mut package = Package::new();
    package.entry["play"]["unreferenced"] = json!({"url":"javascript:alert(1)"});
    fails(package.check(), E::InvalidEntry);
}

#[test]
fn accepts_resource_default_ports_and_parser_normalization_without_rewriting_data() {
    for raw in [
        "https://EXAMPLE.org:443/a",
        "https://example.org./a",
        "http://example.org:80/a",
        "https://[::1]:8443/a",
        "https://例子.test/a",
    ] {
        let mut package = Package::new();
        package.entry["play"]["demo-main"]["url"] = json!(raw);
        let loaded = package.check().unwrap();
        assert_eq!(loaded.entry()["play"]["demo-main"]["url"], raw);
    }
}

#[test]
fn header_validation_and_media_types() {
    let mut package = Package::new();
    for headers in [
        json!({"Host":"x"}),
        json!({"cOoKiE":"x"}),
        json!({"Content-Length":"1"}),
        json!({"Connection":"x"}),
        json!({"Transfer-Encoding":"x"}),
        json!({"Set-Cookie":"x"}),
        json!({"X-A":"1", "x-a":"2"}),
        json!({"bad name":"x"}),
        json!({"":"x"}),
        json!({"X-A":"x\r\ny"}),
        json!({"X-A":1}),
        json!(null),
    ] {
        package.entry["play"]["demo-main"]["headers"] = headers;
        fails(package.check(), E::InvalidEntry);
    }
    package.entry["play"]["demo-main"]["headers"] =
        json!({"Authorization":"private", "X-Empty":""});
    package.entry["play"]["demo-main"]["mimeType"] = json!("video/mp4; codecs=avc1");
    let source = package.check().unwrap();
    assert!(!format!("{source:?}").contains("private"));
    for mime in ["", "garbage", "video/*", "video/mp4\r\nX: y"] {
        package.entry["play"]["demo-main"]["mimeType"] = json!(mime);
        fails(package.check(), E::InvalidEntry);
    }
}

#[test]
fn independent_snapshots_cannot_mutate_each_other_or_follow_file_changes() {
    let mut package = Package::new();
    let first = package.check().unwrap();
    package.entry["home"]["items"][0]["title"] = json!("changed");
    let second = package.check().unwrap();
    assert_eq!(first.entry()["home"]["items"][0]["title"], "Demo video");
    assert_eq!(second.entry()["home"]["items"][0]["title"], "changed");
}

#[test]
fn byte_limits_accept_exact_boundary_and_reject_one_over() {
    let package = Package::new();
    package.write();
    let manifest_bytes = fs::metadata(package.dir.path().join("manifest.json"))
        .unwrap()
        .len() as usize;
    let entry_bytes = fs::metadata(package.dir.path().join("source.json"))
        .unwrap()
        .len() as usize;
    let limits = LoadLimits {
        manifest_bytes,
        entry_bytes,
        package_bytes: manifest_bytes + entry_bytes,
        ..LoadLimits::default()
    };
    load(package.dir.path(), limits).unwrap();
    for limited in [
        LoadLimits {
            manifest_bytes: manifest_bytes - 1,
            ..limits
        },
        LoadLimits {
            entry_bytes: entry_bytes - 1,
            ..limits
        },
        LoadLimits {
            package_bytes: limits.package_bytes - 1,
            ..limits
        },
    ] {
        fails(load(package.dir.path(), limited), E::ResourceLimit);
    }
}

#[test]
fn depth_budget_ignores_braces_in_strings_and_has_hard_ceilings() {
    let mut package = Package::new();
    package.entry["detail"]["demo"]["description"] = json!("[[[[{{{{ escaped \" text");
    package.check().unwrap();
    fails(
        load(
            package.dir.path(),
            LoadLimits {
                json_depth: 3,
                ..LoadLimits::default()
            },
        ),
        E::ResourceLimit,
    );
    let deep = format!("{}0{}", "[".repeat(65), "]".repeat(65));
    fs::write(package.dir.path().join("source.json"), deep).unwrap();
    fails(
        load(package.dir.path(), LoadLimits::default()),
        E::ResourceLimit,
    );
    for limits in [
        LoadLimits {
            json_depth: 65,
            ..LoadLimits::default()
        },
        LoadLimits {
            manifest_bytes: 0,
            ..LoadLimits::default()
        },
        LoadLimits {
            package_bytes: usize::MAX,
            ..LoadLimits::default()
        },
    ] {
        fails(load(package.dir.path(), limits), E::ResourceLimit);
    }
}
