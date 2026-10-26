mod common;

use minecraft_map_renderer::{
    chunk::{LongArray, RawChunk, Sample, shade, unpack},
    parse_tile_size,
    registry::State,
};

#[test]
fn tile_size_maximum_is_exact_and_does_not_allocate() {
    let max = usize::MAX.isqrt() / 16 * 16;
    assert_eq!(parse_tile_size(&max.to_string()).unwrap(), max);
    assert!(max.checked_mul(max).is_some());
    for value in ["0", "1", "17", "-16", "text", &(max + 16).to_string()] {
        assert!(parse_tile_size(value).is_err(), "{value}");
    }
    assert_eq!(parse_tile_size("2048").unwrap(), 2048);
}

#[test]
fn palette_properties_are_sorted_before_registry_lookup() {
    let registry = common::registry();
    let mut chunk = common::chunk(0, 0, 3);
    let blocks = chunk.sections[0].block_states.as_mut().unwrap();
    blocks.palette = vec![common::palette_with(
        "minecraft:test",
        &[("z", "b"), ("a", "x")],
    )];
    blocks.data = None;
    let nbt = common::to_nbt(&chunk);
    let parsed = common::parse(&nbt, &registry);
    assert_eq!(
        parsed.sections[0].block_states.as_ref().unwrap().palette,
        vec![registry.state(b"minecraft:test[a=x,z=b]").unwrap()]
    );
}

#[test]
fn padded_five_and_nine_bit_arrays_do_not_cross_long_boundaries() {
    for (bits, count) in [(5, 4096), (9, 256)] {
        let values = (0..count)
            .map(|index| index % (1 << bits))
            .collect::<Vec<_>>();
        let packed = common::pack(&values, bits);
        let data = LongArray::new(&packed).unwrap();
        assert_eq!(unpack(data, bits as u32, count).unwrap(), values);
        let shorter = &packed[..packed.len() - 8];
        assert!(unpack(LongArray::new(shorter).unwrap(), bits as u32, count).is_err());
    }
    let empty = LongArray::new(&[]).unwrap();
    assert!(unpack(empty, 0, 256).is_err());
    assert!(unpack(empty, 65, 256).is_err());
}

#[test]
fn negative_sections_and_signed_longs_are_supported() {
    let registry = common::registry();
    let mut chunk = common::chunk(-1, -3, 16);
    chunk.min_section = -4;
    chunk.sections[0].y = -4;
    let nbt = common::to_nbt(&chunk);
    let chunk = common::parse(&nbt, &registry);
    let samples = chunk.sample(&registry).unwrap();
    assert!(
        samples
            .iter()
            .all(|sample| sample.height == -49 && sample.color == 11)
    );
    assert!(chunk.validate(-1, -3).unwrap());
    assert!(chunk.validate(0, -3).is_err());
    let packed = common::pack(&[15; 4096], 4);
    let data = LongArray::new(&packed).unwrap();
    assert_eq!(data.get(0), -1);
    assert_eq!(unpack(data, 4, 4096).unwrap(), vec![15; 4096]);
}

#[test]
fn single_state_and_empty_columns_follow_vanilla_bedrock_fallback() {
    let registry = common::registry();
    let mut chunk = common::chunk(0, 0, 0);
    let nbt = common::to_nbt(&chunk);
    let parsed = common::parse(&nbt, &registry);
    assert!(
        parsed
            .sample(&registry)
            .unwrap()
            .iter()
            .all(|sample| sample.height == 0 && sample.color == 11)
    );

    chunk.sections[0].block_states = Some(common::ChunkBlockStates {
        palette: vec![common::palette("minecraft:stone")],
        data: None,
    });
    chunk
        .heightmaps
        .insert("WORLD_SURFACE".to_string(), common::pack(&[16; 256], 5));
    let nbt = common::to_nbt(&chunk);
    let parsed = common::parse(&nbt, &registry);
    assert!(
        parsed
            .sample(&registry)
            .unwrap()
            .iter()
            .all(|sample| sample.height == 15)
    );
}

#[test]
fn unknown_states_bad_palettes_and_heightmaps_fail() {
    let registry = common::registry();
    let mut chunk = common::chunk(0, 0, 3);
    chunk.sections[0].block_states.as_mut().unwrap().palette[1].name = "missing:stone".to_string();
    let nbt = common::to_nbt(&chunk);
    assert!(
        RawChunk::parse(&nbt, &registry)
            .unwrap_err()
            .to_string()
            .contains("missing:stone")
    );

    chunk = common::chunk(0, 0, 3);
    chunk.sections[0].block_states.as_mut().unwrap().data = Some(common::pack(&[15; 4096], 4));
    let nbt = common::to_nbt(&chunk);
    assert!(common::parse(&nbt, &registry).sample(&registry).is_err());

    chunk = common::chunk(0, 0, 3);
    chunk.heightmaps.clear();
    let nbt = common::to_nbt(&chunk);
    assert!(common::parse(&nbt, &registry).sample(&registry).is_err());

    chunk = common::chunk(0, 0, 3);
    chunk
        .heightmaps
        .insert("WORLD_SURFACE".to_string(), common::pack(&[31; 256], 5));
    let nbt = common::to_nbt(&chunk);
    assert!(common::parse(&nbt, &registry).sample(&registry).is_err());

    chunk = common::chunk(0, 0, 3);
    chunk.status = "minecraft:unexpected".to_string();
    let nbt = common::to_nbt(&chunk);
    assert!(!common::parse(&nbt, &registry).validate(0, 0).unwrap());
    assert!(common::parse(&nbt, &registry).validate(1, 0).is_err());
}

#[test]
fn light_only_sections_outside_the_world_are_ignored() {
    let registry = common::registry();
    let baseline = common::chunk(0, 0, 5);
    let nbt = common::to_nbt(&baseline);
    let expected = common::parse(&nbt, &registry)
        .sample(&registry)
        .unwrap()
        .map(|sample| (sample.height, sample.color, sample.depth));

    // Vanilla stores sky/block light for one section below the world minimum.
    let mut chunk = common::chunk(0, 0, 5);
    chunk.sections.insert(
        0,
        common::ChunkSection {
            y: -1,
            block_states: None,
        },
    );
    let nbt = common::to_nbt(&chunk);
    let actual = common::parse(&nbt, &registry)
        .sample(&registry)
        .unwrap()
        .map(|sample| (sample.height, sample.color, sample.depth));
    assert_eq!(actual, expected);

    // Vanilla drops every section outside the world range, blocks included.
    chunk.sections[0] = common::ChunkSection {
        y: -1,
        block_states: Some(common::ChunkBlockStates {
            palette: vec![common::palette("minecraft:stone")],
            data: None,
        }),
    };
    let nbt = common::to_nbt(&chunk);
    let actual = common::parse(&nbt, &registry)
        .sample(&registry)
        .unwrap()
        .map(|sample| (sample.height, sample.color, sample.depth));
    assert_eq!(actual, expected);
}

#[test]
fn fluid_depth_includes_the_first_nonfluid_block_and_corrects_surface_color() {
    let mut registry = common::registry();
    registry.set_state(
        "minecraft:test_waterlogged",
        State {
            map_color: 11,
            fluid: true,
            surface_color: 12,
        },
    );

    let mut chunk = common::chunk(0, 0, 6);
    let blocks = chunk.sections[0].block_states.as_mut().unwrap();
    blocks
        .palette
        .push(common::palette_with("minecraft:water", &[("level", "0")]));
    blocks
        .palette
        .push(common::palette("minecraft:test_waterlogged"));
    let mut values = vec![0; 4096];
    values[..2 * 256].fill(1);
    values[2 * 256..6 * 256].fill(2);
    values[5 * 256..6 * 256].fill(3);
    blocks.data = Some(common::pack(&values, 4));
    let nbt = common::to_nbt(&chunk);
    let parsed = common::parse(&nbt, &registry);
    let sample = parsed.sample(&registry).unwrap()[0];
    assert_eq!((sample.height, sample.depth, sample.color), (5, 4, 12));
}

#[test]
fn manually_parsed_nbt_matches_expected_samples_and_rejects_truncation() {
    let registry = common::registry();
    let chunk = common::chunk(-1, -3, 8);
    let encoded = common::to_nbt(&chunk);
    let parsed = common::parse(&encoded, &registry);
    assert_eq!((parsed.x, parsed.z, parsed.min_section), (-1, -3, 0));
    assert_eq!(parsed.status, b"minecraft:full");
    assert_eq!(parsed.sections.len(), 1);
    assert!(parsed.sample(&registry).unwrap().iter().all(|sample| (
        sample.height,
        sample.color,
        sample.depth
    ) == (7, 11, 0)));

    let mut truncated = encoded;
    truncated.pop();
    assert!(RawChunk::parse(&truncated, &registry).is_err());
}

#[test]
fn shading_uses_strict_java_floating_point_thresholds() {
    let terrain = Sample {
        height: 10,
        color: 11,
        depth: 0,
    };
    assert_eq!(shade(terrain, 10.0, 0, 0), 45);
    assert_eq!(shade(terrain, 9.0, 0, 0), 46);
    assert_eq!(shade(terrain, 11.0, 0, 1), 44);
    assert_eq!(shade(terrain, 9.0, -1, -1), 46);
    for (depth, parity, brightness) in [(0, 0, 2), (3, 1, 1), (4, 0, 2), (7, 1, 0), (10, 0, 0)] {
        assert_eq!(
            shade(
                Sample {
                    color: 12,
                    depth,
                    ..terrain
                },
                999.0,
                parity,
                0
            ),
            48 + brightness
        );
    }
}
