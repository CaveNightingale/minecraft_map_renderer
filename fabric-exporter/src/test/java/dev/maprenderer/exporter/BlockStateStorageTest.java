package dev.maprenderer.exporter;

import static org.junit.jupiter.api.Assertions.assertEquals;

import net.minecraft.SharedConstants;
import net.minecraft.core.registries.BuiltInRegistries;
import net.minecraft.nbt.CompoundTag;
import net.minecraft.nbt.NbtOps;
import net.minecraft.server.Bootstrap;
import net.minecraft.world.level.block.state.BlockState;
import org.junit.jupiter.api.BeforeAll;
import org.junit.jupiter.api.Test;

class BlockStateStorageTest {
    @BeforeAll
    static void bootstrap() {
        SharedConstants.tryDetectVersion();
        Bootstrap.bootStrap();
    }

    @Test
    void registryKeysMatchEveryStateEncodedByMinecraftsStorageCodec() {
        for (var block : BuiltInRegistries.BLOCK) {
            for (var state : block.getStateDefinition().getPossibleStates()) {
                var stored = BlockState.CODEC.encodeStart(NbtOps.INSTANCE, state).getOrThrow();
                assertEquals(BlockStateExporter.stateKey(state),
                        ExportIntegrationTest.storageKey((CompoundTag) stored));
            }
        }
    }
}
