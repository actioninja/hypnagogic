use serde_yaml::Value;
use tracing::trace;

pub mod adjacency;
pub mod corners;
mod helpers;

#[tracing::instrument]
pub(crate) fn deep_merge_yaml(first: &mut Value, second: Value) {
    match (first, second) {
        (first @ &mut Value::Mapping(_), Value::Mapping(second)) => {
            trace!(second = ?second, "Found mapping");
            let first = first.as_mapping_mut().unwrap();
            for (k, v) in second {
                deep_merge_yaml(first.entry(k).or_insert(Value::Null), v);
            }
        }
        (Value::Tagged(first), Value::Tagged(second)) => {
            if first.tag == second.tag {
                deep_merge_yaml(&mut first.value, second.value);
            } else {
                *first = second;
            }
        }
        (first, second) => *first = second,
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use serde_yaml::Mapping;

    #[test]
    fn deep_merge_test_simple() {
        let left_string = "---
            foo: left
            bar: left
        ";
        let mut left: Value = serde_yaml::from_str(left_string).unwrap();

        let right_string = "---
            bar: right
            baz: right
        ";
        let right: Value = serde_yaml::from_str(right_string).unwrap();

        deep_merge_yaml(&mut left, right);

        let expected_string = "---
            foo: left
            bar: right
            baz: right
        ";
        let expected: Value = serde_yaml::from_str(expected_string).unwrap();

        assert_eq!(left, expected);
    }

    #[test]
    fn deep_merge_test_one_layer() {
        let left_string = "---
            foo: left
            bar: left
            inner:
                foo: left
                bar: left
        ";
        let mut left = serde_yaml::from_str(left_string).unwrap();

        let right_string = "---
            bar: right
            baz: right
            inner:
                bar: right
                baz: right
        ";
        let right = serde_yaml::from_str(right_string).unwrap();

        deep_merge_yaml(&mut left, right);

        let expected_string = "---
            foo: left
            bar: right
            baz: right
            inner:
                foo: left
                bar: right
                baz: right
        ";
        let expected: Value = serde_yaml::from_str(expected_string).unwrap();

        assert_eq!(left, expected);
    }

    #[test]
    fn deep_merge_test_three_layers() {
        let left_string = "---
            foo: left
            bar: left
            inner:
                foo1:
                    foo2:
                        foo: left
                        bar: left
                    bar2:
                        foo: left
                        bar: left 
                bar1:
                    foo2:
                        foo: left
                        bar: left
                    bar2:
                        foo: left
                        bar: left
        ";
        let mut left = serde_yaml::from_str(left_string).unwrap();

        let right_string = "---
            bar: right
            baz: right
            inner:
                bar1:
                    bar2:
                        bar: right
                        baz: right
                    baz2:
                        bar: right
                        baz: right
                baz1:
                    bar2:
                        bar: right
                        baz: right
                    baz2:
                        bar: right
                        baz: right
        ";
        let right = serde_yaml::from_str(right_string).unwrap();

        deep_merge_yaml(&mut left, right);

        let expected_string = "---
            foo: left
            bar: right
            baz: right
            inner:
                foo1:
                    foo2:
                        foo: left
                        bar: left
                    bar2:
                        foo: left
                        bar: left 
                bar1:
                    foo2:
                        foo: left
                        bar: left
                    bar2:
                        foo: left
                        bar: right
                        baz: right
                    baz2:
                        bar: right
                        baz: right
                baz1:
                    bar2:
                        bar: right
                        baz: right
                    baz2:
                        bar: right
                        baz: right
        ";
        let expected: Value = serde_yaml::from_str(expected_string).unwrap();

        assert_eq!(left, expected);
    }

    #[test]
    fn deep_merge_with_tags() {
        let left_string = "---
            foo: left
            bar: left
            inner: !tagged
                foo: left
                bar: left
        ";
        let mut left = serde_yaml::from_str(left_string).unwrap();

        let right_string = "---
            bar: right
            baz: right
            inner: !tagged
                bar: right
                baz: right
        ";
        let right = serde_yaml::from_str(right_string).unwrap();

        deep_merge_yaml(&mut left, right);

        let expected_string = "---
            foo: left
            bar: right
            baz: right
            inner: !tagged
                foo: left
                bar: right
                baz: right
        ";
        let expected: Value = serde_yaml::from_str(expected_string).unwrap();

        assert_eq!(left, expected);
    }
}
