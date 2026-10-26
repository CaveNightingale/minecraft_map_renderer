package dev.maprenderer.exporter;

import java.io.IOException;
import java.nio.file.Files;
import java.util.Arrays;
import java.util.Random;
import net.fabricmc.api.DedicatedServerModInitializer;
import net.fabricmc.fabric.api.event.lifecycle.v1.ServerLifecycleEvents;
import net.minecraft.server.MinecraftServer;
import net.minecraft.world.level.chunk.status.ChunkStatus;
import net.minecraft.world.level.storage.LevelResource;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

public final class MapExporter implements DedicatedServerModInitializer {
    private static final Logger LOGGER = LoggerFactory.getLogger("map-renderer-exporter");
    private record Position(int x, int y, int z) {}
    private record Counts(int missing, int partial, int full) {}
    private record Reference(int format, String seed, Position spawn, Counts chunks) {}

    @Override
    public void onInitializeServer() {
        ServerLifecycleEvents.SERVER_STARTED.register(server -> {
            try {
                export(server);
            } catch (IOException | RuntimeException exception) {
                LOGGER.error("Map export failed", exception);
                throw new IllegalStateException("Map export failed", exception);
            } finally {
                server.halt(false);
            }
        });
    }

    private static void export(MinecraftServer server) throws IOException {
        var world = server.getWorldPath(LevelResource.ROOT);
        var referenceJson = world.resolve("reference.json");
        // reference.json marks a complete export, so retire it before replacing outputs.
        Files.deleteIfExists(referenceJson);
        var registry = BlockStateExporter.export(world.resolve("blockstate_registry.json"));
        generateChunks(server);
        save(server);
        var reference = VanillaReference.capture(server.overworld());
        var colors = reference.render(server);
        var mask = reference.statuses();
        var counts = ReferenceFormat.validate(colors, mask, true);
        if (!Arrays.equals(mask, VanillaReference.capture(server.overworld()).statuses())) {
            throw new IllegalStateException("Map sampling changed chunk generation status");
        }
        save(server);
        Files.write(world.resolve("reference.colors"), colors);
        Files.write(world.resolve("reference.chunks"), mask);
        var spawn = server.getRespawnData().pos();
        ExportFiles.writeJson(referenceJson, new Reference(2,
                Long.toString(server.overworld().getSeed()), new Position(spawn.getX(), spawn.getY(), spawn.getZ()),
                new Counts(counts[0], counts[1], counts[2])));
        LOGGER.info("Map export complete in {}; {} states; missing={}, partial={}, full={}",
                world, registry.states().size(), counts[0], counts[1], counts[2]);
    }

    private static void generateChunks(MinecraftServer server) {
        var level = server.overworld();
        var source = level.getChunkSource();
        var random = new Random(level.getSeed());
        for (int patch = 0; patch < 4; patch++) {
            int originX = (patch % 2) * 64 + 8 + random.nextInt(40);
            int originZ = (patch / 2) * 64 + 8 + random.nextInt(40);
            for (int z = originZ; z < originZ + 8; z++) {
                for (int x = originX; x < originX + 8; x++) {
                    source.getChunk(x, z, ChunkStatus.FULL, true);
                }
            }
        }
        source.getChunk(64, 64, ChunkStatus.NOISE, true);
    }

    private static void save(MinecraftServer server) {
        if (!server.saveEverything(true, true, true)) {
            throw new IllegalStateException("Server refused to save the reference world");
        }
        server.overworld().getChunkSource().chunkMap.synchronize(true).join();
    }
}
