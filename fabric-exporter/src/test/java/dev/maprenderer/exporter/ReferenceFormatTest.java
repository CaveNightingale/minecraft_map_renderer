package dev.maprenderer.exporter;

import static org.junit.jupiter.api.Assertions.assertArrayEquals;
import static org.junit.jupiter.api.Assertions.assertDoesNotThrow;
import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertThrows;

import org.junit.jupiter.api.Test;

class ReferenceFormatTest {
    @Test
    void validatesThreeClassesAndUnsignedColors() {
        var colors = new byte[ReferenceFormat.SIZE * ReferenceFormat.SIZE];
        var chunks = new byte[ReferenceFormat.CHUNKS * ReferenceFormat.CHUNKS];
        chunks[0] = ReferenceFormat.PARTIAL;
        chunks[1] = ReferenceFormat.FULL;
        colors[16] = (byte) 255;
        assertArrayEquals(new int[] {16382, 1, 1}, ReferenceFormat.validate(colors, chunks, true));
    }

    @Test
    void fullyGeneratedTransparentChunkIsStillCovered() {
        var colors = new byte[ReferenceFormat.SIZE * ReferenceFormat.SIZE];
        var chunks = new byte[ReferenceFormat.CHUNKS * ReferenceFormat.CHUNKS];
        chunks[0] = ReferenceFormat.FULL;
        assertEquals(1, ReferenceFormat.validate(colors, chunks, false)[2]);
    }

    @Test
    void rejectsVisiblePixelsInMissingAndPartialChunks() {
        for (byte status : new byte[] {ReferenceFormat.MISSING, ReferenceFormat.PARTIAL}) {
            var colors = new byte[ReferenceFormat.SIZE * ReferenceFormat.SIZE];
            var chunks = new byte[ReferenceFormat.CHUNKS * ReferenceFormat.CHUNKS];
            chunks[chunks.length - 1] = status;
            colors[colors.length - 1] = 1;
            assertThrows(IllegalArgumentException.class, () -> ReferenceFormat.validate(colors, chunks, false));
        }
    }

    @Test
    void rejectsIncorrectFileLengthsAndUnknownStatus() {
        var colors = new byte[ReferenceFormat.SIZE * ReferenceFormat.SIZE];
        var chunks = new byte[ReferenceFormat.CHUNKS * ReferenceFormat.CHUNKS];
        assertThrows(IllegalArgumentException.class, () -> ReferenceFormat.validate(new byte[1], chunks, false));
        assertThrows(IllegalArgumentException.class, () -> ReferenceFormat.validate(colors, new byte[1], false));
        chunks[0] = (byte) 255;
        assertThrows(IllegalArgumentException.class, () -> ReferenceFormat.validate(colors, chunks, false));
    }

    @Test
    void fixtureMustActuallyContainEveryClass() {
        var colors = new byte[ReferenceFormat.SIZE * ReferenceFormat.SIZE];
        var chunks = new byte[ReferenceFormat.CHUNKS * ReferenceFormat.CHUNKS];
        assertDoesNotThrow(() -> ReferenceFormat.validate(colors, chunks, false));
        assertThrows(IllegalArgumentException.class, () -> ReferenceFormat.validate(colors, chunks, true));
    }

}
