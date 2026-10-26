package dev.maprenderer.exporter;

public final class ReferenceFormat {
    public static final int SIZE = 2048;
    public static final int CHUNKS = SIZE / 16;
    public static final byte MISSING = 0;
    public static final byte PARTIAL = 1;
    public static final byte FULL = 2;

    private ReferenceFormat() {}

    public static int[] validate(byte[] colors, byte[] chunks, boolean requireAllStates) {
        if (colors.length != SIZE * SIZE || chunks.length != CHUNKS * CHUNKS) {
            throw new IllegalArgumentException("Incorrect reference file dimensions");
        }
        var counts = new int[3];
        for (int cz = 0; cz < CHUNKS; cz++) {
            for (int cx = 0; cx < CHUNKS; cx++) {
                int status = Byte.toUnsignedInt(chunks[cz * CHUNKS + cx]);
                if (status > FULL) {
                    throw new IllegalArgumentException("Invalid reference chunk status at " + cx + "," + cz);
                }
                counts[status]++;
                if (status != FULL) {
                    for (int z = cz * 16; z < cz * 16 + 16; z++) {
                        for (int x = cx * 16; x < cx * 16 + 16; x++) {
                            if (colors[z * SIZE + x] != 0) {
                                throw new IllegalArgumentException("Non-full chunk has a visible pixel at " + x + "," + z);
                            }
                        }
                    }
                }
            }
        }
        if (requireAllStates && (counts[0] == 0 || counts[1] == 0 || counts[2] == 0)) {
            throw new IllegalArgumentException("Test fixture must contain missing, partial and full chunks");
        }
        return counts;
    }
}
