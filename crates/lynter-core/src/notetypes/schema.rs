use crate::rule::Options;
use serde_json::{Map, Value};

/// The note-type schema the integrator threads in as `options.schema`.
pub fn schema_from(options: &Options) -> Option<&Map<String, Value>> {
    options.get("schema").and_then(Value::as_object)
}

pub fn key_order_from(schema: &Map<String, Value>) -> Vec<String> {
    match schema.get("key-order") {
        Some(Value::Array(a)) => a
            .iter()
            .filter_map(|v| v.as_str().map(str::to_owned))
            .collect(),
        _ => Vec::new(),
    }
}
