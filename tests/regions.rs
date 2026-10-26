use std::{fs, io::Write};

use flate2::{
    Compression,
    write::{GzEncoder, ZlibEncoder},
};
use lz4_java_wrc::Lz4BlockOutput;
use minecraft_map_renderer::region;

fn encode(compression: u8, value: &[u8]) -> Vec<u8> {
    match compression {
        1 => {
            let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
            encoder.write_all(value).unwrap();
            encoder.finish().unwrap()
        }
        2 => {
            let mut encoder = ZlibEncoder::new(Vec::new(), Compression::default());
            encoder.write_all(value).unwrap();
            encoder.finish().unwrap()
        }
        3 => value.to_vec(),
        4 => {
            let mut output = Vec::new();
            let mut encoder = Lz4BlockOutput::new(&mut output);
            encoder.write_all(value).unwrap();
            encoder.flush().unwrap();
            drop(encoder);
            output
        }
        _ => unreachable!(),
    }
}

#[test]
fn all_vanilla_compressions_and_external_chunks_round_trip() {
    let payload = b"test chunk data ".repeat(1000);
    for compression in 1..=4 {
        for external in [false, true] {
            let directory = tempfile::tempdir().unwrap();
            let compressed = encode(compression, &payload);
            let mut file = vec![0_u8; 8192 + (compressed.len() + 5).div_ceil(4096) * 4096];
            let sectors = if external {
                1
            } else {
                (compressed.len() + 5).div_ceil(4096) as u32
            };
            file[..4].copy_from_slice(&((2 << 8) | sectors).to_be_bytes());
            let length = if external {
                1
            } else {
                compressed.len() as u32 + 1
            };
            file[8192..8196].copy_from_slice(&length.to_be_bytes());
            file[8196] = compression | if external { 0x80 } else { 0 };
            if external {
                fs::write(directory.path().join("c.-32.-64.mcc"), &compressed).unwrap();
            } else {
                file[8197..8197 + compressed.len()].copy_from_slice(&compressed);
            }
            fs::write(directory.path().join("r.-1.-2.mca"), file).unwrap();
            let chunks = region::scan(directory.path()).unwrap();
            assert_eq!((chunks[0].x, chunks[0].z), (-32, -64));
            let file = region::RegionFile::read(&chunks[0].region).unwrap();
            assert_eq!(
                chunks[0].read(&file).unwrap(),
                payload,
                "compression={compression}, external={external}"
            );
        }
    }
}

#[test]
fn empty_regions_are_absent_but_truncated_headers_and_overlaps_fail() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("r.0.0.mca");
    fs::write(&path, []).unwrap();
    assert!(region::scan(directory.path()).unwrap().is_empty());
    fs::write(&path, vec![0; 8192]).unwrap();
    assert!(region::scan(directory.path()).unwrap().is_empty());
    fs::write(&path, vec![0; 4096]).unwrap();
    assert!(region::scan(directory.path()).is_err());
    let mut file = vec![0; 12288];
    file[..4].copy_from_slice(&513_u32.to_be_bytes());
    file[4..8].copy_from_slice(&513_u32.to_be_bytes());
    fs::write(&path, &file).unwrap();
    assert!(region::scan(directory.path()).is_err());
}

#[test]
fn invalid_lengths_missing_external_payloads_and_unknown_compression_fail() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("r.0.0.mca");
    for (length, compression) in [(0_u32, 3), (4093, 3), (1, 127), (1, 0x82)] {
        let mut file = vec![0; 12288];
        file[..4].copy_from_slice(&513_u32.to_be_bytes());
        file[8192..8196].copy_from_slice(&length.to_be_bytes());
        file[8196] = compression;
        fs::write(&path, file).unwrap();
        let chunks = region::scan(directory.path()).unwrap();
        let file = region::RegionFile::read(&chunks[0].region).unwrap();
        assert!(chunks[0].read(&file).is_err());
    }
}
