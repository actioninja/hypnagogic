use serde_yaml::Value;

pub mod adjacency;
pub mod corners;
mod helpers;

pub(crate) fn deep_merge_yaml(first: &mut Value, second: Value) {
    match (first, second) {
        (first @ &mut Value::Mapping(_), Value::Mapping(second)) => {
            let first = first.as_mapping_mut().unwrap();
            for (k, v) in second {
                deep_merge_yaml(first.entry(k).or_insert(Value::Null), v);
            }
        }
        (first, second) => *first = second,
    }
}
