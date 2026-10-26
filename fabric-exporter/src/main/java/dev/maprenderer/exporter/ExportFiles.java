package dev.maprenderer.exporter;

import com.google.gson.Gson;
import com.google.gson.GsonBuilder;
import java.io.IOException;
import java.nio.charset.StandardCharsets;
import java.nio.file.Files;
import java.nio.file.Path;

public final class ExportFiles {
    private static final Gson JSON = new GsonBuilder().setPrettyPrinting().disableHtmlEscaping().create();

    private ExportFiles() {}

    public static void writeJson(Path file, Object value) throws IOException {
        Files.writeString(file, JSON.toJson(value) + "\n", StandardCharsets.UTF_8);
    }
}
