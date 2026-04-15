use std::collections::HashMap;
use url::form_urlencoded;

pub fn parse_query_string(query: &str) -> HashMap<String, String> {
    let mut params = HashMap::new();
    for (key, value) in form_urlencoded::parse(query.as_bytes()) {
        params.insert(key.to_string(), value.to_string());
    }
    params
}

pub fn build_query_string(params: &HashMap<String, String>) -> String {
    let encoded: Vec<String> = params
        .iter()
        .map(|(k, v)| {
            form_urlencoded::byte_serialize(k.as_bytes()).collect::<String>()
                + "="
                + &form_urlencoded::byte_serialize(v.as_bytes()).collect::<String>()
        })
        .collect();
    encoded.join("&")
}

pub fn get_query_param(query: &str, key: &str) -> Option<String> {
    parse_query_string(query).get(key).cloned()
}

pub fn set_query_param(url: &str, key: &str, value: &str) -> String {
    if let Some(pos) = url.find('?') {
        let (base, query) = url.split_at(pos);
        let mut params = parse_query_string(query.trim_start_matches('?'));
        params.insert(key.to_string(), value.to_string());
        format!("{}?{}", base, build_query_string(&params))
    } else {
        format!("{}?{}={}", url, key, value)
    }
}

pub fn remove_query_param(url: &str, key: &str) -> String {
    if let Some(pos) = url.find('?') {
        let (base, query) = url.split_at(pos);
        let mut params = parse_query_string(query.trim_start_matches('?'));
        params.remove(key);
        if params.is_empty() {
            base.to_string()
        } else {
            format!("{}?{}", base, build_query_string(&params))
        }
    } else {
        url.to_string()
    }
}
