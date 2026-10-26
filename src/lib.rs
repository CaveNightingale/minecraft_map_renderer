pub mod chunk;
mod nbt;
pub mod region;
pub mod registry;

use std::{
    collections::{BTreeMap, HashMap},
    fs::{self, File},
    io::{self, BufWriter, IsTerminal, Write},
    path::{Path, PathBuf},
    sync::Arc,
};

use anyhow::{Context, Result};
use indicatif::{ProgressBar, ProgressStyle};
use libdeflater::{CompressionLvl, Compressor};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use chunk::RawChunk;
use registry::Registry;

pub const TILE_MAGIC: [u8; 8] = *b"MPT\x02\0\0\0\0";
/// Level 6 adds about 5% to a render; level 12 takes three times longer for
/// roughly 10% smaller tiles.
const TILE_COMPRESSION_LEVEL: i32 = 12;

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Index {
    pub format: u32,
    pub tile_size: usize,
    pub palette: Vec<i32>,
    pub tiles: Vec<Tile>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Tile {
    pub x: i32,
    pub z: i32,
    pub hash: String,
}

#[derive(Default, Debug)]
pub struct Stats {
    pub full_chunks: usize,
    pub partial_chunks: usize,
    pub tiles: usize,
}

struct TileProgress(ProgressBar);

impl TileProgress {
    fn new(total: usize) -> Self {
        if total == 0 || !io::stderr().is_terminal() {
            return Self(ProgressBar::hidden());
        }
        let bar = ProgressBar::new(total as u64);
        bar.set_style(
            ProgressStyle::with_template("[{bar:32}] {pos:>6}/{len} tiles ({percent:>3}%)")
                .expect("static progress template is valid"),
        );
        Self(bar)
    }

    fn advance(&self) {
        self.0.inc(1);
    }
}

impl Drop for TileProgress {
    fn drop(&mut self) {
        self.0.finish_and_clear();
    }
}

pub fn parse_tile_size(value: &str) -> std::result::Result<usize, String> {
    let size: usize = value
        .parse()
        .map_err(|_| "tile size must be an unsigned integer".to_string())?;
    let max = usize::MAX.isqrt() / 16 * 16;
    if size == 0 || !size.is_multiple_of(16) || size > max {
        return Err(format!(
            "tile size must be a positive multiple of 16, at most {max}"
        ));
    }
    Ok(size)
}

pub fn render(
    input: &Path,
    output: &Path,
    registry_path: &Path,
    tile_size: usize,
) -> Result<Stats> {
    parse_tile_size(&tile_size.to_string()).map_err(anyhow::Error::msg)?;
    let registry = Registry::read(registry_path)?;
    let chunks_per_tile = (tile_size / 16) as i32;
    let mut groups = BTreeMap::<(i32, i32), Vec<region::Location>>::new();
    for location in region::scan(input)? {
        let tile_x = location.x.div_euclid(chunks_per_tile);
        let tile_z = location.z.div_euclid(chunks_per_tile);
        groups.entry((tile_x, tile_z)).or_default().push(location);
    }
    let total_tiles = groups.len();
    // Process rows north to south so region files stay in the cache while their
    // whole row of tiles is rendered.
    let mut ordered = groups.into_iter().collect::<Vec<_>>();
    ordered.sort_unstable_by_key(|((x, z), _locations)| (*z, *x));

    fs::create_dir_all(output).with_context(|| format!("creating {}", output.display()))?;
    let mut index = Index {
        format: 1,
        tile_size,
        palette: registry.palette.clone(),
        tiles: Vec::new(),
    };
    let mut stats = Stats::default();
    let mut heights = BTreeMap::<i32, Vec<f64>>::new();
    let mut cache = region::RegionCache::default();
    let mut decoder = region::Decoder::new();
    let mut encoder = TileEncoder::new();
    let progress = TileProgress::new(total_tiles);
    for ((tile_x, tile_z), mut locations) in ordered {
        let previous_heights = heights
            .entry(tile_x)
            .or_insert_with(|| vec![0.0; tile_size]);
        let mut regions = HashMap::<PathBuf, Arc<region::RegionFile>>::new();
        for location in &locations {
            if !regions.contains_key(&location.region) {
                let file = cache.get(&location.region)?;
                regions.insert(location.region.clone(), file);
            }
        }
        locations.sort_unstable_by_key(|location| (location.z, location.x));
        let mut colors = vec![0; tile_size * tile_size];
        let mut coverage = vec![0; (tile_size / 16 * (tile_size / 16)).div_ceil(8)];
        let mut full_chunks = 0;
        for location in &locations {
            let context = || {
                format!(
                    "decoding chunk {},{} in {}",
                    location.x,
                    location.z,
                    location.region.display()
                )
            };
            let region = regions
                .get(&location.region)
                .expect("region file was loaded for this tile");
            let nbt = region
                .chunk_into(location, &mut decoder)
                .with_context(context)?;
            let chunk = RawChunk::parse(nbt, &registry).with_context(context)?;
            if !chunk
                .validate(location.x, location.z)
                .with_context(context)?
            {
                stats.partial_chunks += 1;
                continue;
            }
            let samples = chunk.sample(&registry).with_context(context)?;
            let local_x = location.x.rem_euclid(chunks_per_tile) as usize * 16;
            let local_z = location.z.rem_euclid(chunks_per_tile) as usize * 16;
            let bit = local_z / 16 * (tile_size / 16) + local_x / 16;
            coverage[bit / 8] |= 1 << (bit % 8);
            for x in 0..16 {
                for z in 0..16 {
                    let sample = samples[z * 16 + x];
                    let world_x = i64::from(location.x) * 16 + x as i64;
                    let world_z = i64::from(location.z) * 16 + z as i64;
                    colors[(local_z + z) * tile_size + local_x + x] =
                        chunk::shade(sample, previous_heights[local_x + x], world_x, world_z);
                    previous_heights[local_x + x] = f64::from(sample.height);
                }
            }
            full_chunks += 1;
        }
        if full_chunks != 0 {
            let hash = write_tile(output, tile_size, &colors, &coverage, &mut encoder)?;
            index.tiles.push(Tile {
                x: tile_x,
                z: tile_z,
                hash,
            });
            stats.full_chunks += full_chunks;
        }
        progress.advance();
    }
    index.tiles.sort_unstable_by_key(|tile| (tile.x, tile.z));
    stats.tiles = index.tiles.len();
    let mut manifest = BufWriter::new(File::create(output.join("index.map"))?);
    serde_json::to_writer(&mut manifest, &index)?;
    manifest.write_all(b"\n")?;
    manifest.flush()?;
    drop(manifest);
    Ok(stats)
}

/// Gzip-compresses tile payloads with one reusable libdeflate instance.
struct TileEncoder {
    compressor: Compressor,
    payload: Vec<u8>,
    compressed: Vec<u8>,
}

impl TileEncoder {
    fn new() -> Self {
        Self {
            compressor: Compressor::new(
                CompressionLvl::new(TILE_COMPRESSION_LEVEL).expect("tile level is valid"),
            ),
            payload: Vec::new(),
            compressed: Vec::new(),
        }
    }

    fn encode(&mut self, colors: &[u8], coverage: &[u8]) -> &[u8] {
        self.payload.clear();
        self.payload.extend_from_slice(colors);
        self.payload.extend_from_slice(coverage);
        let bound = self.compressor.gzip_compress_bound(self.payload.len());
        self.compressed.resize(bound, 0);
        let length = self
            .compressor
            .gzip_compress(&self.payload, &mut self.compressed)
            .expect("output buffer is sized by gzip_compress_bound");
        &self.compressed[..length]
    }
}

fn write_tile(
    output: &Path,
    size: usize,
    colors: &[u8],
    coverage: &[u8],
    encoder: &mut TileEncoder,
) -> Result<String> {
    let size_bytes = (size as u64).to_le_bytes();
    let compressed = encoder.encode(colors, coverage);
    let pieces: [&[u8]; 3] = [&TILE_MAGIC, &size_bytes, compressed];
    let mut digest = Sha256::new();
    for piece in pieces {
        digest.update(piece);
    }
    let hash = format!("{:x}", digest.finalize());
    let mut file = BufWriter::new(File::create(output.join(format!("{hash}.mpt")))?);
    for piece in pieces {
        file.write_all(piece)?;
    }
    file.flush()?;
    Ok(hash)
}
