use crate::{LoadedSource, entry, urls};
use serde_json::{Value, json};
use std::{collections::HashSet, fmt};

const OPERATIONS: [&str; 5] = ["home", "category", "search", "detail", "play"];
const MAX_PAGE: u64 = 9_007_199_254_740_991;

/// Immutable host context, separate from Source API input. No ambient grants.
#[derive(Debug, Default)]
pub struct EffectiveGrants {
    network: HashSet<String>,
}

/// Host setup failure, not a Source API error code.
#[derive(Debug, PartialEq, Eq)]
pub struct InvalidGrants;
impl fmt::Display for InvalidGrants {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("invalid effective grants")
    }
}
impl std::error::Error for InvalidGrants {}
impl EffectiveGrants {
    /// Validate exact origin grants and reject authority beyond this source's request.
    /// The static profile has no cookie/storage services or grants.
    pub fn new(source: &LoadedSource, network: &[String]) -> Result<Self, InvalidGrants> {
        let values: Vec<Value> = network.iter().map(|s| json!(s)).collect();
        let network = urls::origins(&values).map_err(|_| InvalidGrants)?;
        if network
            .iter()
            .any(|origin| !source.origins.contains(origin))
        {
            return Err(InvalidGrants);
        }
        Ok(Self {
            network: network.into_iter().collect(),
        })
    }
}

pub(crate) fn error(code: &str) -> Value {
    let message = match code {
        "UNSUPPORTED_OPERATION" => "Operation is not declared",
        "INVALID_ARGUMENT" => "Invalid operation input",
        "NOT_FOUND" => "Unknown identifier",
        "PERMISSION_DENIED" => "Resource origin is not permitted",
        "CANCELLED" => "Call cancelled",
        "TIMEOUT" => "Call deadline expired",
        "RESOURCE_LIMIT" => "Call resource limit",
        "SOURCE_ERROR" => "Source execution failed",
        _ => "Invalid source result",
    };
    json!({"ok": false, "error": {"code": code, "message": message}})
}
fn input_page(operation: &str, input: &Value) -> Result<u64, ()> {
    let obj = input.as_object().ok_or(())?;
    let fields: &[&str] = match operation {
        "home" => &[],
        "category" => &["id", "page"],
        "search" => &["query", "page"],
        _ => &["id"],
    };
    if obj.keys().any(|k| !fields.contains(&k.as_str())) {
        return Err(());
    }
    if operation != "home" {
        let key = if operation == "search" { "query" } else { "id" };
        let text = obj
            .get(key)
            .and_then(Value::as_str)
            .filter(|s| !s.is_empty())
            .ok_or(())?;
        if operation == "search" && text.chars().all(|c| matches!(c, ' ' | '\t' | '\r' | '\n')) {
            return Err(());
        }
    }
    let Some(value) = obj.get("page") else {
        return Ok(1);
    };
    // JSON integer values include 1.0 and 1e0; booleans and strings never coerce.
    let number = value.as_number().ok_or(())?;
    if let Some(n) = number.as_u64() {
        return (1..=MAX_PAGE).contains(&n).then_some(n).ok_or(());
    }
    let n = number.as_f64().ok_or(())?;
    if n.fract() != 0.0 || n < 1.0 || n > MAX_PAGE as f64 {
        return Err(());
    }
    Ok(n as u64)
}

pub(crate) fn declared(source: &LoadedSource, operation: &str) -> bool {
    OPERATIONS.contains(&operation) && source.entry.get(operation).is_some()
}

impl LoadedSource {
    pub(crate) fn invoke_checked(
        &self,
        operation: &str,
        input: &Value,
        grants: &EffectiveGrants,
        check: &impl Fn() -> Result<(), &'static str>,
    ) -> Value {
        if !declared(self, operation) {
            return error("UNSUPPORTED_OPERATION");
        }
        if let Err(code) = check() {
            return error(code);
        }
        let Ok(page) = input_page(operation, input) else {
            return error("INVALID_ARGUMENT");
        };
        let stored = &self.entry[operation];
        let data = match operation {
            "home" => stored.clone(),
            "category" | "search" => {
                let key = if operation == "category" {
                    "id"
                } else {
                    "query"
                };
                let found = stored.get(input[key].as_str().expect("validated input"));
                if operation == "category" && found.is_none() {
                    return error("NOT_FOUND");
                }
                let items = if page == 1 {
                    found.cloned().unwrap_or_else(|| json!([]))
                } else {
                    json!([])
                };
                json!({"items": items, "page": page, "hasMore": false})
            }
            "detail" | "play" => {
                let Some(value) = stored.get(input["id"].as_str().expect("validated input")) else {
                    return error("NOT_FOUND");
                };
                value.clone()
            }
            _ => unreachable!("vocabulary checked"),
        };
        if let Err(code) = check() {
            return error(code);
        }
        self.finish(operation, input, page, data, grants, check)
    }

    fn finish(
        &self,
        operation: &str,
        input: &Value,
        page: u64,
        data: Value,
        grants: &EffectiveGrants,
        check: &impl Fn() -> Result<(), &'static str>,
    ) -> Value {
        // Local implementation order only. U2 is not a portable precedence decision.
        if entry::result(operation, &data, input, page, &self.entry).is_err() {
            return error("INVALID_RESULT");
        }
        if let Err(code) = check() {
            return error(code);
        }
        let allowed = |raw: &Value| {
            if check().is_err() {
                return false;
            }
            raw.as_str()
                .and_then(|s| urls::resource_origin(s).ok())
                .is_some_and(|origin| {
                    self.origins.contains(&origin) && grants.network.contains(&origin)
                })
        };
        let poster = |record: &Value| record.get("poster").is_none_or(&allowed);
        // Visit typed URL-bearing fields only: IDs, descriptions, header values
        // and lookup-map keys named "url" or "poster" are opaque data.
        let permitted = match operation {
            "home" | "category" | "search" => data["items"].as_array().unwrap().iter().all(poster),
            "detail" => poster(&data),
            "play" => allowed(&data["url"]),
            _ => false,
        };
        if let Err(code) = check() {
            return error(code);
        }
        if !permitted {
            return error("PERMISSION_DENIED");
        }
        json!({"ok": true, "data": data})
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn malformed_constructed_results_are_never_delivered() {
        let source = LoadedSource {
            manifest: json!({}),
            entry: json!({"home":{"categories":[],"items":[]}}),
            origins: vec![],
        };
        for data in [
            json!({"categories":[],"items":[],"extra":1}),
            json!({"categories":[],"items":null}),
            json!({"categories":[],"items":[{"id":"x","title":""}]}),
        ] {
            let outcome = source.finish(
                "home",
                &json!({}),
                1,
                data,
                &EffectiveGrants::default(),
                &|| Ok(()),
            );
            assert_eq!(outcome["error"]["code"], "INVALID_RESULT");
            assert!(outcome.get("data").is_none());
        }
    }
    #[test]
    fn result_identity_page_and_header_checks() {
        let source = LoadedSource {
            manifest: json!({}),
            entry: json!({}),
            origins: vec![],
        };
        for (op, input, page, data) in [
            (
                "detail",
                json!({"id":"a"}),
                1,
                json!({"id":"b","title":"B","playables":[]}),
            ),
            (
                "search",
                json!({"query":"a"}),
                2,
                json!({"items":[],"page":1,"hasMore":false}),
            ),
            (
                "play",
                json!({"id":"a"}),
                1,
                json!({"url":"https://example.org/a","headers":{"Host":"x"}}),
            ),
        ] {
            // No simultaneous permission violation expectation is standardized here.
            assert!(entry::result(op, &data, &input, page, &source.entry).is_err());
        }
    }
}
