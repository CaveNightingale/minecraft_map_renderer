export type { MapIndex, Tile, TileData } from "./protocol.js";
export interface Position {
    readonly x: number;
    /** Canvas Y corresponds to world Z. */
    readonly y: number;
}
export interface View extends Position {
    readonly scale: number;
}
export interface RenderOptions {
    readonly initialPosition?: Position;
    readonly initialScale?: number;
    readonly movable?: boolean;
    readonly scaleRange?: {
        readonly min: number;
        readonly max: number;
    };
    readonly maxConcurrentRequests?: number;
    readonly maxCachedTiles?: number;
}
export declare class MapErrorEvent extends Event {
    readonly error: Error;
    constructor(error: Error);
}
export declare class Map extends EventTarget {
    private readonly baseUrl;
    private readonly lifetime;
    private readonly cache;
    private readonly requests;
    private readonly failures;
    private readonly tileLookup;
    private needed;
    private visible;
    private index;
    private canvas;
    private context;
    private observer;
    private savedCanvas;
    private frame;
    private drag;
    private started;
    private destroyed;
    private loaded;
    private width;
    private height;
    private dpr;
    private x;
    private y;
    private scale;
    private minScale;
    private maxScale;
    private concurrent;
    private cachedTiles;
    constructor(baseUrl: string | URL);
    get view(): View;
    render(canvas: HTMLCanvasElement, options?: RenderOptions): Promise<void>;
    setView(position: Position, scale?: number): void;
    destroy(): void;
    private readonly schedule;
    private refresh;
    private evict;
    private pump;
    private loadTile;
    private draw;
    private readonly pointerDown;
    private readonly pointerMove;
    private readonly pointerUp;
    private readonly wheel;
}
