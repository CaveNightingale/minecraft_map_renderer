package dev.maprenderer.exporter.mixin;

import dev.maprenderer.exporter.VanillaReference;
import net.minecraft.world.item.MapItem;
import net.minecraft.world.level.Level;
import net.minecraft.world.level.chunk.LevelChunk;
import net.minecraft.world.level.saveddata.maps.MapItemSavedData;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Constant;
import org.spongepowered.asm.mixin.injection.ModifyConstant;
import org.spongepowered.asm.mixin.injection.Redirect;

@Mixin(MapItem.class)
public abstract class MapItemMixin {
    @ModifyConstant(method = "update", constant = @Constant(intValue = 128), require = 3, allow = 3)
    private int maprenderer$size(int original) {
        var reference = VanillaReference.active();
        return reference == null ? original : reference.size();
    }

    @ModifyConstant(method = "update", constant = @Constant(intValue = 64), require = 4, allow = 4)
    private int maprenderer$halfSize(int original) {
        var reference = VanillaReference.active();
        return reference == null ? original : reference.size() / 2;
    }

    @Redirect(method = "update", at = @At(value = "INVOKE",
            target = "Lnet/minecraft/world/level/Level;getChunk(II)Lnet/minecraft/world/level/chunk/LevelChunk;"))
    private LevelChunk maprenderer$fullChunk(Level level, int x, int z) {
        var reference = VanillaReference.active();
        return reference == null ? level.getChunk(x, z) : reference.chunk(x, z);
    }

    @Redirect(method = "update", at = @At(value = "INVOKE",
            target = "Lnet/minecraft/world/level/saveddata/maps/MapItemSavedData;updateColor(IIB)Z"))
    private boolean maprenderer$writeColor(MapItemSavedData map, int x, int z, byte color) {
        var reference = VanillaReference.active();
        return reference == null ? map.updateColor(x, z, color) : reference.updateColor(x, z, color);
    }
}
