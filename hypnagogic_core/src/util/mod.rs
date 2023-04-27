use toml::map::Map;
use toml::Value;

pub mod adjacency;
pub mod corners;
pub mod icon_ops;

#[tracing::instrument]
pub(crate) fn deep_merge_toml(first: &mut Value, second: Value) {
    match (first, second) {
        (first @ &mut Value::Table(_), Value::Table(second)) => {
            let first = first.as_table_mut().unwrap();
            for (k, v) in second {
                deep_merge_toml(first.entry(k).or_insert(Value::Table(Map::new())), v);
            }
        }
        (first, second) => *first = second,
    }
}

#[must_use]
pub fn repeat_for<T: Clone>(to_repeat: &[T], amount: usize) -> Vec<T> {
    to_repeat.iter().cycle().take(amount).cloned().collect()
}

#[cfg(test)]
mod test {

    use crate::util::deep_merge_toml;
    use toml::Value;

    #[test]
    fn deep_merge_simple() {
        let left_string = r#"
            foo = "left"
            bar = "left"
            "#;

        let mut left: Value = toml::from_str(left_string).unwrap();

        let right_string = r#"
            bar = "right"
            baz = "right"
            "#;

        let right: Value = toml::from_str(right_string).unwrap();

        deep_merge_toml(&mut left, right);

        let expected_string = r#"
            foo = "left"
            bar = "right"
            baz = "right"
            "#;
        let expected: Value = toml::from_str(expected_string).unwrap();

        assert_eq!(left, expected);
    }

    #[test]
    fn deep_merge_1_layer() {
        let left_string = r#"
            foo = "left"
            bar = "left"
            
            [inner]
            foo = "left"
            bar = "left"
            "#;

        let mut left: Value = toml::from_str(left_string).unwrap();

        let right_string = r#"
            bar = "right"
            baz = "right"
            
            [inner]
            bar = "right"
            baz = "right"
            "#;

        let right: Value = toml::from_str(right_string).unwrap();

        deep_merge_toml(&mut left, right);

        let expected_string = r#"
            foo = "left"
            bar = "right"
            baz = "right"
            
            [inner]
            foo = "left"
            bar = "right"
            baz = "right"
            "#;
        let expected: Value = toml::from_str(expected_string).unwrap();

        assert_eq!(left, expected);
    }

    #[test]
    fn deep_merge_3_layer() {
        let left_string = r#"
            foo = "left"
            bar = "left"
            
            [inner.foo1]
            foo2 = { foo = "left", bar = "left" }
            bar2 = { foo = "left", bar = "left" }
            
            [inner.bar1]
            foo2 = { foo = "left", bar = "left" }
            bar2 = { foo = "left", bar = "left" }
            "#;

        let mut left: Value = toml::from_str(left_string).unwrap();

        let right_string = r#"
            bar = "right"
            baz = "right"
            
            [inner.bar1]
            bar2 = { bar = "right", baz = "right"}
            baz2 = { bar = "right", baz = "right"}

            [inner.baz1]
            bar2 = { bar = "right", baz = "right"}
            baz2 = { bar = "right", baz = "right"}
            "#;

        let right: Value = toml::from_str(right_string).unwrap();

        deep_merge_toml(&mut left, right);

        let expected_string = r#"
            foo = "left"
            bar = "right"
            baz = "right"
            
            [inner.foo1]
            foo2 = { foo = "left", bar = "left" }
            bar2 = { foo = "left", bar = "left" }
            [inner.bar1]
            foo2 = { foo = "left", bar = "left" }
            bar2 = { foo = "left", bar = "right", baz = "right"}
            baz2 = { bar = "right", baz = "right"}
            [inner.baz1]
            bar2 = { bar = "right", baz = "right"}
            baz2 = { bar = "right", baz = "right"}
            "#;
        let expected: Value = toml::from_str(expected_string).unwrap();

        assert_eq!(left, expected);
    }
}
