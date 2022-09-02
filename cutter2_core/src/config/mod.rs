use std::collections::HashMap;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use tracing::debug;
use crate::modes::{CutterMode};

pub const LATEST_VERSION: &str = "1";

#[derive(Copy, Clone, Default, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub struct CornersData<T> {
    pub southeast: T,
    pub northwest: T,
    pub northeast: T,
    pub southwest: T,
}

impl<T> CornersData<T> {

}

#[derive(Copy, Clone, Default, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub struct Sides {
    pub west: u32,
    pub east: u32,
    pub north: u32,
    pub south: u32,
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Config {
    mode: CutterMode,
}

#[derive(Clone, Default, PartialEq, Eq, Debug)]
pub struct TemplatedConfig {
    pub template: Option<String>,
    pub map: HashMap<String, serde_yaml::Value>,
}

#[tracing::instrument(skip(resolver))]
pub fn resolve_templates(first: TemplatedConfig, resolver: impl TemplateResolver) -> Result<HashMap<String, serde_yaml::Value>> {
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
    let mut out = HashMap::new();
    for conf in stack.iter().rev() {
        debug!(collapsing = ?conf, "Collapsing value");
        for (k, v) in &conf.map {
            out.insert(k.clone(), v.clone());
        }
    }
    Ok(out)
}

pub trait TemplateResolver {
    fn resolve(&self, input: &str) -> Result<TemplatedConfig>;
}

#[cfg(test)]
mod test {
    use serde_yaml::Value;
    use super::*;

    struct TestResolver;

    impl TemplateResolver for TestResolver {
        fn resolve(&self, input: &str) -> Result<TemplatedConfig> {
            let mut map1 = HashMap::new();
            map1.insert("second".to_string(), 2.into());
            map1.insert("third".to_string(), 2.into());
            let first = TemplatedConfig {
                template: Some("second".to_string()),
                map: map1,
            };

            let mut map2 = HashMap::new();
            map2.insert("first".to_string(), 3.into());
            map2.insert("second".to_string(), 3.into());
            map2.insert("third".to_string(), 3.into());
            map2.insert("fourth".to_string(), 3.into());
            let second = TemplatedConfig {
                template: None,
                map: map2,
            };

            match input {
                "first" => Ok(first),
                "second" => Ok(second),
                _ => panic!("Malformed test"),
            }
        }
    }

    #[test]
    fn test_flattening() {
        let mut map = HashMap::new();
        map.insert("first".to_string(), 1.into());
        map.insert("second".to_string(), 1.into());
        let test_template = TemplatedConfig {
            template: Some("first".to_string()),
            map,
        };

        let result = resolve_templates(test_template, TestResolver {}).unwrap();
        let mut expected: HashMap<String, Value> = HashMap::new();
        expected.insert("first".to_string(), 1.into());
        expected.insert("second".to_string(), 1.into());
        expected.insert("third".to_string(), 2.into());
        expected.insert("fourth".to_string(), 3.into());
        assert_eq!(result, expected);
    }
}
