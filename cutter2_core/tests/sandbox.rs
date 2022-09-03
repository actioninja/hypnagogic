use cutter2_core::config::Config;
use cutter2_core::modes::cutters::bitmask_slice::BitmaskSlice;
use std::fs;
use std::path::Path;
use tracing::debug;

#[test]
fn test() {
    let slice_config = BitmaskSlice::default();

    let config = Config {
        mode: slice_config.into(),
    };

    let yamlified = serde_yaml::to_value(&config).expect("Failed");
    println!("{:?}", yamlified);
    let serialized = serde_yaml::to_string(&config).expect("Failed");

    fs::write(Path::new("test.yaml"), serialized).expect("Couldn't write");
}
