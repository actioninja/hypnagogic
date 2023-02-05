use std::io::{read_to_string, Read, Seek};

use serde::Deserialize;
use toml::map::Map;
use toml::Value;
use tracing::{debug, trace};

use template_resolver::TemplateResolver;

use crate::config::error::ConfigResult;
use crate::config::template_resolver::error::TemplateResult;
use crate::operations::IconOperation;
use crate::util::deep_merge_toml;

pub mod blocks;
pub mod error;
pub mod template_resolver;

pub const LATEST_VERSION: &str = "1";

#[tracing::instrument(skip(resolver, input))]
pub fn read_config<R: Read + Seek>(
    input: &mut R,
    resolver: impl TemplateResolver,
) -> ConfigResult<IconOperation> {
    let reader_string = read_to_string(input)?;
    let toml_value = toml::from_str(&reader_string)?;

    let result_value = resolve_templates(toml_value, resolver)?;

    let out_icon_mode: IconOperation = IconOperation::deserialize(result_value)?;
    debug!(config = ?out_icon_mode, "Deserialized");
    Ok(out_icon_mode)
}

/// Seeks out template string from a value and returns it as a `Some(String)`
/// If not found, returns `None`
/// SIDE EFFECT: removes it from the `Value` if it finds it!
fn extract_template_string(value: &mut Value) -> Option<String> {
    match value {
        Value::Table(table) => {
            if let Some(Value::String(string)) = table.remove("template") {
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
    let mut out: Value = Value::Table(Map::new());
    for conf in stack.iter_mut().rev() {
        deep_merge_toml(&mut out, conf.clone());
        trace!(current = ?out, collapsing = ?conf, "Collapsing value step");
    }

    debug!(collapsed = ?out, "Collapsed value");
    Ok(out)
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn extract_template_test() {
        let mapping = r#"
        template = "found"
        still_there = "junk"
        "#;

        let mut toml_value: Value = toml::from_str(mapping).unwrap();

        let extracted = extract_template_string(&mut toml_value).unwrap();

        let expected = "found".to_string();

        assert_eq!(extracted, expected);

        let expected_mapping = r#"still_there = "junk""#;
        let expected_value: Value = toml::from_str(expected_mapping).unwrap();

        assert_eq!(toml_value, expected_value);
    }

    struct TestResolver;

    impl TemplateResolver for TestResolver {
        fn resolve(&self, input: &str) -> TemplateResult {
            let first_string = r#"
            template = "second"
            second = 1
            third = 1
            "#;

            let second_string = r#"
            first = 2
            second = 2
            third = 2
            fourth = 2
            "#;

            let third_string = r#"
            template = "fourth"
            first = 3
            second = 3
            third = 3
            [inner]
            inner_1 = 3
            inner_2 = 3
            "#;

            let fourth_string = r#"
            first = 4
            second = 4
            third = 4
            [inner]
            inner_1 = 4
            inner_2 = 4
            inner_3 = 4
            "#;

            Ok(toml::from_str(match input {
                "first" => first_string,
                "second" => second_string,
                "third" => third_string,
                "fourth" => fourth_string,
                _ => panic!("Malformed test"),
            })
            .unwrap())
        }
    }

    mod config_templates {
        use crate::config::resolve_templates;

        use super::*;

        #[test]
        fn flattening_simple() {
            let input_string = r#"
            template = "first"
            first = 10
            second = 10
            "#;

            let input: Value = toml::from_str(input_string).unwrap();

            let result = resolve_templates(input, TestResolver).unwrap();

            let expected_string = r#"
            first = 10
            second = 10
            third = 1
            fourth = 2
            "#;
            let expected: Value = toml::from_str(expected_string).unwrap();
            assert_eq!(result, expected);
        }

        #[test]
        fn flattening_complex() {
            let input_string = r#"
            template = "third"
            first = 10
            second = 10
            [inner]
            inner_1 = 10
            "#;

            let input: Value = toml::from_str(input_string).unwrap();

            let result = resolve_templates(input, TestResolver).unwrap();

            let expected_string = r#"
            first = 10
            second = 10
            third = 3
            [inner]
            inner_1 = 10
            inner_2 = 3
            inner_3 = 4
            "#;
            let expected_value: Value = toml::from_str(expected_string).unwrap();
            assert_eq!(result, expected_value);
        }
    }

    mod config {
        use crate::operations::cutters::bitmask_slice::BitmaskSlice;

        use super::*;

        #[test]
        fn symmetrical_serialize() {
            let config: IconOperation = BitmaskSlice::default().into();

            println!("toml:");

            let tomled = toml::to_string(&config).unwrap();
            println!("{tomled}");

            let test_toml = "
                operation = \"BitmaskSlice\"
                produce_dirs = false
                smooth_diagonally = false

                [icon_size]
                x = 32
                y = 32

                [output_icon_pos]
                x = 0
                y = 0

                [output_icon_size]
                x = 32
                y = 32

                [positions]
                concave = 1
                convex = 0
                horizontal = 2
                vertical = 3

                [cut_position]
                x = 16
                y = 16
            ";

            let deserialized: IconOperation = toml::from_str(test_toml).unwrap();
            println!("deserialized");
            println!("{deserialized:#?}");
        }
    }
}
