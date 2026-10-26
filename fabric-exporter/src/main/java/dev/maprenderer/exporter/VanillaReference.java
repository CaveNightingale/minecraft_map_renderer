package dev.maprenderer.exporter;

import com.mojang.authlib.GameProfile;
import java.util.HashMap;
import java.util.Map;
import java.util.UUID;
import net.minecraft.core.registries.Registries;
import net.minecraft.server.MinecraftServer;
import net.minecraft.server.level.ClientInformation;
import net.minecraft.server.level.ServerLevel;
import net.minecraft.server.level.ServerPlayer;
import net.minecraft.world.item.Items;
import net.minecraft.world.item.MapItem;
import net.minecraft.world.level.ChunkPos;
import net.minecraft.world.level.biome.Biomes;
import net.minecraft.world.level.chunk.EmptyLevelChunk;
import net.minecraft.world.level.chunk.ImposterProtoChunk;
import net.minecraft.world.level.chunk.LevelChunk;
import net.minecraft.world.level.chunk.status.ChunkStatus;
import net.minecraft.world.level.saveddata.maps.MapItemSavedData;

public final class VanillaReference {
    private static final ThreadLocal<VanillaReference> ACTIVE = new ThreadLocal<>();
    private final Map<ChunkPos, LevelChunk> chunks = new HashMap<>();
    private final byte[] statuses = new byte[ReferenceFormat.CHUNKS * ReferenceFormat.CHUNKS];
    private final byte[] colors = new byte[ReferenceFormat.SIZE * ReferenceFormat.SIZE];
    private final LevelChunk empty;

    private VanillaReference(ServerLevel level) {
        empty = new EmptyLevelChunk(level, ChunkPos.ZERO,
                level.registryAccess().lookupOrThrow(Registries.BIOME).getOrThrow(Biomes.PLAINS));
    }

    public static VanillaReference capture(ServerLevel level) {
        var reference = new VanillaReference(level);
        for (int z = -1; z < ReferenceFormat.CHUNKS; z++) {
            for (int x = 0; x < ReferenceFormat.CHUNKS; x++) {
                var holder = level.getChunkSource().chunkMap.getUpdatingChunkIfPresent(ChunkPos.pack(x, z));
                var chunk = holder == null ? null : holder.getLatestChunk();
                if (chunk == null) {
                    continue;
                }
                boolean full = chunk.getPersistedStatus() == ChunkStatus.FULL;
                if (z >= 0) {
                    reference.statuses[z * ReferenceFormat.CHUNKS + x] =
                            full ? ReferenceFormat.FULL : ReferenceFormat.PARTIAL;
                }
                if (full) {
                    if (chunk instanceof ImposterProtoChunk wrapped) {
                        chunk = wrapped.getWrapped();
                    }
                    if (!(chunk instanceof LevelChunk loaded)) {
                        throw new IllegalStateException("FULL holder does not contain a LevelChunk at " + x + "," + z);
                    }
                    reference.chunks.put(new ChunkPos(x, z), loaded);
                }
            }
        }
        return reference;
    }

    public static VanillaReference active() {
        return ACTIVE.get();
    }

    public byte[] render(MinecraftServer server) {
        if (ACTIVE.get() != null) {
            throw new IllegalStateException("Nested reference rendering");
        }
        var level = server.overworld();
        var observer = new ServerPlayer(server, level,
                new GameProfile(new UUID(0, 1), "MapReference"), ClientInformation.createDefault());
        var map = MapItemSavedData.createFresh(size() / 2, size() / 2, (byte) 0, false, false, level.dimension());
        if (map.centerX != size() / 2 || map.centerZ != size() / 2) {
            throw new IllegalStateException("Unexpected vanilla map center");
        }
        observer.setPos(size() / 2.0 + 0.5, level.getSeaLevel(), size() / 2.0 + 0.5);
        ACTIVE.set(this);
        try {
            var item = (MapItem) Items.FILLED_MAP;
            for (int step = 0; step < 16; step++) {
                item.update(level, observer, map);
            }
        } finally {
            ACTIVE.remove();
        }
        return colors;
    }

    public int size() {
        return ReferenceFormat.SIZE;
    }

    public byte[] statuses() {
        return statuses.clone();
    }

    public LevelChunk chunk(int x, int z) {
        return chunks.getOrDefault(new ChunkPos(x, z), empty);
    }

    public boolean updateColor(int x, int z, byte color) {
        int index = z * size() + x;
        boolean changed = colors[index] != color;
        colors[index] = color;
        return changed;
    }
}
