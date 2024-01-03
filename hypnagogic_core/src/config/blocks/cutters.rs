use std::collections::{BTreeMap, HashMap};

use fixed_map::Map;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::util::corners::{CornerType, Side};

#[derive(Copy, Clone, Eq, PartialEq, Debug, Serialize, Deserialize)]
pub struct IconSize {
    pub x: u32,
    pub y: u32,
}

impl Default for IconSize {
    fn default() -> Self {
        Self { x: 32, y: 32 }
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Debug, Serialize, Deserialize, Default)]
pub struct OutputIconPosition {
    pub x: u32,
    pub y: u32,
}

#[derive(Copy, Clone, Eq, PartialEq, Debug, Serialize, Deserialize)]
pub struct OutputIconSize {
    pub x: u32,
    pub y: u32,
}

impl Default for OutputIconSize {
    fn default() -> Self {
        Self { x: 32, y: 32 }
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Debug, Serialize, Deserialize)]
pub struct CutPosition {
    pub x: u32,
    pub y: u32,
}

impl Default for CutPosition {
    fn default() -> Self {
        Self { x: 16, y: 16 }
    }
}

#[derive(Clone, Eq, PartialEq, Debug)]
pub struct Positions(pub Map<CornerType, u32>);

impl Positions {
    #[must_use]
    pub fn get(&self, key: CornerType) -> Option<u32> {
        self.0.get(key).copied()
    }
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
#[serde(transparent)]
struct PositionsHelper {
    map: BTreeMap<String, u32>,
}

impl Serialize for Positions {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut map = BTreeMap::new();

        for (k, v) in self.0.iter() {
            map.insert(k.to_string(), *v);
        }

        PositionsHelper { map }.serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for Positions {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        Deserialize::deserialize(deserializer).map(|PositionsHelper { map }| {
            let mut result = Map::new();
            for (k, v) in map {
                result.insert(k.as_str().into(), v);
            }
            Positions(result)
        })
    }
}

impl Default for Positions {
    fn default() -> Self {
        let mut map = Map::new();
        map.insert(CornerType::Convex, 0);
        map.insert(CornerType::Concave, 1);
        map.insert(CornerType::Horizontal, 2);
        map.insert(CornerType::Vertical, 3);
        Positions(map)
    }
}

#[derive(Clone, Eq, PartialEq, Debug, Default)]
pub struct StringMap(pub HashMap<String, String>);

impl StringMap {
    #[must_use]
    pub fn get(&self, key: &str) -> Option<&String> {
        self.0.get(key)
    }
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
#[serde(transparent)]
struct StringMapHelper {
    map: HashMap<String, String>,
}

impl Serialize for StringMap {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut map = HashMap::new();

        for (k, v) in &self.0 {
            map.insert(k.clone(), v.clone());
        }

        StringMapHelper { map }.serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for StringMap {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        Deserialize::deserialize(deserializer).map(|StringMapHelper { map }| {
            let mut result = HashMap::new();
            for (k, v) in map {
                result.insert(k, v);
            }
            StringMap(result)
        })
    }
}

#[derive(Clone, Eq, PartialEq, Debug, Default)]
pub struct Prefabs(pub BTreeMap<u8, u32>);

#[derive(Clone, Eq, PartialEq, Debug, Serialize, Deserialize)]
#[serde(transparent)]
struct PrefabsHelper {
    map: BTreeMap<String, u32>,
}

impl Serialize for Prefabs {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut map = BTreeMap::new();

        for (k, v) in &self.0 {
            map.insert(k.to_string(), *v);
        }

        PrefabsHelper { map }.serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for Prefabs {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        Deserialize::deserialize(deserializer).map(|PrefabsHelper { map }| {
            let mut result = BTreeMap::new();
            for (k, v) in map {
                result.insert(k.parse().unwrap(), v);
            }
            Prefabs(result)
        })
    }
}

#[derive(Clone, Eq, PartialEq, Debug, Default)]
pub struct PrefabOverlays(pub BTreeMap<u8, Vec<u32>>);

#[derive(Clone, Eq, PartialEq, Debug, Serialize, Deserialize)]
#[serde(transparent)]
struct PrefabOverlaysHelper {
    map: BTreeMap<String, Vec<u32>>,
}

impl Serialize for PrefabOverlays {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut map = BTreeMap::new();

        for (k, v) in &self.0 {
            map.insert(k.to_string(), v.clone());
        }

        PrefabOverlaysHelper { map }.serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for PrefabOverlays {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        Deserialize::deserialize(deserializer).map(|PrefabOverlaysHelper { map }| {
            let mut result = BTreeMap::new();
            for (k, v) in map {
                result.insert(k.parse().unwrap(), v);
            }
            PrefabOverlays(result)
        })
    }
}

#[derive(Clone, PartialEq, Debug, Default, Serialize, Deserialize)]
pub struct Animation {
    pub delays: Vec<f32>,
}

#[derive(Clone, Eq, PartialEq, Debug)]
pub struct SlicePoint(pub Map<Side, u32>);

impl SlicePoint {
    #[must_use]
    pub fn get(&self, key: Side) -> Option<u32> {
        self.0.get(key).copied()
    }
}

#[derive(Clone, Eq, PartialEq, Debug, Serialize, Deserialize)]
#[serde(transparent)]
struct SlicePointHelper {
    map: BTreeMap<String, u32>,
}

impl Serialize for SlicePoint {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut map = BTreeMap::new();

        for (k, v) in self.0.iter() {
            map.insert(k.to_string(), *v);
        }

        SlicePointHelper { map }.serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for SlicePoint {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        Deserialize::deserialize(deserializer).map(|SlicePointHelper { map }| {
            let mut result = Map::new();
            for (k, v) in map {
                result.insert(k.as_str().into(), v);
            }
            SlicePoint(result)
        })
    }
}

impl Default for SlicePoint {
    fn default() -> Self {
        let mut map = Map::new();
        map.insert(Side::West, 4);
        map.insert(Side::North, 16);
        map.insert(Side::South, 16);
        map.insert(Side::East, 28);
        SlicePoint(map)
    }
}
