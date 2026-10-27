export interface Tile {
    readonly x: number;
    readonly z: number;
    readonly hash: string;
}
export interface MapIndex {
    readonly format: 1;
    readonly tileSize: number;
    readonly palette: readonly number[];
    readonly tiles: readonly Tile[];
}
export interface TileData {
    readonly colors: Uint8Array;
    readonly coverage: Uint8Array;
}
export declare function parseIndex(value: unknown): MapIndex;
export declare function decodeTile(buffer: ArrayBuffer, tileSize: number): Promise<TileData>;
