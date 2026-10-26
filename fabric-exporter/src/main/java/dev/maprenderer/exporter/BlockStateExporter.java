package dev.maprenderer.exporter;

import java.io.IOException;
import java.nio.file.Path;
import java.util.Comparator;
import java.util.Map;
import java.util.TreeMap;
import java.util.stream.Collectors;
import net.minecraft.core.BlockPos;
import net.minecraft.core.Direction;
import net.minecraft.core.registries.BuiltInRegistries;
import net.minecraft.world.level.EmptyBlockGetter;
import net.minecraft.world.level.block.state.BlockState;
import net.minecraft.world.level.block.state.properties.Property;
import net.minecraft.world.level.material.MapColor;

public final class BlockStateExporter {
    public record State(int mapColor, boolean fluid, int surfaceColor) {}

    public record Registry(int[] palette, Map<String, State> states) {}

    private BlockStateExporter() {}

    public static Registry export(Path destination) throws IOException {
        var states = new TreeMap<String, State>();
        for (var block : BuiltInRegistries.BLOCK) {
            for (var state : block.getStateDefinition().getPossibleStates()) {
                var fluid = state.getFluidState();
                var surface = !fluid.isEmpty()
                        && !state.isFaceSturdy(EmptyBlockGetter.INSTANCE, BlockPos.ZERO, Direction.UP)
                        ? fluid.createLegacyBlock() : state;
                states.put(stateKey(state), new State(
                        state.getMapColor(EmptyBlockGetter.INSTANCE, BlockPos.ZERO).id,
                        !fluid.isEmpty(),
                        surface.getMapColor(EmptyBlockGetter.INSTANCE, BlockPos.ZERO).id));
            }
        }
        var palette = new int[256];
        for (int id = 0; id < palette.length; id++) {
            palette[id] = MapColor.getColorFromPackedId(id);
        }
        var registry = new Registry(palette, states);
        ExportFiles.writeJson(destination, registry);
        return registry;
    }

    public static String stateKey(BlockState state) {
        var name = BuiltInRegistries.BLOCK.getKey(state.getBlock()).toString();
        if (state.getProperties().isEmpty()) {
            return name;
        }
        return name + state.getProperties().stream().sorted(Comparator.comparing(Property::getName))
                .map(property -> property.getName() + "=" + propertyValue(state, property))
                .collect(Collectors.joining(",", "[", "]"));
    }

    private static <T extends Comparable<T>> String propertyValue(BlockState state, Property<T> property) {
        return property.getName(state.getValue(property));
    }
}
