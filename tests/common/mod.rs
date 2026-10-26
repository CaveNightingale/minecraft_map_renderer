#![allow(dead_code)]

use std::collections::BTreeMap;

use minecraft_map_renderer::{
    chunk::RawChunk,
    registry::{Registry, State},
};

pub struct Chunk {
    pub x: i32,
    pub z: i32,
    pub min_section: i32,
    pub status: String,
    pub sections: Vec<ChunkSection>,
    pub heightmaps: BTreeMap<String, Vec<u8>>,
}

pub struct ChunkSection {
    pub y: i8,
    pub block_states: Option<ChunkBlockStates>,
}

pub struct ChunkBlockStates {
    pub palette: Vec<PaletteEntry>,
    pub data: Option<Vec<u8>>,
}

pub struct PaletteEntry {
    pub name: String,
    pub properties: Vec<(String, String)>,
}

pub fn palette(name: &str) -> PaletteEntry {
    PaletteEntry {
        name: name.to_string(),
        properties: Vec::new(),
    }
}

pub fn palette_with(name: &str, properties: &[(&str, &str)]) -> PaletteEntry {
    PaletteEntry {
        name: name.to_string(),
        properties: properties
            .iter()
            .map(|(key, value)| (key.to_string(), value.to_string()))
            .collect(),
    }
}

pub fn registry() -> Registry {
    Registry::new(
        vec![0; 256],
        [
            (
                "minecraft:air",
                State {
                    map_color: 0,
                    fluid: false,
                    surface_color: 0,
                },
            ),
            (
                "minecraft:bedrock",
                State {
                    map_color: 11,
                    fluid: false,
                    surface_color: 11,
                },
            ),
            (
                "minecraft:stone",
                State {
                    map_color: 11,
                    fluid: false,
                    surface_color: 11,
                },
            ),
            (
                "minecraft:water[level=0]",
                State {
                    map_color: 12,
                    fluid: true,
                    surface_color: 12,
                },
            ),
            (
                "minecraft:test_waterlogged",
                State {
                    map_color: 0,
                    fluid: true,
                    surface_color: 12,
                },
            ),
            (
                "minecraft:test[a=x,z=b]",
                State {
                    map_color: 13,
                    fluid: false,
                    surface_color: 13,
                },
            ),
        ]
        .into_iter()
        .map(|(key, value)| (key.to_string(), value))
        .collect(),
    )
}

pub fn pack(values: &[usize], bits: usize) -> Vec<u8> {
    let per_long = 64 / bits;
    let mut longs = vec![0_u64; values.len().div_ceil(per_long)];
    for (index, value) in values.iter().enumerate() {
        longs[index / per_long] |= (*value as u64) << ((index % per_long) * bits);
    }
    longs
        .into_iter()
        .flat_map(u64::to_be_bytes)
        .collect::<Vec<_>>()
}

pub fn chunk(x: i32, z: i32, height: usize) -> Chunk {
    let mut values = vec![0; 4096];
    for y in 0..height {
        values[y * 256..(y + 1) * 256].fill(1);
    }
    Chunk {
        x,
        z,
        min_section: 0,
        status: "minecraft:full".to_string(),
        sections: vec![ChunkSection {
            y: 0,
            block_states: Some(ChunkBlockStates {
                palette: vec![palette("minecraft:air"), palette("minecraft:stone")],
                data: Some(pack(&values, 4)),
            }),
        }],
        heightmaps: BTreeMap::from([("WORLD_SURFACE".to_string(), pack(&vec![height; 256], 5))]),
    }
}

pub fn parse<'a>(data: &'a [u8], registry: &Registry) -> RawChunk<'a> {
    RawChunk::parse(data, registry).unwrap()
}

/// Verifies a `.mpt` file and returns its colors and coverage.
pub fn read_tile(bytes: &[u8], size: usize) -> (Vec<u8>, Vec<u8>) {
    use std::io::Read;
    assert_eq!(&bytes[..8], &minecraft_map_renderer::TILE_MAGIC);
    assert_eq!(
        u64::from_le_bytes(bytes[8..16].try_into().unwrap()),
        size as u64
    );
    let mut payload = Vec::new();
    flate2::read::GzDecoder::new(&bytes[16..])
        .read_to_end(&mut payload)
        .unwrap();
    let pixels = size * size;
    let coverage_length = (size / 16 * (size / 16)).div_ceil(8);
    assert_eq!(payload.len(), pixels + coverage_length);
    let coverage = payload.split_off(pixels);
    (payload, coverage)
}

fn write_string(out: &mut Vec<u8>, value: &str) {
    let length = u16::try_from(value.len()).expect("test string is too long");
    out.extend_from_slice(&length.to_be_bytes());
    out.extend_from_slice(value.as_bytes());
}

fn write_tag(out: &mut Vec<u8>, tag: u8, name: &str) {
    out.push(tag);
    write_string(out, name);
}

fn write_byte(out: &mut Vec<u8>, name: &str, value: i8) {
    write_tag(out, 1, name);
    out.push(value as u8);
}

fn write_int(out: &mut Vec<u8>, name: &str, value: i32) {
    write_tag(out, 3, name);
    out.extend_from_slice(&value.to_be_bytes());
}

fn write_string_tag(out: &mut Vec<u8>, name: &str, value: &str) {
    write_tag(out, 8, name);
    write_string(out, value);
}

fn write_long_array(out: &mut Vec<u8>, name: &str, data: &[u8]) {
    write_tag(out, 12, name);
    out.extend_from_slice(&((data.len() / 8) as i32).to_be_bytes());
    out.extend_from_slice(data);
}

fn write_compound(out: &mut Vec<u8>, name: &str, fields: impl FnOnce(&mut Vec<u8>)) {
    write_tag(out, 10, name);
    fields(out);
    out.push(0);
}

fn write_list_header(out: &mut Vec<u8>, name: &str, element: u8, length: usize) {
    write_tag(out, 9, name);
    out.push(element);
    out.extend_from_slice(&(length as i32).to_be_bytes());
}

/// Serializes the chunk subset the renderer consumes, including unrelated
/// fields at each level so the parser's skip path is exercised.
pub fn to_nbt(chunk: &Chunk) -> Vec<u8> {
    let mut out = vec![10];
    write_string(&mut out, "");
    write_int(&mut out, "xPos", chunk.x);
    write_int(&mut out, "zPos", chunk.z);
    write_int(&mut out, "yPos", chunk.min_section);
    write_string_tag(&mut out, "Status", &chunk.status);
    write_int(&mut out, "IgnoredInt", 42);

    write_list_header(&mut out, "sections", 10, chunk.sections.len());
    for section in &chunk.sections {
        write_byte(&mut out, "Y", section.y);
        if let Some(blocks) = &section.block_states {
            write_compound(&mut out, "block_states", |out| {
                write_list_header(out, "palette", 10, blocks.palette.len());
                for entry in &blocks.palette {
                    write_string_tag(out, "Name", &entry.name);
                    write_compound(out, "Properties", |out| {
                        for (key, value) in &entry.properties {
                            write_string_tag(out, key, value);
                        }
                    });
                    out.push(0);
                }
                if let Some(data) = &blocks.data {
                    write_long_array(out, "data", data);
                }
            });
        }
        write_compound(&mut out, "IgnoredSectionCompound", |_out| {});
        out.push(0);
    }

    write_compound(&mut out, "Heightmaps", |out| {
        for (name, data) in &chunk.heightmaps {
            write_long_array(out, name, data);
        }
    });
    write_string_tag(&mut out, "IgnoredString", "value");
    out.push(0);
    out
}
