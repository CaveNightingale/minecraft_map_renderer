use std::{collections::BTreeMap, fs::File, io::BufReader, path::Path};

use anyhow::{Context, Result, ensure};
use serde::{Deserialize, Serialize};

use rapidhash::{HashMapExt, RapidHashMap};

pub const DATA_VERSION: i32 = 4786;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct State {
    pub map_color: u8,
    pub fluid: bool,
    pub surface_color: u8,
}

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Registry {
    pub palette: Vec<i32>,
    states: BTreeMap<String, State>,
    #[serde(skip)]
    lookup: RapidHashMap<Vec<u8>, State>,
}

impl Registry {
    pub fn new(palette: Vec<i32>, states: BTreeMap<String, State>) -> Self {
        let mut registry = Self {
            palette,
            states,
            lookup: RapidHashMap::with_capacity(1_000_000),
        };
        registry.refresh_lookup();
        registry
    }

    pub fn state(&self, key: &[u8]) -> Option<State> {
        self.lookup.get(key).copied()
    }

    pub fn set_state(&mut self, key: &str, state: State) {
        self.states.insert(key.to_owned(), state);
        self.lookup.insert(key.as_bytes().to_vec(), state);
    }

    fn refresh_lookup(&mut self) {
        self.lookup = self
            .states
            .iter()
            .map(|(key, state)| (key.as_bytes().to_vec(), *state))
            .collect();
    }

    pub fn read(path: &Path) -> Result<Self> {
        let mut registry: Self = serde_json::from_reader(BufReader::new(
            File::open(path).with_context(|| format!("opening registry {}", path.display()))?,
        ))
        .with_context(|| format!("reading registry {}", path.display()))?;
        registry.refresh_lookup();
        ensure!(
            registry.palette.len() == 256,
            "registry palette must contain 256 ARGB entries"
        );
        ensure!(
            registry.palette[..4].iter().all(|color| *color == 0),
            "map color IDs 0..3 must be transparent"
        );
        for (key, state) in &registry.states {
            ensure!(
                state.map_color < 64 && state.surface_color < 64,
                "invalid map color in state {key}"
            );
        }
        for key in [
            "minecraft:air",
            "minecraft:bedrock",
            "minecraft:water[level=0]",
        ] {
            ensure!(
                registry.states.contains_key(key),
                "registry is missing {key}"
            );
        }
        ensure!(
            registry.states["minecraft:water[level=0]"].map_color == 12,
            "unsupported vanilla water map color"
        );
        Ok(registry)
    }
}
