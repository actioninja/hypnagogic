pub mod template_resolver;

use crate::modes::CutterMode;
use crate::util::deep_merge_yaml;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use serde_yaml::{Mapping, Value};
use std::io::{Read, Seek};
use template_resolver::TemplateResolver;
use tracing::{debug, trace};

pub const LATEST_VERSION: &str = "1";

#[derive(Copy, Clone, Default, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub struct CornersData<T> {
    pub southeast: T,
    pub northwest: T,
    pub northeast: T,
    pub southwest: T,
}

#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub struct Config {
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub file_prefix: Option<String>,
    pub mode: CutterMode,
}

impl Config {
    /// Load a config from a reader and provide a collapsed, template resolved back.
    /// # Errors
    /// Returns an error if serde fails to load from the reader
    #[tracing::instrument(skip(resolver, input))]
    pub fn load<R: Read + Seek>(input: &mut R, resolver: impl TemplateResolver) -> Result<Self> {
        let config = serde_yaml::from_reader(input)?;

        let result_value = resolve_templates(config, resolver)?;

        let out_config: Self = serde_yaml::from_value(result_value)?;
        debug!(config = ?out_config, "Deserialized");
        Ok(out_config)
    }
}

/// Seeks out template string from a value and returns it as a Some(String)
/// If not found, returns none
/// SIDE EFFECT: removes it from the `Value` if it finds it!
fn extract_template_string(value: &mut Value) -> Option<String> {
    match value {
        Value::Mapping(mapping) => {
            if let Some(Value::String(string)) = mapping.remove("template") {
                Some(string)
            } else {
                None
            }
        }
        _ => None,
    }
}

#[tracing::instrument(skip(resolver))]
pub fn resolve_templates(first: Value, resolver: impl TemplateResolver) -> Result<Value> {
    debug!(first = ?first, "Started resolving templates");
    let mut current = first;
    let mut stack: Vec<Value> = vec![];

    let mut extracted_template = extract_template_string(&mut current);
    trace!(extracted = ?extracted_template, "extracted first template");

    //push the first on to the stack to be resolved
    stack.push(current.clone());
    let mut recursion_cap = 0;
    //Drill in to templates and resolve until no new ones found
    while recursion_cap < 100 {
        if let Some(template) = &extracted_template {
            current = resolver.resolve(template)?;
            extracted_template = extract_template_string(&mut current);
            trace!("Resolved config: {:?}", current);
            stack.push(current.clone());
            recursion_cap += 1;
        } else {
            break;
        }
    }
    trace!(num_in_chain = ?stack.len(), stack = ?stack, "Finished resolving templates");
    //merge stack in to one hashmap
    let mut out: Value = Mapping::new().into();
    for conf in stack.iter().rev() {
        let conf_value: Value = conf.clone();
        deep_merge_yaml(&mut out, conf_value);
        trace!(current = ?out, collapsing = ?conf, "Collapsing value step");
    }

    debug!(collapsed = ?out, "Collapsed value");
    Ok(out)
}

#[cfg(test)]
mod test {
    use super::*;
    use serde_yaml::{Mapping, Value};

    #[test]
    fn extract_template_test() {
        let mut mapping = Mapping::new();
        mapping.insert("template".into(), "found".into());
        mapping.insert("still_there".into(), "junk".into());

        let mut value: Value = mapping.into();

        let extracted = extract_template_string(&mut value).unwrap();

        let expected = "found".to_string();

        assert_eq!(extracted, expected);

        let mut expected_value = Mapping::new();
        expected_value.insert("still_there".into(), "junk".into());
        let expected_value: Value = expected_value.into();

        assert_eq!(value, expected_value);
    }

    struct TestResolver;

    impl TemplateResolver for TestResolver {
        fn resolve(&self, input: &str) -> Result<Value> {
            let mut map1 = Mapping::new();
            map1.insert("template".into(), "second".into());

            map1.insert("second".into(), 2.into());
            map1.insert("third".into(), 2.into());

            let mut map2 = Mapping::new();
            map2.insert("template".into(), Value::Null);

            map2.insert("first".into(), 3.into());
            map2.insert("second".into(), 3.into());
            map2.insert("third".into(), 3.into());
            map2.insert("fourth".into(), 3.into());

            let mut map4 = Mapping::new();
            map4.insert("template".into(), "fifth".into());

            map4.insert("first".into(), 4.into());
            map4.insert("second".into(), 4.into());
            map4.insert("third".into(), 4.into());
            let mut inner_map4 = Mapping::new();
            inner_map4.insert("inner1".into(), 4.into());
            inner_map4.insert("inner2".into(), 4.into());
            map4.insert("inner".into(), inner_map4.into());

            let mut map5 = Mapping::new();
            map5.insert("template".into(), Value::Null);

            map5.insert("first".into(), 5.into());
            map5.insert("second".into(), 5.into());
            map5.insert("third".into(), 5.into());
            let mut inner_map5 = Mapping::new();
            inner_map5.insert("inner1".into(), 5.into());
            inner_map5.insert("inner2".into(), 5.into());
            inner_map5.insert("inner3".into(), 5.into());
            map5.insert("inner".into(), inner_map5.into());

            match input {
                "first" => Ok(map1.into()),
                "second" => Ok(map2.into()),
                "fourth" => Ok(map4.into()),
                "fifth" => Ok(map5.into()),
                _ => panic!("Malformed test"),
            }
        }
    }

    mod config_templates {
        use super::*;
        use crate::config::resolve_templates;
        use serde_yaml::{Mapping, Value};

        #[test]
        fn flattening_simple() {
            let mut map = Mapping::new();
            map.insert("template".into(), "first".into());

            map.insert("first".into(), 1.into());
            map.insert("second".into(), 1.into());

            let result = resolve_templates(map.into(), TestResolver).unwrap();
            let mut expected = Mapping::new();
            expected.insert("first".into(), 1.into());
            expected.insert("second".into(), 1.into());
            expected.insert("third".into(), 2.into());
            expected.insert("fourth".into(), 3.into());
            let value: Value = expected.into();
            assert_eq!(result, value);
        }

        #[test]
        fn flattening_complex() {
            let mut map = Mapping::new();
            map.insert("template".into(), "fourth".into());

            map.insert("first".into(), 1.into());
            map.insert("second".into(), 1.into());
            let mut inner_map = Mapping::new();
            inner_map.insert("inner1".into(), 1.into());

            map.insert("inner".into(), inner_map.into());

            let result = resolve_templates(map.into(), TestResolver).unwrap();

            let mut expected = Mapping::new();
            expected.insert("first".into(), 1.into());
            expected.insert("second".into(), 1.into());
            expected.insert("third".into(), 4.into());
            let mut second_mapping = Mapping::new();
            second_mapping.insert("inner1".into(), 1.into());
            second_mapping.insert("inner2".into(), 4.into());
            second_mapping.insert("inner3".into(), 5.into());
            expected.insert("inner".into(), second_mapping.into());

            let expected_value: Value = expected.into();
            assert_eq!(result, expected_value);
        }
    }

    mod config {
        use super::*;
        use crate::config::template_resolver::NullResolver;
        use crate::modes::cutters::bitmask_slice::BitmaskSlice;
        use std::io::Cursor;

        #[test]
        fn symmetrical_serialize() {
            let config = Config {
                file_prefix: None,
                mode: BitmaskSlice::default().into(),
            };
            let config_string = serde_yaml::to_string(&config).unwrap();
            println!("{}", config_string);

            let mut reader = Cursor::new(&config_string);

            let result = Config::load(&mut reader, NullResolver).unwrap();

            assert_eq!(result, config);
        }
    }
}
