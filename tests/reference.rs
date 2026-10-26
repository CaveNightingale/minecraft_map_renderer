mod common;

use std::{fs, path::Path, process::Command};

use minecraft_map_renderer::Index;
use sha2::{Digest, Sha256};

#[test]
#[ignore = "set MINECRAFT_TEST_WORLD to the one-shot exporter's world directory"]
fn real_cli_matches_every_native_reference_pixel_and_coverage_bit() {
    let world = std::env::var_os("MINECRAFT_TEST_WORLD").expect("MINECRAFT_TEST_WORLD is required");
    let world = Path::new(&world);
    let expected = fs::read(world.join("reference.colors")).unwrap();
    let states = fs::read(world.join("reference.chunks")).unwrap();
    assert_eq!(expected.len(), 2048 * 2048);
    assert_eq!(states.len(), 128 * 128);
    for size in [16, 512, 2048, 4096] {
        let temporary = tempfile::tempdir().unwrap();
        let output = temporary.path().join("map");
        let result = Command::new(env!("CARGO_BIN_EXE_minecraft-map-renderer"))
            .arg("--input")
            .arg(world.join("dimensions/minecraft/overworld/region"))
            .arg("--output")
            .arg(&output)
            .arg("--blockstate-registry")
            .arg(world.join("blockstate_registry.json"))
            .arg("--tile-size")
            .arg(size.to_string())
            .output()
            .unwrap();
        assert!(
            result.status.success(),
            "{}",
            String::from_utf8_lossy(&result.stderr)
        );
        let index: Index =
            serde_json::from_slice(&fs::read(output.join("index.map")).unwrap()).unwrap();
        assert_eq!(index.format, 1);
        assert_eq!(index.tile_size, size);
        let mut actual = vec![0_u8; expected.len()];
        let mut coverage = vec![0_u8; states.len()];
        for tile in index.tiles {
            let bytes = fs::read(output.join(format!("{}.mpt", tile.hash))).unwrap();
            assert_eq!(format!("{:x}", Sha256::digest(&bytes)), tile.hash);
            let (colors, coverage_bits) = common::read_tile(&bytes, size);
            let origin_x = i64::from(tile.x) * size as i64;
            let origin_z = i64::from(tile.z) * size as i64;
            for z in origin_z.max(0)..(origin_z + size as i64).min(2048) {
                for x in origin_x.max(0)..(origin_x + size as i64).min(2048) {
                    actual[z as usize * 2048 + x as usize] =
                        colors[(z - origin_z) as usize * size + (x - origin_x) as usize];
                }
            }
            for cz in 0..size / 16 {
                for cx in 0..size / 16 {
                    let x = origin_x.div_euclid(16) + cx as i64;
                    let z = origin_z.div_euclid(16) + cz as i64;
                    if (0..128).contains(&x) && (0..128).contains(&z) {
                        let bit = cz * (size / 16) + cx;
                        coverage[z as usize * 128 + x as usize] =
                            (coverage_bits[bit / 8] >> (bit % 8)) & 1;
                    }
                }
            }
        }
        let differences = actual
            .iter()
            .zip(&expected)
            .enumerate()
            .filter(|(_, (actual, expected))| actual != expected)
            .collect::<Vec<_>>();
        assert!(
            differences.is_empty(),
            "tile size {size}: {} color differences; first {:?}",
            differences.len(),
            differences
                .first()
                .map(|(index, values)| (index % 2048, index / 2048, values))
        );
        for (index, state) in states.iter().enumerate() {
            assert_eq!(
                coverage[index],
                u8::from(*state == 2),
                "tile size {size}, chunk {},{}",
                index % 128,
                index / 128
            );
        }
    }
}
