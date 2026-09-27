mod support;
use serde_json::{Value, json};
use std::{collections::HashSet, fs, path::Path};
use universal_source_runtime::{EffectiveGrants, LoadLimits, load};

#[test]
fn execute_authored_declarative_cases() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .canonicalize()
        .unwrap();
    let corpus: Value =
        serde_json::from_slice(&fs::read(root.join("conformance/declarative/cases.json")).unwrap())
            .unwrap();
    let schema: Value = serde_json::from_slice(
        &fs::read(root.join("conformance/declarative/cases.schema.json")).unwrap(),
    )
    .unwrap();
    jsonschema::draft202012::new(&schema)
        .unwrap()
        .validate(&corpus)
        .unwrap();
    let cases = corpus["cases"].as_array().unwrap();
    assert_eq!(cases.len(), 34, "review coverage when corpus changes");
    let mut ids = HashSet::new();
    let mut failures = Vec::new();
    for case in cases {
        let id = case["id"].as_str().unwrap();
        assert!(ids.insert(id), "duplicate case {id}");
        // A fresh production load per case; setup failures are test failures,
        // never converted into operation error envelopes.
        let checked = std::panic::catch_unwind(|| {
            let manifest = root
                .join(
                    corpus["sources"][case["source"].as_str().unwrap()]["manifest"]
                        .as_str()
                        .unwrap(),
                )
                .canonicalize()
                .expect("source path setup");
            assert!(manifest.starts_with(&root));
            let source = load(manifest.parent().unwrap(), LoadLimits::default())
                .expect("loader setup failed");
            assert_eq!(case["grants"]["cookies"], false);
            assert_eq!(case["grants"]["storage"], false);
            let network = case["grants"]["network"]
                .as_array()
                .unwrap()
                .iter()
                .map(|v| v.as_str().unwrap().to_owned())
                .collect::<Vec<_>>();
            let grants = EffectiveGrants::new(&source, &network).expect("grant setup failed");
            let actual =
                source.invoke(case["operation"].as_str().unwrap(), &case["input"], &grants);
            support::envelope(&actual);
            if let Some(expected) = case["expect"].get("result") {
                assert!(
                    support::equal(&actual, expected),
                    "expected {expected}, actual {actual}"
                );
            } else {
                assert_eq!(actual["ok"], false);
                assert_eq!(actual["error"]["code"], case["expect"]["errorCode"]);
            }
        });
        if checked.is_err() {
            failures.push(id);
            eprintln!("FAIL {id}");
        } else {
            println!("PASS {id}");
        }
    }
    assert!(failures.is_empty(), "failed case IDs: {failures:?}");
    println!("34/34 authored declarative cases passed");
}

#[test]
fn comparison_preserves_types_arrays_and_missing_fields() {
    assert!(support::equal(
        &json!({"a":1,"b":[2.0]}),
        &json!({"b":[2],"a":1.0})
    ));
    for (a, b) in [
        (json!(true), json!(1)),
        (json!("1"), json!(1)),
        (json!({}), json!({"a":null})),
        (json!([1, 2]), json!([2, 1])),
        (json!(9007199254740993u64), json!(9007199254740992.0)),
    ] {
        assert!(!support::equal(&a, &b));
    }
}
