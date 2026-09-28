mod support;
use serde_json::{Value, json};
use std::{fs, path::Path};
use universal_source_runtime::{
    CallLimits, Cancellation, EffectiveGrants, Instance, LoadLimits, load,
};

fn example() -> Instance {
    Instance::new(
        load(
            Path::new(env!("CARGO_MANIFEST_DIR")).join("../examples/json/minimal"),
            LoadLimits::default(),
        )
        .unwrap(),
        CallLimits::default(),
    )
    .unwrap()
}
fn custom(operations: Value, entry: Value, origins: Value) -> Instance {
    let directory = tempfile::tempdir().unwrap();
    let mut manifest = example().source().manifest().clone();
    manifest["capabilities"]["operations"] = operations;
    manifest["permissions"]["network"] = origins;
    fs::write(directory.path().join("manifest.json"), manifest.to_string()).unwrap();
    fs::write(directory.path().join("source.json"), entry.to_string()).unwrap();
    Instance::new(
        load(directory.path(), LoadLimits::default()).unwrap(),
        CallLimits::default(),
    )
    .unwrap()
}
fn invoke(source: &Instance, op: &str, input: &Value, grants: &EffectiveGrants) -> Value {
    source
        .invoke(op, input, grants, &Cancellation::default())
        .unwrap()
}
fn call(source: &Instance, op: &str, input: Value) -> Value {
    let output = invoke(source, op, &input, &EffectiveGrants::default());
    support::envelope(&output);
    output
}
fn code(actual: Value, code: &str) {
    assert_eq!(actual["error"]["code"], code, "{actual}");
}

#[test]
fn vocabulary_and_declaration_precede_input() {
    let source = custom(
        json!(["home"]),
        json!({"home":{"categories":[],"items":[]}}),
        json!([]),
    );
    for op in ["search", "Home", "init", ""] {
        code(call(&source, op, json!(null)), "UNSUPPORTED_OPERATION");
    }
}
#[test]
fn closed_inputs_types_and_page_bounds() {
    let source = example();
    for op in ["home", "category", "search", "detail", "play"] {
        for input in [json!(null), json!([]), json!(true), json!({"extra":1})] {
            code(call(&source, op, input), "INVALID_ARGUMENT");
        }
    }
    for page in [
        json!(null),
        json!(true),
        json!("1"),
        json!(0),
        json!(-1),
        json!(1.5),
        json!(9007199254740992u64),
        json!(1e30),
    ] {
        for op in ["category", "search"] {
            let input = if op == "category" {
                json!({"id":"all","page":page})
            } else {
                json!({"query":"demo","page":page})
            };
            code(call(&source, op, input), "INVALID_ARGUMENT");
        }
    }
    let default = call(&source, "category", json!({"id":"all"}));
    assert!(support::equal(
        &default,
        &call(&source, "category", json!({"id":"all","page":1.0}))
    ));
    assert_eq!(
        call(
            &source,
            "category",
            json!({"id":"all","page":9007199254740991u64})
        )["data"]["page"],
        9007199254740991u64
    );
    code(
        call(&source, "category", json!({"id":"missing","page":2})),
        "NOT_FOUND",
    );
}
#[test]
fn exact_search_empty_pages_and_no_hidden_sequence() {
    let source = example();
    assert_eq!(call(&source, "detail", json!({"id":"demo"}))["ok"], true);
    code(
        call(&source, "detail", json!({"id":"missing"})),
        "NOT_FOUND",
    );
    assert_eq!(
        call(&source, "search", json!({"query":"demo"}))["data"]["items"]
            .as_array()
            .unwrap()
            .len(),
        1
    );
    for query in ["Demo", " demo ", "missing", "\u{00a0}"] {
        assert_eq!(
            call(&source, "search", json!({"query":query}))["data"],
            json!({"items":[],"page":1,"hasMore":false})
        );
    }
    for query in ["", " \t\r\n"] {
        code(
            call(&source, "search", json!({"query":query})),
            "INVALID_ARGUMENT",
        );
    }
    assert_eq!(
        call(&source, "search", json!({"query":"demo","page":2}))["data"],
        json!({"items":[],"page":2,"hasMore":false})
    );
}
#[test]
fn play_lookup_before_denial_and_grants_are_per_call() {
    let source = example();
    let grants =
        EffectiveGrants::new(source.source(), source.source().requested_origins()).unwrap();
    code(call(&source, "play", json!({"id":"missing"})), "NOT_FOUND");
    code(
        call(&source, "play", json!({"id":"demo-main"})),
        "PERMISSION_DENIED",
    );
    let permitted = invoke(&source, "play", &json!({"id":"demo-main"}), &grants);
    support::envelope(&permitted);
    assert_eq!(
        permitted["data"],
        source.source().entry()["play"]["demo-main"]
    );
    code(
        call(&source, "play", json!({"id":"demo-main"})),
        "PERMISSION_DENIED",
    );
    assert_eq!(call(&source, "home", json!({}))["ok"], true);
}
#[test]
fn all_returned_posters_are_checked_but_opaque_text_is_not() {
    let item = json!({"id":"url","title":"poster","poster":"https://images.example.invalid/p"});
    let source = custom(
        json!(["home", "category", "search", "detail"]),
        json!({
            "home":{"categories":[],"items":[item]},"category":{"poster":[item]},"search":{"url":[item]},
            "detail":{"url":{"id":"url","title":"X","playables":[],"poster":"https://images.example.invalid/p","description":"https://unrequested.invalid"}}
        }),
        json!(["https://images.example.invalid"]),
    );
    let grants =
        EffectiveGrants::new(source.source(), source.source().requested_origins()).unwrap();
    for (op, input) in [
        ("home", json!({})),
        ("category", json!({"id":"poster"})),
        ("search", json!({"query":"url"})),
        ("detail", json!({"id":"url"})),
    ] {
        code(call(&source, op, input.clone()), "PERMISSION_DENIED");
        assert_eq!(invoke(&source, op, &input, &grants)["ok"], true);
    }
    assert_eq!(
        call(&source, "category", json!({"id":"poster","page":2}))["ok"],
        true
    );
}
#[test]
fn normalized_origins_exact_matching_and_no_implicit_authority() {
    for (url, origin) in [
        ("https://EXAMPLE.org:443/x", "https://example.org"),
        ("http://example.org:80/x", "http://example.org"),
        ("https://[0:0:0:0:0:0:0:1]/x", "https://[::1]"),
        ("https://example.org:8443/x", "https://example.org:8443"),
        ("https://例子.test/x", "https://xn--fsqu00a.test"),
    ] {
        let source = custom(
            json!(["play"]),
            json!({"play":{"p":{"url":url}}}),
            json!([origin]),
        );
        let grants =
            EffectiveGrants::new(source.source(), source.source().requested_origins()).unwrap();
        assert_eq!(
            invoke(&source, "play", &json!({"id":"p"}), &grants)["data"]["url"],
            url
        );
    }
    for url in [
        "https://sub.example.org/x",
        "http://example.org/x",
        "https://example.org:8443/x",
        "https://example.org.evil/x",
        "https://example.org./x",
    ] {
        let source = custom(
            json!(["play"]),
            json!({"play":{"p":{"url":url}}}),
            json!(["https://example.org"]),
        );
        let grants =
            EffectiveGrants::new(source.source(), source.source().requested_origins()).unwrap();
        code(
            invoke(&source, "play", &json!({"id":"p"}), &grants),
            "PERMISSION_DENIED",
        );
    }
}
#[test]
fn grant_setup_is_separate_and_cannot_bypass_manifest_requests() {
    let source = example();
    for network in [
        vec!["https://unrequested.invalid".into()],
        vec!["*".into()],
        vec!["https://media.example.invalid:443".into()],
        vec!["https://media.example.invalid".into(); 2],
    ] {
        assert!(EffectiveGrants::new(source.source(), &network).is_err());
    }
    let other = custom(
        json!(["play"]),
        json!({"play":{"p":{"url":"https://media.example.invalid/x"}}}),
        json!([]),
    );
    let grants =
        EffectiveGrants::new(source.source(), source.source().requested_origins()).unwrap();
    code(
        invoke(&other, "play", &json!({"id":"p"}), &grants),
        "PERMISSION_DENIED",
    );
}
#[test]
fn results_are_owned_preserve_order_and_do_not_mutate_snapshot() {
    let source = custom(
        json!(["home"]),
        json!({"home":{"categories":[],"items":[{"id":"b","title":"B"},{"id":"a","title":"A"}]}}),
        json!([]),
    );
    let mut first = call(&source, "home", json!({}));
    first["data"]["items"][0]["title"] = json!("changed");
    let next = call(&source, "home", json!({}));
    assert_eq!(next["data"]["items"][0]["title"], "B");
    assert_eq!(next["data"]["items"][1]["id"], "a");
}
