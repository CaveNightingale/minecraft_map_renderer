mod common;

use std::{
    collections::BTreeMap,
    fs,
    io::Write,
    path::Path,
    process::{Command, Output},
};

use flate2::{Compression, write::ZlibEncoder};
use minecraft_map_renderer::{Index, registry::State};
use sha2::{Digest, Sha256};

fn write_chunks(directory: &Path, chunks: Vec<common::Chunk>) {
    fs::create_dir_all(directory).unwrap();
    let mut regions = BTreeMap::<(i32, i32), Vec<u8>>::new();
    for chunk in chunks {
        let data = common::to_nbt(&chunk);
        let mut compressed = ZlibEncoder::new(Vec::new(), Compression::default());
        compressed.write_all(&data).unwrap();
        let compressed = compressed.finish().unwrap();
        let region = regions
            .entry((chunk.x.div_euclid(32), chunk.z.div_euclid(32)))
            .or_insert_with(|| vec![0; 8192]);
        let start = region.len();
        let sectors = (compressed.len() + 5).div_ceil(4096);
        region.resize(start + sectors * 4096, 0);
        let index = (chunk.z.rem_euclid(32) * 32 + chunk.x.rem_euclid(32)) as usize * 4;
        region[index..index + 4]
            .copy_from_slice(&(((start / 4096) as u32) << 8 | sectors as u32).to_be_bytes());
        region[start..start + 4].copy_from_slice(&(compressed.len() as u32 + 1).to_be_bytes());
        region[start + 4] = 2;
        region[start + 5..start + 5 + compressed.len()].copy_from_slice(&compressed);
    }
    for ((x, z), bytes) in regions {
        fs::write(directory.join(format!("r.{x}.{z}.mca")), bytes).unwrap();
    }
}

fn run(input: &Path, output: &Path, registry: &Path, size: &str) -> Output {
    Command::new(env!("CARGO_BIN_EXE_minecraft-map-renderer"))
        .arg("--input")
        .arg(input)
        .arg("--output")
        .arg(output)
        .arg("--blockstate-registry")
        .arg(registry)
        .arg("--tile-size")
        .arg(size)
        .output()
        .unwrap()
}

fn pixels(directory: &Path) -> BTreeMap<(i64, i64), (u8, bool)> {
    let index: Index =
        serde_json::from_slice(&fs::read(directory.join("index.map")).unwrap()).unwrap();
    let size = index.tile_size;
    let mut result = BTreeMap::new();
    for tile in index.tiles {
        let bytes = fs::read(directory.join(format!("{}.mpt", tile.hash))).unwrap();
        assert_eq!(format!("{:x}", Sha256::digest(&bytes)), tile.hash);
        let (colors, coverage) = common::read_tile(&bytes, size);
        for z in 0..size {
            for x in 0..size {
                let bit = z / 16 * (size / 16) + x / 16;
                result.insert(
                    (
                        i64::from(tile.x) * size as i64 + x as i64,
                        i64::from(tile.z) * size as i64 + z as i64,
                    ),
                    (
                        colors[z * size + x],
                        coverage[bit / 8] & (1 << (bit % 8)) != 0,
                    ),
                );
            }
        }
    }
    result
}

#[test]
fn negative_coordinates_gaps_and_tile_boundaries_do_not_change_pixels() {
    let temporary = tempfile::tempdir().unwrap();
    let input = temporary.path().join("region");
    let registry = temporary.path().join("registry.json");
    fs::write(&registry, serde_json::to_vec(&common::registry()).unwrap()).unwrap();
    let mut partial = common::chunk(0, 1, 16);
    partial.status = "minecraft:noise".to_string();
    partial.sections[0].block_states.as_mut().unwrap().palette[1].name =
        "unknown:partial".to_string();
    write_chunks(
        &input,
        vec![
            common::chunk(-2, -2, 8),
            common::chunk(-2, -1, 9),
            common::chunk(-2, 1, 9),
            common::chunk(-1, -2, 8),
            common::chunk(0, -1, 8),
            common::chunk(0, 0, 8),
            partial,
            common::chunk(0, 2, 9),
            common::chunk(1, 0, 9),
            common::chunk(2, 3, 0),
        ],
    );
    let original = fs::read_dir(&input)
        .unwrap()
        .map(|entry| {
            let path = entry.unwrap().path();
            let data = fs::read(&path).unwrap();
            (path, data)
        })
        .collect::<Vec<_>>();
    let mut baseline = BTreeMap::new();
    for size in [16, 32, 64] {
        let output = temporary.path().join(format!("map-{size}"));
        let result = run(&input, &output, &registry, &size.to_string());
        assert!(
            result.status.success(),
            "{}",
            String::from_utf8_lossy(&result.stderr)
        );
        let actual = pixels(&output);
        assert_eq!(actual[&(-32, -32)], (46, true));
        assert_eq!(actual[&(-32, -16)], (46, true));
        assert_eq!(actual[&(-32, -15)], (45, true));
        assert_eq!(actual[&(-32, 16)], (45, true));
        assert_eq!(
            actual.get(&(0, 16)).copied().unwrap_or_default(),
            (0, false)
        );
        if baseline.is_empty() {
            baseline = actual;
        } else {
            for z in -32..64 {
                for x in -32..48 {
                    assert_eq!(
                        baseline.get(&(x, z)).copied().unwrap_or_default(),
                        actual.get(&(x, z)).copied().unwrap_or_default(),
                        "{x},{z}; tile size {size}"
                    );
                }
            }
        }
    }
    for (path, original) in original {
        assert_eq!(fs::read(path).unwrap(), original);
    }
}

#[test]
fn errors_report_the_chunk_and_never_overwrite_existing_output() {
    let temporary = tempfile::tempdir().unwrap();
    let input = temporary.path().join("region");
    let registry = temporary.path().join("registry.json");
    let output = temporary.path().join("map");
    fs::write(&registry, serde_json::to_vec(&common::registry()).unwrap()).unwrap();
    let mut invalid = common::chunk(0, 0, 8);
    invalid.sections[0].block_states.as_mut().unwrap().palette[1].name =
        "unknown:block".to_string();
    write_chunks(&input, vec![common::chunk(-1, 0, 8), invalid]);
    let result = run(&input, &output, &registry, "16");
    assert!(!result.status.success());
    let error = String::from_utf8_lossy(&result.stderr);
    assert!(
        error.contains("unknown:block") && error.contains("chunk 0,0"),
        "{error}"
    );
    // Tiles are written directly, but the index is published last, so a failed
    // run never looks like a complete map.
    assert!(!output.join("index.map").exists());
    assert!(fs::read_dir(temporary.path()).unwrap().all(|entry| {
        !entry
            .unwrap()
            .file_name()
            .to_string_lossy()
            .starts_with(".minecraft-map-")
    }));

    fs::write(output.join("keep.txt"), b"existing output").unwrap();
    assert!(!run(&input, &output, &registry, "16").status.success());
    assert_eq!(
        fs::read(output.join("keep.txt")).unwrap(),
        b"existing output"
    );
    assert!(!output.join("index.map").exists());

    let result = run(
        &input,
        &temporary.path().join("invalid-size"),
        &registry,
        "17",
    );
    assert_eq!(result.status.code(), Some(2));
    assert!(!temporary.path().join("invalid-size").exists());
}

#[test]
fn empty_world_produces_an_explicit_empty_index() {
    let temporary = tempfile::tempdir().unwrap();
    let input = temporary.path().join("region");
    let registry = temporary.path().join("registry.json");
    let output = temporary.path().join("map");
    fs::create_dir(&input).unwrap();
    fs::write(&registry, serde_json::to_vec(&common::registry()).unwrap()).unwrap();
    assert!(run(&input, &output, &registry, "16").status.success());
    let index: Index =
        serde_json::from_slice(&fs::read(output.join("index.map")).unwrap()).unwrap();
    assert!(index.tiles.is_empty());
    assert_eq!(fs::read_dir(output).unwrap().count(), 1);
}

#[test]
fn incompatible_registries_fail_before_creating_output() {
    let temporary = tempfile::tempdir().unwrap();
    let registry = temporary.path().join("registry.json");
    for invalid in 0..2 {
        let mut value = common::registry();
        match invalid {
            0 => {
                value.palette.pop();
            }
            1 => value.set_state(
                "minecraft:stone",
                State {
                    map_color: 64,
                    fluid: false,
                    surface_color: 11,
                },
            ),
            _ => unreachable!(),
        }
        fs::write(&registry, serde_json::to_vec(&value).unwrap()).unwrap();
        let output = temporary.path().join(format!("map-{invalid}"));
        let result = run(&temporary.path().join("region"), &output, &registry, "16");
        assert!(!result.status.success());
        assert!(!output.exists());
    }
}
