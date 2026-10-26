package dev.maprenderer.exporter;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertTrue;

import com.google.gson.JsonParser;
import java.nio.file.Files;
import java.nio.file.Path;
import net.minecraft.SharedConstants;
import net.minecraft.server.Bootstrap;
import net.minecraft.nbt.CompoundTag;
import net.minecraft.world.level.Level;
import net.minecraft.world.level.ChunkPos;
import net.minecraft.world.level.chunk.status.ChunkStatus;
import net.minecraft.world.level.chunk.storage.RegionStorageInfo;
import net.minecraft.world.level.chunk.storage.SimpleRegionStorage;
import net.minecraft.world.level.chunk.storage.SerializableChunkData;
import net.minecraft.util.datafix.DataFixers;
import net.minecraft.util.datafix.DataFixTypes;
import java.util.stream.Collectors;
import org.junit.jupiter.api.BeforeAll;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.condition.EnabledIfSystemProperty;

@EnabledIfSystemProperty(named = "maprenderer.testExport", matches = ".+")
class ExportIntegrationTest {
    @BeforeAll
    static void bootstrapGameRegistries() {
        SharedConstants.tryDetectVersion();
        Bootstrap.bootStrap();
    }

    @Test
    void verifiesActualServerExport() throws Exception {
        var root = Path.of(System.getProperty("maprenderer.testExport"));
        var reference = JsonParser.parseString(Files.readString(root.resolve("reference.json"))).getAsJsonObject();
        var registry = JsonParser.parseString(Files.readString(root.resolve("blockstate_registry.json"))).getAsJsonObject();
        assertEquals(2, reference.get("format").getAsInt());
        assertEquals(4, reference.size());
        Long.parseLong(reference.get("seed").getAsString());

        var colors = Files.readAllBytes(root.resolve("reference.colors"));
        var mask = Files.readAllBytes(root.resolve("reference.chunks"));
        var counts = ReferenceFormat.validate(colors, mask, true);
        assertEquals(counts[0], reference.getAsJsonObject("chunks").get("missing").getAsInt());
        assertEquals(counts[1], reference.getAsJsonObject("chunks").get("partial").getAsInt());
        assertEquals(counts[2], reference.getAsJsonObject("chunks").get("full").getAsInt());
        assertTrue(counts[2] >= 256);
        assertEquals(ReferenceFormat.PARTIAL, mask[64 * 128 + 64]);
        assertTrue(Files.isRegularFile(root.resolve("level.dat")));
        boolean visible = false;
        for (byte color : colors) {
            visible |= color != 0;
        }
        assertTrue(visible, "Reference must contain terrain, not only void");

        assertEquals(4, registry.size());
        assertEquals(2, registry.get("format").getAsInt());
        assertEquals(SharedConstants.getCurrentVersion().dataVersion().version(), registry.get("dataVersion").getAsInt());
        assertEquals(256, registry.getAsJsonArray("palette").size());
        assertEquals(0, registry.getAsJsonArray("palette").get(0).getAsInt());
        var states = registry.getAsJsonObject("states");
        assertTrue(states.has("minecraft:air"));
        assertTrue(states.has("minecraft:water[level=0]"));
        for (var entry : states.entrySet()) {
            var state = entry.getValue().getAsJsonObject();
            assertEquals(3, state.size());
            assertTrue(state.get("mapColor").getAsInt() >= 0 && state.get("mapColor").getAsInt() < 64);
            assertTrue(state.get("surfaceColor").getAsInt() >= 0 && state.get("surfaceColor").getAsInt() < 64);
        }
        org.junit.jupiter.api.Assertions.assertFalse(states.getAsJsonObject("minecraft:air").get("fluid").getAsBoolean());
        assertTrue(states.getAsJsonObject("minecraft:water[level=0]").get("fluid").getAsBoolean());
        var info = new RegionStorageInfo("export-integration-test", Level.OVERWORLD, "chunk");
        try (var storage = new SimpleRegionStorage(info, root.resolve("dimensions/minecraft/overworld/region"),
                DataFixers.getDataFixer(), false, DataFixTypes.CHUNK)) {
            for (int z = 0; z < 128; z++) {
                for (int x = 0; x < 128; x++) {
                    var saved = storage.read(new ChunkPos(x, z)).join();
                    byte status = ReferenceFormat.MISSING;
                    if (saved.isPresent()) {
                        var tag = saved.orElseThrow();
                        status = SerializableChunkData.getChunkStatusFromTag(tag) == ChunkStatus.FULL
                                ? ReferenceFormat.FULL : ReferenceFormat.PARTIAL;
                        for (var section : tag.getList("sections").orElseThrow()) {
                            var blocks = ((CompoundTag) section).getCompound("block_states");
                            if (blocks.isEmpty()) {
                                continue;
                            }
                            for (var entry : blocks.orElseThrow().getList("palette").orElseThrow()) {
                                assertTrue(states.has(storageKey((CompoundTag) entry)));
                            }
                        }
                    }
                    assertEquals(status, mask[z * 128 + x], "Persisted status at " + x + "," + z);
                }
            }
        }
    }

    static String storageKey(CompoundTag tag) {
        var name = tag.getString("Name").orElseThrow();
        var properties = tag.getCompound("Properties").orElseGet(CompoundTag::new);
        return properties.isEmpty() ? name : name + properties.keySet().stream().sorted()
                .map(key -> key + "=" + properties.getString(key).orElseThrow())
                .collect(Collectors.joining(",", "[", "]"));
    }
}
