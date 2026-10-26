use anyhow::{Context, Result, ensure};
use tinyvec::ArrayVec;

pub use crate::nbt::LongArray;

use crate::{
    nbt::{Reader, TAG_BYTE, TAG_COMPOUND, TAG_INT, TAG_LIST, TAG_LONG_ARRAY, TAG_STRING},
    registry::{Registry, State},
};

#[derive(Debug)]
pub struct RawChunk<'a> {
    pub x: i32,
    pub z: i32,
    pub min_section: i32,
    pub status: &'a [u8],
    pub sections: Vec<Section<'a>>,
    pub world_surface: Option<LongArray<'a>>,
}

#[derive(Debug)]
pub struct Section<'a> {
    pub y: i8,
    pub block_states: Option<BlockStates<'a>>,
}

#[derive(Debug)]
pub struct BlockStates<'a> {
    pub palette: Vec<State>,
    pub data: Option<LongArray<'a>>,
}

#[derive(Clone, Copy, Debug)]
pub struct Sample {
    pub height: i32,
    pub color: u8,
    pub depth: u32,
}

pub fn unpack(data: LongArray<'_>, bits: u32, count: usize) -> Result<Vec<usize>> {
    let per_long = packed_layout(data, bits, count)?;
    Ok((0..count)
        .map(|index| packed_value(data, per_long, bits, index))
        .collect())
}

fn packed_layout(data: LongArray<'_>, bits: u32, count: usize) -> Result<usize> {
    ensure!((1..=32).contains(&bits), "invalid palette bit width {bits}");
    let per_long = 64 / bits as usize;
    ensure!(
        data.len() == count.div_ceil(per_long),
        "incorrect packed array length: expected {}, found {}",
        count.div_ceil(per_long),
        data.len()
    );
    Ok(per_long)
}

fn packed_value(data: LongArray<'_>, per_long: usize, bits: u32, index: usize) -> usize {
    let mask = (1_u64 << bits) - 1;
    (((data.get(index / per_long) as u64) >> ((index % per_long) * bits as usize)) & mask) as usize
}

/// Borrowed view over one section. Values stay packed so a column lookup touches
/// only the blocks the sampler actually reads.
struct SectionView<'view, 'chunk> {
    palette: &'view [State],
    data: Option<LongArray<'chunk>>,
    per_long: usize,
    bits: u32,
}

impl SectionView<'_, '_> {
    fn get(&self, x: usize, y: i32, z: usize) -> Option<State> {
        if self.palette.len() == 1 {
            return self.palette.first().copied();
        }
        let data = self.data?;
        let index = y.rem_euclid(16) as usize * 256 + z * 16 + x;
        let value = packed_value(data, self.per_long, self.bits, index);
        self.palette.get(value).copied()
    }
}

fn parse_sections<'a>(reader: &mut Reader<'a>, registry: &Registry) -> Result<Vec<Section<'a>>> {
    let mut sections = Vec::new();
    reader.list(|reader, tag| {
        ensure!(tag == TAG_COMPOUND, "section entry is not a compound");
        let mut y = None;
        let mut block_states = None;
        reader.compound(|reader, name, tag| {
            match name {
                b"Y" => {
                    ensure!(tag == TAG_BYTE, "section Y is not a byte");
                    y = Some(reader.i8()?);
                }
                b"block_states" => {
                    ensure!(tag == TAG_COMPOUND, "block_states is not a compound");
                    block_states = Some(parse_block_states(reader, registry)?);
                }
                _ => reader.skip(tag)?,
            }
            Ok(())
        })?;
        sections.push(Section {
            y: y.context("section is missing Y")?,
            block_states,
        });
        Ok(())
    })?;
    Ok(sections)
}

fn parse_block_states<'a>(reader: &mut Reader<'a>, registry: &Registry) -> Result<BlockStates<'a>> {
    let mut palette = None;
    let mut data = None;
    reader.compound(|reader, name, tag| {
        match name {
            b"palette" => {
                ensure!(tag == TAG_LIST, "block state palette is not a list");
                palette = Some(parse_palette(reader, registry)?);
            }
            b"data" => {
                ensure!(
                    tag == TAG_LONG_ARRAY,
                    "block state data is not a long array"
                );
                data = Some(reader.long_array()?);
            }
            _ => reader.skip(tag)?,
        }
        Ok(())
    })?;
    Ok(BlockStates {
        palette: palette.context("block_states is missing palette")?,
        data,
    })
}

fn parse_palette<'a>(reader: &mut Reader<'a>, registry: &Registry) -> Result<Vec<State>> {
    let mut palette = Vec::new();
    reader.list(|reader, tag| {
        let mut key = ArrayVec::<[u8; 1024]>::new();
        let mut properties = ArrayVec::<[(&[u8], &[u8]); 24]>::new();
        ensure!(tag == TAG_COMPOUND, "palette entry is not a compound");
        let mut name = None;
        reader.compound(|reader, field, tag| {
            match field {
                b"Name" => {
                    ensure!(tag == TAG_STRING, "palette entry Name is not a string");
                    name = Some(reader.string_bytes()?);
                }
                b"Properties" => {
                    ensure!(tag == TAG_COMPOUND, "palette Properties is not a compound");
                    reader.compound(|reader, property, tag| {
                        ensure!(tag == TAG_STRING, "block state property is not a string");
                        properties.push((property, reader.string_bytes()?));
                        Ok(())
                    })?;
                }
                _ => reader.skip(tag)?,
            }
            Ok(())
        })?;
        let name = name.context("palette entry is missing Name")?;
        key.extend_from_slice(name);
        if !properties.is_empty() {
            properties.sort_unstable_by(|left, right| left.0.cmp(right.0));
            key.push(b'[');
            for (index, (property, value)) in properties.iter().enumerate() {
                if index != 0 {
                    key.push(b',');
                }
                key.extend_from_slice(property);
                key.push(b'=');
                key.extend_from_slice(value);
            }
            key.push(b']');
        }
        let state = registry
            .state(&key)
            .with_context(|| format!("unknown block state {}", key.escape_ascii()))?;
        palette.push(state);
        Ok(())
    })?;
    Ok(palette)
}

fn parse_world_surface<'a>(reader: &mut Reader<'a>) -> Result<Option<LongArray<'a>>> {
    let mut surface = None;
    reader.compound(|reader, name, tag| {
        if name == b"WORLD_SURFACE" {
            ensure!(tag == TAG_LONG_ARRAY, "WORLD_SURFACE is not a long array");
            surface = Some(reader.long_array()?);
        } else {
            reader.skip(tag)?;
        }
        Ok(())
    })?;
    Ok(surface)
}

struct Metadata<'a> {
    x: i32,
    z: i32,
    min_section: i32,
    status: &'a [u8],
}

fn parse_metadata(data: &[u8]) -> Result<Metadata<'_>> {
    let mut reader = Reader::new(data);
    ensure!(reader.u8()? == TAG_COMPOUND, "chunk root is not a compound");
    reader.string_bytes()?;

    let mut x = None;
    let mut z = None;
    let mut min_section = None;
    let mut status = None;

    reader.compound(|reader, name, tag| {
        match name {
            b"xPos" => {
                ensure!(tag == TAG_INT, "xPos is not an int");
                x = Some(reader.i32()?);
            }
            b"zPos" => {
                ensure!(tag == TAG_INT, "zPos is not an int");
                z = Some(reader.i32()?);
            }
            b"yPos" => {
                ensure!(tag == TAG_INT, "yPos is not an int");
                min_section = Some(reader.i32()?);
            }
            b"Status" => {
                ensure!(tag == TAG_STRING, "Status is not a string");
                status = Some(reader.string_bytes()?);
            }
            _ => reader.skip(tag)?,
        }
        Ok(())
    })?;

    Ok(Metadata {
        x: x.context("chunk is missing xPos")?,
        z: z.context("chunk is missing zPos")?,
        min_section: min_section.context("chunk is missing yPos")?,
        status: status.context("chunk is missing Status")?,
    })
}

fn parse_full<'a>(
    data: &'a [u8],
    registry: &Registry,
) -> Result<(Vec<Section<'a>>, Option<LongArray<'a>>)> {
    let mut reader = Reader::new(data);
    ensure!(reader.u8()? == TAG_COMPOUND, "chunk root is not a compound");
    reader.string_bytes()?;

    let mut sections = None;
    let mut world_surface = None;

    reader.compound(|reader, name, tag| {
        match name {
            b"sections" => {
                ensure!(tag == TAG_LIST, "sections is not a list");
                sections = Some(parse_sections(reader, registry)?);
            }
            b"Heightmaps" => {
                ensure!(tag == TAG_COMPOUND, "Heightmaps is not a compound");
                world_surface = parse_world_surface(reader)?;
            }
            _ => reader.skip(tag)?,
        }
        Ok(())
    })?;

    Ok((
        sections.context("chunk is missing sections")?,
        world_surface,
    ))
}

impl<'a> RawChunk<'a> {
    pub fn parse(data: &'a [u8], registry: &Registry) -> Result<Self> {
        let metadata = parse_metadata(data)?;
        let (sections, world_surface) = if metadata.status == b"minecraft:full" {
            parse_full(data, registry)?
        } else {
            (Vec::new(), None)
        };
        Ok(Self {
            x: metadata.x,
            z: metadata.z,
            min_section: metadata.min_section,
            status: metadata.status,
            sections,
            world_surface,
        })
    }

    pub fn validate(&self, x: i32, z: i32) -> Result<bool> {
        ensure!(
            self.x == x && self.z == z,
            "NBT coordinates {},{} do not match {x},{z}",
            self.x,
            self.z
        );
        Ok(self.status == b"minecraft:full")
    }

    pub fn sample(&self, registry: &Registry) -> Result<[Sample; 256]> {
        ensure!(
            (-128..=127).contains(&self.min_section),
            "invalid minimum section {}",
            self.min_section
        );
        let max_section = self
            .sections
            .iter()
            .map(|section| i32::from(section.y))
            .filter(|y| *y >= self.min_section)
            .max()
            .context("full chunk has no sections")?;
        let min_y = self.min_section * 16;
        let height = (max_section - self.min_section + 1) * 16;
        let air = registry
            .state(b"minecraft:air")
            .context("registry is missing minecraft:air")?;
        let section_count = (max_section - self.min_section + 1) as usize;
        let mut sections: Vec<Option<SectionView>> = (0..section_count).map(|_| None).collect();
        let mut seen = vec![false; section_count];
        for section in &self.sections {
            let section_index = i32::from(section.y) - self.min_section;
            if section_index < 0 {
                continue;
            }
            let section_index = section_index as usize;
            ensure!(
                section_index < section_count,
                "section above maximum height"
            );
            ensure!(!seen[section_index], "duplicate section Y={}", section.y);
            seen[section_index] = true;
            if let Some(blocks) = &section.block_states {
                let (data, per_long, bits) = if blocks.palette.len() == 1 {
                    ensure!(
                        blocks.data.is_none_or(|data| data.is_empty()),
                        "single-state palette has packed data"
                    );
                    (None, 0, 0)
                } else {
                    let bits = (usize::BITS - (blocks.palette.len() - 1).leading_zeros()).max(4);
                    let data = blocks
                        .data
                        .context("multi-state palette has no packed data")?;
                    (Some(data), packed_layout(data, bits, 4096)?, bits)
                };
                sections[section_index] = Some(SectionView {
                    palette: &blocks.palette,
                    data,
                    per_long,
                    bits,
                });
            }
        }
        let block = |x: usize, y: i32, z: usize| -> Result<State> {
            let section = (y.div_euclid(16) - self.min_section) as usize;
            match sections.get(section).and_then(Option::as_ref) {
                Some(view) => view
                    .get(x, y, z)
                    .with_context(|| "palette index is out of range"),
                None => Ok(air),
            }
        };
        let height_bits = (u32::BITS - (height as u32).leading_zeros()).max(1);
        let surface = self
            .world_surface
            .context("full chunk is missing WORLD_SURFACE heightmap")?;
        let per_long = packed_layout(surface, height_bits, 256)?;
        let bedrock = registry
            .state(b"minecraft:bedrock")
            .context("registry is missing minecraft:bedrock")?
            .map_color;
        let mut samples = [Sample {
            height: min_y,
            color: bedrock,
            depth: 0,
        }; 256];
        for z in 0..16 {
            for x in 0..16 {
                let index = z * 16 + x;
                let top = packed_value(surface, per_long, height_bits, index);
                ensure!(
                    top <= height as usize,
                    "heightmap value exceeds world height"
                );
                let mut y = min_y + top as i32;
                let mut color = bedrock;
                let mut depth = 0;
                if y > min_y {
                    let state = loop {
                        y -= 1;
                        let state = block(x, y, z)?;
                        if state.map_color != 0 || y <= min_y {
                            break state;
                        }
                    };
                    color = state.map_color;
                    if y > min_y && state.fluid {
                        let mut below = y - 1;
                        loop {
                            let fluid = block(x, below, z)?.fluid;
                            below -= 1;
                            depth += 1;
                            if below <= min_y || !fluid {
                                break;
                            }
                        }
                        color = state.surface_color;
                    }
                }
                samples[index] = Sample {
                    height: y,
                    color,
                    depth,
                };
            }
        }
        Ok(samples)
    }
}

pub fn shade(sample: Sample, previous_height: f64, x: i64, z: i64) -> u8 {
    let parity = ((x + z) & 1) as f64;
    let brightness = if sample.color == 12 {
        let value = f64::from(sample.depth) * 0.1 + parity * 0.2;
        if value < 0.5 {
            2
        } else if value > 0.9 {
            0
        } else {
            1
        }
    } else {
        let value = (f64::from(sample.height) - previous_height) * 4.0 / 5.0 + (parity - 0.5) * 0.4;
        if value > 0.6 {
            2
        } else if value < -0.6 {
            0
        } else {
            1
        }
    };
    sample.color * 4 + brightness
}
