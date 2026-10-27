# Minecraft Map Renderer

This project renders Minecraft regions into vanilla-style big maps, without opening the game.

## Minecraft Compatibility

Currently targeting Minecraft 26.x, but we may add support for other versions in the future.

## Usage

First, you need to dump your BlockState registry from the game. Copy the fabric mod to your Minecraft mods folder and run the game once to generate the registry file `blockstate_registry.json`. For neoforge and other mod loaders, please help yourself to generate the registry file accordingly and we are not going to provide mod for them.

With that json, you can use the renderer to convert Minecraft region files into big maps. Typically, you will use
```bash
minecraft-map-renderer \
    --input /path/to/server/world/dimensions/minecraft/overworld/region \
    --output /path/to/output/directory/overworld \
    --blockstate-registry /path/to/blockstate_registry.json
minecraft-map-renderer \
    --input /path/to/server/world/dimensions/minecraft/the_end/region \
    --output /path/to/output/directory/the_end \
    --blockstate-registry /path/to/blockstate_registry.json
minecraft-map-renderer \
    --input /path/to/server/world/dimensions/minecraft/the_nether/region \
    --output /path/to/output/directory/the_nether \
    --blockstate-registry /path/to/blockstate_registry.json
```

Only fully generated chunks will be rendered.

The output should contains an `index.map` showing which parts of the world have been rendered, and several `<hash>.mpt` files representing the rendered map tiles, containing the raw color id (61 * 3 colors in vanilla minecraft maps).

Finally, you will have a npm package that can be used render the generated map tiles for display in a web application. Use it like:

```html
<canvas id="mapCanvas"></canvas>
<script type="module">
import { Map } from 'minecraft-map-renderer';

const map = new Map('/path/to/output/directory/overworld');

const canvas = document.getElementById('mapCanvas');

map.render(canvas, {
    initialPosition: { x: 0, y: 0 },
    movable: true,
    scaleRange: { min: 1, max: 4 }
});
</script>
```

For an example, you can generate the map into the `target/reference-map` directory and then use `npm --prefix web run example` to view it in a web application.

## About the format

Each MPT starts with a fixed 8-byte magic header `MPT\x02\0\0\0\0`, followed by the 64-bit original length, and then the compressed tile data using the Gzip algorithm. The tile data represents is the array (tile_size * tile_size) of raw color IDs for the map tile, and a (tile_size / 16 * tile_size / 16) bitmap indicating the coverage of each 16x16 chunk within the tile.

Each MAP file (`index.map`) contains a JSON object with the following structure:
```json
{
    "format": 1,
    "tile_size": 128,
    "palette": [0, 1, 2, ...],
    "tiles": [
        { "x": 0, "z": 0, "hash": "abcd1234..." },
        { "x": 1, "z": 0, "hash": "efgh5678..." }
    ]
}
```
- `format` indicates the index format version.
- `tile_size` is the size of each tile in pixels.
- `palette` is the color (ARGB) for each color id used in the map tiles.
- `tiles` is an array of tile metadata, where each tile has `x` and `z` coordinates and a `hash` corresponding to the `<hash>.mpt` file.

## Render.sh script

There is a template of the render script available in this repository as `render.sh`. You can modify it according to your server setup and preferences.

Things you may like to modify include the server directory, path to the blockstate registry, and output directory for the rendered maps.

## License

The code in this repository is dedicated to the public domain under
[CC0 1.0 Universal](LICENSE). Third-party components keep their own licenses;
see [THIRD-PARTY-NOTICES.md](THIRD-PARTY-NOTICES.md) for the full list and the
notices you must keep when you redistribute a binary.

This project is not affiliated with, endorsed by or sponsored by Mojang Studios
or Microsoft. *Minecraft* is a trademark of Mojang Synergies AB. No Minecraft
code or assets are distributed here, and the CC0 dedication does not extend to
them. You need a legitimate copy of the game to use the exporter.
