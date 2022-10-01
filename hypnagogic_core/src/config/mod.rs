use std::io::{Read, Seek};

use serde::{Deserialize, Serialize};
use serde_yaml::{Mapping, Value};
use tracing::{debug, trace};

use template_resolver::TemplateResolver;

use crate::config::error::ConfigResult;
use crate::config::template_resolver::error::TemplateResult;
use crate::modes::CutterMode;
use crate::util::deep_merge_yaml;

pub mod error;
pub mod template_resolver;

pub const LATEST_VERSION: &str = "1";

#[derive(Copy, Clone, Default, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub struct CornersData<T> {
    pub southeast: T,
    pub northwest: T,
    pub northeast: T,
    pub southwest: T,
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Config {
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub file_prefix: Option<String>,
    pub mode: CutterMode,
}

impl Config {
    /// Load a config from a reader and provide a collapsed, template resolved config back.
    /// # Errors
    /// Returns an error if serde fails to load from the reader
    #[tracing::instrument(skip(resolver, input))]
    pub fn load<R: Read + Seek>(
        input: &mut R,
        resolver: impl TemplateResolver,
    ) -> ConfigResult<Config> {
        let config = serde_yaml::from_reader(input)?;

        let result_value = resolve_templates(config, resolver)?;

        let out_config: Self = serde_yaml::from_value(result_value)?;
        debug!(config = ?out_config, "Deserialized");
        Ok(out_config)
    }
}

/// Seeks out template string from a value and returns it as a `Some(String)`
/// If not found, returns `None`
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
pub fn resolve_templates(first: Value, resolver: impl TemplateResolver) -> TemplateResult {
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
            current = resolver.resolve(template.as_str())?;
            extracted_template = extract_template_string(&mut current);
            trace!(value = ?current, "Resolved config");
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
    use serde_yaml::{Mapping, Value};

    use super::*;

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
        fn resolve(&self, input: &str) -> TemplateResult {
            let first_string = "---
                template: second
                second: 2
                third: 2
            ";

            let second_string = "---
                first: 3
                second: 3
                third: 3
                fourth: 3
            ";

            let fourth_string = "---
                template: fifth
                first: 4
                second: 4
                third: 4
                inner:
                    inner1: 4
                    inner2: 4
            ";

            let fifth_string = "---
                first: 5
                second: 5
                third: 5
                inner:
                    inner1: 5
                    inner2: 5
                    inner3: 5
            ";

            Ok(serde_yaml::from_str(match input {
                "first" => first_string,
                "second" => second_string,
                "fourth" => fourth_string,
                "fifth" => fifth_string,
                _ => panic!("Malformed test"),
            })
            .unwrap())
        }
    }

    mod config_templates {
        use serde_yaml::Value;

        use crate::config::resolve_templates;

        use super::*;

        #[test]
        fn flattening_simple() {
            let input_string = "---
                template: first
                first: 1
                second: 1
            ";
            let input: Value = serde_yaml::from_str(input_string).unwrap();

            let result = resolve_templates(input, TestResolver).unwrap();

            let expected_string = "---
                first: 1
                second: 1
                third: 2
                fourth: 3
            ";
            let expected: Value = serde_yaml::from_str(expected_string).unwrap();
            assert_eq!(result, expected);
        }

        #[test]
        fn flattening_complex() {
            let input_string = "---
                template: fourth
                first: 1
                second: 1
                inner:
                    inner1: 1
            ";

            let input: Value = serde_yaml::from_str(input_string).unwrap();

            let result = resolve_templates(input, TestResolver).unwrap();

            let expected_string = "---
                first: 1
                second: 1
                third: 4
                inner:
                    inner1: 1
                    inner2: 4
                    inner3: 5
            ";
            let expected_value: Value = serde_yaml::from_str(expected_string).unwrap();
            assert_eq!(result, expected_value);
        }
    }

    mod config {
        use std::io::Cursor;

        use crate::config::template_resolver::NullResolver;
        use crate::modes::cutters::bitmask_slice::BitmaskSlice;

        use super::*;

        #[test]
        fn symmetrical_serialize() {
            let config = Config {
                file_prefix: None,
                mode: BitmaskSlice::default().into(),
            };
            let config_string = serde_yaml::to_string(&config).unwrap();

            let mut reader = Cursor::new(&config_string);

            let result = Config::load(&mut reader, NullResolver).unwrap();

            assert_eq!(result, config);
        }
    }
}
