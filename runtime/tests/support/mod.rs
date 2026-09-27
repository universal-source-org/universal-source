use serde_json::{Number, Value};

// Compare JSON numeric values without rounding large integers through f64.
fn decimal(n: &Number) -> (bool, String, i64) {
    let raw = n.to_string();
    let (mantissa, exp) = raw.split_once(['e', 'E']).unwrap_or((&raw, "0"));
    let fraction = mantissa.split_once('.').map_or(0, |(_, f)| f.len());
    let mut digits: String = mantissa.chars().filter(|c| c.is_ascii_digit()).collect();
    digits = digits.trim_start_matches('0').to_owned();
    let mut exponent = exp.parse::<i64>().unwrap() - fraction as i64;
    while digits.ends_with('0') {
        digits.pop();
        exponent += 1;
    }
    if digits.is_empty() {
        return (false, "0".into(), 0);
    }
    (mantissa.starts_with('-'), digits, exponent)
}
pub fn equal(a: &Value, b: &Value) -> bool {
    match (a, b) {
        (Value::Number(a), Value::Number(b)) => decimal(a) == decimal(b),
        (Value::Array(a), Value::Array(b)) => {
            a.len() == b.len() && a.iter().zip(b).all(|(a, b)| equal(a, b))
        }
        (Value::Object(a), Value::Object(b)) => {
            a.len() == b.len() && a.iter().all(|(k, a)| b.get(k).is_some_and(|b| equal(a, b)))
        }
        _ => a == b,
    }
}
pub fn envelope(actual: &Value) {
    let obj = actual.as_object().expect("envelope must be an object");
    assert_eq!(obj.len(), 2, "closed envelope: {actual}");
    match actual["ok"].as_bool().expect("ok must be boolean") {
        true => {
            assert!(obj.contains_key("data"));
            assert!(!obj.contains_key("error"));
        }
        false => {
            assert!(!obj.contains_key("data"));
            let error = actual["error"].as_object().expect("error object");
            assert_eq!(error.len(), 2);
            assert!(
                !error["message"]
                    .as_str()
                    .expect("message string")
                    .is_empty()
            );
            assert!(
                [
                    "INVALID_ARGUMENT",
                    "UNSUPPORTED_OPERATION",
                    "NOT_FOUND",
                    "PERMISSION_DENIED",
                    "NETWORK_ERROR",
                    "TIMEOUT",
                    "CANCELLED",
                    "RESOURCE_LIMIT",
                    "INVALID_RESULT",
                    "SOURCE_ERROR"
                ]
                .contains(&error["code"].as_str().expect("code string"))
            );
        }
    }
}
