use serde_json::{Map, Value};
use std::collections::{HashMap, HashSet};

use crate::urls;

type Check<T> = Result<T, ()>;
fn object(value: &Value) -> Check<&Map<String, Value>> {
    value.as_object().ok_or(())
}
fn array(value: &Value) -> Check<&Vec<Value>> {
    value.as_array().ok_or(())
}
fn text(value: &Value) -> Check<&str> {
    value.as_str().filter(|s| !s.is_empty()).ok_or(())
}
fn fields<'a>(
    value: &'a Value,
    required: &[&str],
    optional: &[&str],
) -> Check<&'a Map<String, Value>> {
    let obj = object(value)?;
    if required.iter().any(|key| !obj.contains_key(*key))
        || obj
            .keys()
            .any(|key| !required.contains(&key.as_str()) && !optional.contains(&key.as_str()))
    {
        return Err(());
    }
    Ok(obj)
}
fn poster(obj: &Map<String, Value>) -> Check<()> {
    if let Some(value) = obj.get("poster") {
        urls::resource(text(value)?)?;
    }
    Ok(())
}
fn records<'a>(values: &'a Value, label: &str, optional: &[&str]) -> Check<Vec<&'a str>> {
    let mut seen = HashSet::new();
    let mut ids = Vec::new();
    for value in array(values)? {
        let obj = fields(value, &["id", label], optional)?;
        let id = text(&obj["id"])?;
        text(&obj[label])?;
        if !seen.insert(id) {
            return Err(());
        }
        poster(obj)?;
        ids.push(id);
    }
    Ok(ids)
}
fn references(ids: &[&str], target: Option<&Map<String, Value>>) -> Check<()> {
    if let Some(target) = target
        && ids.iter().any(|id| !target.contains_key(*id))
    {
        return Err(());
    }
    Ok(())
}
fn token(name: &str) -> bool {
    !name.is_empty()
        && name
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b"!#$%&'*+-.^_`|~".contains(&b))
}
fn resource(value: &Value) -> Check<()> {
    let obj = fields(value, &["url"], &["headers", "mimeType"])?;
    urls::resource(text(&obj["url"])?)?;
    if let Some(value) = obj.get("mimeType") {
        let mime = text(value)?.parse::<mime::Mime>().map_err(|_| ())?;
        // A concrete media type, not an Accept header wildcard range.
        if mime.type_() == mime::STAR || mime.subtype() == mime::STAR {
            return Err(());
        }
    }
    if let Some(value) = obj.get("headers") {
        let mut names = HashSet::new();
        for (name, value) in object(value)? {
            let name_lower = name.to_ascii_lowercase();
            if !token(name)
                || !names.insert(name_lower.clone())
                || [
                    "host",
                    "content-length",
                    "connection",
                    "transfer-encoding",
                    "cookie",
                    "set-cookie",
                ]
                .contains(&name_lower.as_str())
                || value.as_str().ok_or(())?.contains(['\r', '\n'])
            {
                return Err(());
            }
        }
    }
    Ok(())
}

pub(crate) fn validate(entry: &Value, operations: &[Value]) -> Check<()> {
    let root = object(entry)?;
    let declared: HashSet<_> = operations
        .iter()
        .map(|v| v.as_str().ok_or(()))
        .collect::<Check<_>>()?;
    if root.len() != declared.len() || root.keys().any(|key| !declared.contains(key.as_str())) {
        return Err(());
    }
    let mut maps = HashMap::new();
    for op in ["category", "search", "detail", "play"] {
        if let Some(value) = root.get(op) {
            let map = object(value)?;
            for key in map.keys() {
                if key.is_empty()
                    || (op == "search"
                        && key.chars().all(|c| matches!(c, ' ' | '\t' | '\r' | '\n')))
                {
                    return Err(());
                }
            }
            maps.insert(op, map);
        }
    }
    let detail = maps.get("detail").copied();
    let play = maps.get("play").copied();
    if let Some(value) = root.get("home") {
        let home = fields(value, &["categories", "items"], &[])?;
        references(
            &records(&home["categories"], "name", &[])?,
            maps.get("category").copied(),
        )?;
        references(&records(&home["items"], "title", &["poster"])?, detail)?;
    }
    for op in ["category", "search"] {
        if let Some(map) = maps.get(op) {
            for values in map.values() {
                references(&records(values, "title", &["poster"])?, detail)?;
            }
        }
    }
    let mut playables = HashMap::new();
    if let Some(details) = detail {
        for (key, value) in details {
            let obj = fields(
                value,
                &["id", "title", "playables"],
                &["poster", "description"],
            )?;
            if text(&obj["id"])? != key {
                return Err(());
            }
            text(&obj["title"])?;
            poster(obj)?;
            if let Some(value) = obj.get("description") {
                value.as_str().ok_or(())?;
            }
            let ids = records(&obj["playables"], "title", &[])?;
            references(&ids, play)?;
            for playable in array(&obj["playables"])? {
                let id = text(&playable["id"])?;
                if let Some(previous) = playables.insert(id, playable)
                    && previous != playable
                {
                    return Err(());
                }
            }
        }
    }
    if let Some(resources) = play {
        for value in resources.values() {
            resource(value)?;
        }
    }
    Ok(())
}
