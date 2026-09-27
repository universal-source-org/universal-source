use std::collections::HashSet;
use url::{Host, Url};

// Parser edge policies are documented in runtime/README.md; no DNS or I/O.
fn http_url(raw: &str) -> Result<Url, ()> {
    if raw.bytes().any(|b| b <= 0x20 || b == 0x7f || b == b'\\') {
        return Err(());
    }
    let (_, rest) = raw.split_once("://").ok_or(())?;
    let authority = rest.split(['/', '?', '#']).next().ok_or(())?;
    if authority.is_empty() || authority.contains('@') {
        return Err(());
    }
    let url = Url::parse(raw).map_err(|_| ())?;
    if !matches!(url.scheme(), "http" | "https") || url.host().is_none() {
        return Err(());
    }
    if let Some(Host::Domain(domain)) = url.host() {
        let domain = domain.strip_suffix('.').unwrap_or(domain);
        if domain.is_empty()
            || domain.len() > 253
            || domain.split('.').any(|label| {
                label.is_empty()
                    || label.len() > 63
                    || label.starts_with('-')
                    || label.ends_with('-')
                    || !label
                        .bytes()
                        .all(|b| b.is_ascii_alphanumeric() || b == b'-')
            })
        {
            return Err(());
        }
    }
    Ok(url)
}

pub(crate) fn resource(raw: &str) -> Result<(), ()> {
    http_url(raw).map(|_| ())
}

pub(crate) fn origins(values: &[serde_json::Value]) -> Result<Vec<String>, ()> {
    let mut seen = HashSet::new();
    let mut normalized = Vec::new();
    for value in values {
        let raw = value.as_str().ok_or(())?;
        let url = http_url(raw)?;
        let (scheme, authority) = raw.split_once("://").ok_or(())?;
        if !matches!(scheme, "http" | "https")
            || !authority.is_ascii()
            || authority.bytes().any(|b| b.is_ascii_uppercase())
            || authority.contains(['/', '?', '#', '%'])
        {
            return Err(());
        }
        let port = if authority.starts_with('[') {
            let end = authority.find(']').ok_or(())?;
            let suffix = &authority[end + 1..];
            if suffix.is_empty() {
                None
            } else {
                Some(suffix.strip_prefix(':').ok_or(())?)
            }
        } else {
            authority.split_once(':').map(|(_, p)| p)
        };
        if let Some(port) = port {
            if port.starts_with('0') || !port.bytes().all(|b| b.is_ascii_digit()) {
                return Err(());
            }
            let port: u16 = port.parse().map_err(|_| ())?;
            if port == 0 || (scheme == "http" && port == 80) || (scheme == "https" && port == 443) {
                return Err(());
            }
        }
        let canonical = normalized_origin(&url)?;
        if !seen.insert(canonical.clone()) {
            return Err(());
        }
        normalized.push(canonical);
    }
    Ok(normalized)
}

fn normalized_origin(url: &Url) -> Result<String, ()> {
    let host = match url.host().ok_or(())? {
        Host::Domain(name) => name.to_owned(),
        Host::Ipv4(ip) => ip.to_string(),
        Host::Ipv6(ip) => format!("[{ip}]"),
    };
    Ok(match url.port() {
        Some(port) => format!("{}://{host}:{port}", url.scheme()),
        None => format!("{}://{host}", url.scheme()),
    })
}

pub(crate) fn resource_origin(raw: &str) -> Result<String, ()> {
    normalized_origin(&http_url(raw)?)
}
