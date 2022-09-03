pub mod template_resolver;

use crate::modes::CutterMode;
use crate::util::deep_merge_yaml;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use serde_yaml::{Mapping, Value};
use std::io::{Read, Seek};
use template_resolver::TemplateResolver;
use tracing::debug;

pub const LATEST_VERSION: &str = "1";

#[derive(Copy, Clone, Default, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub struct CornersData<T> {
    pub southeast: T,
    pub northwest: T,
    pub northeast: T,
    pub southwest: T,
}

impl<T> CornersData<T> {}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Config {
    pub file_prefix: Option<String>,
    pub mode: CutterMode,
}

impl Config {
    /// Load a config from a reader and provide a collapsed, template resolved back.
    /// # Errors
    /// Returns an error if serde fails to load from the reader
    pub fn load<R: Read + Seek>(input: &mut R, resolver: impl TemplateResolver) -> Result<Self> {
        let config = serde_yaml::from_reader(input)?;

        let result_value = resolve_templates(config, resolver)?;

        let out_config: Self = serde_yaml::from_value(result_value)?;
        Ok(out_config)
    }
}

#[derive(Clone, Default, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub struct TemplatedConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub template: Option<String>,
    pub map: Mapping,
}

#[tracing::instrument(skip(resolver))]
pub fn resolve_templates(first: TemplatedConfig, resolver: impl TemplateResolver) -> Result<Value> {
    let mut current = first;
    let mut stack: Vec<TemplatedConfig> = vec![];
    //push the first on to the stack to be resolved
    stack.push(current.clone());
    //Drill in to templates and resolve until no new ones found
    while let Some(template) = &current.template {
        if let Ok(found) = resolver.resolve(template) {
            current = found;
        }
        stack.push(current.clone());
    }
    debug!("Found {} templates in chain", stack.len());
    //merge stack in to one hashmap
    let mut out: Value = Mapping::new().into();
    for conf in stack.iter().rev() {
        debug!(collapsing = ?conf, "Collapsing value");
        let conf_value: Value = conf.map.clone().into();
        deep_merge_yaml(&mut out, conf_value);
    }
    Ok(out)
}

#[cfg(test)]
mod test {
    use super::*;
    use serde_yaml::{Mapping, Value};

    struct TestResolver;

    impl TemplateResolver for TestResolver {
        fn resolve(&self, input: &str) -> Result<TemplatedConfig> {
            let mut map1 = Mapping::new();
            map1.insert("second".into(), 2.into());
            map1.insert("third".into(), 2.into());
            let first = TemplatedConfig {
                template: Some("second".to_string()),
                map: map1,
            };

            let mut map2 = Mapping::new();
            map2.insert("first".into(), 3.into());
            map2.insert("second".into(), 3.into());
            map2.insert("third".into(), 3.into());
            map2.insert("fourth".into(), 3.into());
            let second = TemplatedConfig {
                template: None,
                map: map2,
            };

            let mut map4 = Mapping::new();
            map4.insert("first".into(), 4.into());
            map4.insert("second".into(), 4.into());
            map4.insert("third".into(), 4.into());
            let mut inner_map4 = Mapping::new();
            inner_map4.insert("inner1".into(), 4.into());
            inner_map4.insert("inner2".into(), 4.into());
            map4.insert("inner".into(), inner_map4.into());
            let fourth = TemplatedConfig {
                template: Some("fifth".to_string()),
                map: map4,
            };

            let mut map5 = Mapping::new();
            map5.insert("first".into(), 5.into());
            map5.insert("second".into(), 5.into());
            map5.insert("third".into(), 5.into());
            let mut inner_map5 = Mapping::new();
            inner_map5.insert("inner1".into(), 5.into());
            inner_map5.insert("inner2".into(), 5.into());
            inner_map5.insert("inner3".into(), 5.into());
            map5.insert("inner".into(), inner_map5.into());
            let fifth = TemplatedConfig {
                template: None,
                map: map5,
            };

            match input {
                "first" => Ok(first),
                "second" => Ok(second),
                "fourth" => Ok(fourth),
                "fifth" => Ok(fifth),
                _ => panic!("Malformed test"),
            }
        }
    }

    mod config_templates {
        use super::*;
        use crate::config::{resolve_templates, TemplatedConfig};
        use serde_yaml::{Mapping, Value};

        #[test]
        fn flattening_simple() {
            let mut map = Mapping::new();
            map.insert("first".into(), 1.into());
            map.insert("second".into(), 1.into());
            let test_template = TemplatedConfig {
                template: Some("first".to_string()),
                map,
            };

            let result = resolve_templates(test_template, TestResolver {}).unwrap();
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
            map.insert("first".into(), 1.into());
            map.insert("second".into(), 1.into());
            let mut inner_map = Mapping::new();
            inner_map.insert("inner1".into(), 1.into());

            map.insert("inner".into(), inner_map.into());

            let test_template = TemplatedConfig {
                template: Some("fourth".to_string()),
                map,
            };

            let result = resolve_templates(test_template, TestResolver {}).unwrap();

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
}
