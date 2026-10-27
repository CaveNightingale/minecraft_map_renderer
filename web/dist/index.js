import { decodeTile, parseIndex } from "./protocol.js";
export class MapErrorEvent extends Event {
    error;
    constructor(error) {
        super("error");
        this.error = error;
    }
}
export class Map extends EventTarget {
    baseUrl;
    lifetime = new AbortController();
    cache = new globalThis.Map();
    requests = new globalThis.Map();
    failures = new globalThis.Map();
    tileLookup = new globalThis.Map();
    needed = new Set();
    visible = [];
    index = null;
    canvas = null;
    context = null;
    observer = null;
    savedCanvas = null;
    frame = null;
    drag = null;
    started = false;
    destroyed = false;
    loaded = false;
    width = 0;
    height = 0;
    dpr = 1;
    x = 0;
    y = 0;
    scale = 1;
    minScale = 1;
    maxScale = 4;
    concurrent = 4;
    cachedTiles = 32;
    constructor(baseUrl) {
        super();
        this.baseUrl = new URL(baseUrl, typeof document === "undefined" ? undefined : document.baseURI);
        if (!["http:", "https:"].includes(this.baseUrl.protocol)) {
            throw new Error("Map base URL must use HTTP or HTTPS");
        }
        if (!this.baseUrl.pathname.endsWith("/")) {
            this.baseUrl.pathname += "/";
        }
    }
    get view() {
        return { x: this.x, y: this.y, scale: this.scale };
    }
    async render(canvas, options = {}) {
        if (this.started || this.destroyed) {
            throw new Error("A Map can be rendered only once; create a new instance");
        }
        const position = options.initialPosition ?? { x: 0, y: 0 };
        const range = options.scaleRange ?? { min: 1, max: 4 };
        const scale = options.initialScale ?? Math.min(range.max, Math.max(range.min, 1));
        const concurrent = options.maxConcurrentRequests ?? 4;
        const cached = options.maxCachedTiles ?? 32;
        if (!Number.isFinite(range.min) || !Number.isFinite(range.max) ||
            range.min <= 0 || range.max < range.min ||
            !Number.isFinite(position.x) || !Number.isFinite(position.y) ||
            !Number.isFinite(scale) || scale < range.min || scale > range.max ||
            !Number.isSafeInteger(concurrent) || concurrent < 1 ||
            !Number.isSafeInteger(cached) || cached < 1 ||
            (options.movable !== undefined && typeof options.movable !== "boolean")) {
            throw new Error("Invalid position, scale range, initial scale, concurrency or cache size");
        }
        const context = canvas.getContext("2d");
        if (!context) {
            throw new Error("Canvas 2D is not available");
        }
        this.started = true;
        this.x = position.x;
        this.y = position.y;
        this.scale = scale;
        this.minScale = range.min;
        this.maxScale = range.max;
        this.concurrent = concurrent;
        this.cachedTiles = cached;
        try {
            const url = new URL("index.map", this.baseUrl);
            const response = await fetch(url, { signal: this.lifetime.signal, cache: "no-cache" });
            if (!response.ok) {
                throw new Error(`${url}: HTTP ${response.status}`);
            }
            this.index = parseIndex(await response.json());
            this.lifetime.signal.throwIfAborted();
            for (const tile of this.index.tiles) {
                this.tileLookup.set(`${tile.x},${tile.z}`, tile);
            }
            this.canvas = canvas;
            this.context = context;
            this.savedCanvas = {
                width: canvas.width, height: canvas.height,
                cssWidth: canvas.style.width, cssHeight: canvas.style.height,
                touchAction: canvas.style.touchAction,
            };
            const signal = this.lifetime.signal;
            if (options.movable ?? true) {
                canvas.style.touchAction = "none";
                canvas.addEventListener("pointerdown", this.pointerDown, { signal });
                canvas.addEventListener("pointermove", this.pointerMove, { signal });
                canvas.addEventListener("pointerup", this.pointerUp, { signal });
                canvas.addEventListener("pointercancel", this.pointerUp, { signal });
                canvas.addEventListener("lostpointercapture", () => { this.drag = null; }, { signal });
            }
            canvas.addEventListener("wheel", this.wheel, { signal, passive: false });
            window.addEventListener("resize", this.schedule, { signal });
            this.observer = new ResizeObserver(this.schedule);
            this.observer.observe(canvas);
            this.refresh();
        }
        catch (error) {
            this.destroy();
            throw new Error(`Cannot initialize map at ${this.baseUrl}: ${error instanceof Error ? error.message : String(error)}`, { cause: error });
        }
    }
    setView(position, scale = this.scale) {
        if (!this.canvas || this.destroyed) {
            throw new Error("Map is not rendered or has been destroyed");
        }
        if (!Number.isFinite(position.x) || !Number.isFinite(position.y) ||
            !Number.isFinite(scale) || scale < this.minScale || scale > this.maxScale) {
            throw new Error("Invalid view position or scale outside scaleRange");
        }
        this.x = position.x;
        this.y = position.y;
        this.scale = scale;
        this.loaded = false;
        this.schedule();
        this.dispatchEvent(new CustomEvent("viewchange", { detail: this.view }));
    }
    destroy() {
        if (this.destroyed) {
            return;
        }
        this.destroyed = true;
        this.lifetime.abort();
        for (const controller of this.requests.values()) {
            controller.abort();
        }
        this.observer?.disconnect();
        if (this.frame !== null) {
            cancelAnimationFrame(this.frame);
        }
        const canvas = this.canvas;
        if (canvas && this.savedCanvas) {
            if (this.drag && canvas.hasPointerCapture(this.drag.pointerId)) {
                canvas.releasePointerCapture(this.drag.pointerId);
            }
            canvas.width = this.savedCanvas.width;
            canvas.height = this.savedCanvas.height;
            canvas.style.width = this.savedCanvas.cssWidth;
            canvas.style.height = this.savedCanvas.cssHeight;
            canvas.style.touchAction = this.savedCanvas.touchAction;
        }
        this.drag = null;
        this.canvas = null;
        this.context = null;
        this.index = null;
        this.visible = [];
        this.needed.clear();
        this.cache.clear();
        this.failures.clear();
        this.tileLookup.clear();
    }
    schedule = () => {
        if (this.destroyed || this.frame !== null) {
            return;
        }
        this.frame = requestAnimationFrame(() => {
            this.frame = null;
            this.refresh();
        });
    };
    refresh() {
        const canvas = this.canvas;
        const index = this.index;
        if (!canvas || !index || this.destroyed) {
            return;
        }
        const width = canvas.clientWidth;
        const height = canvas.clientHeight;
        const dpr = window.devicePixelRatio;
        if (width !== this.width || height !== this.height || dpr !== this.dpr) {
            this.loaded = false;
            this.width = width;
            this.height = height;
            this.dpr = dpr;
            canvas.width = Math.round(width * dpr);
            canvas.height = Math.round(height * dpr);
            // Preserve intrinsic CSS size when backing-store resizing would change layout.
            if (canvas.clientWidth !== width) {
                canvas.style.width = `${width}px`;
            }
            if (canvas.clientHeight !== height) {
                canvas.style.height = `${height}px`;
            }
        }
        const left = (this.x - width / (2 * this.scale)) / index.tileSize;
        const top = (this.y - height / (2 * this.scale)) / index.tileSize;
        const right = (this.x + width / (2 * this.scale)) / index.tileSize;
        const bottom = (this.y + height / (2 * this.scale)) / index.tileSize;
        const minX = Math.floor(left), maxX = Math.ceil(right) - 1;
        const minZ = Math.floor(top), maxZ = Math.ceil(bottom) - 1;
        const visible = [];
        if (width > 0 && height > 0) {
            if ([minX, maxX, minZ, maxZ].every(Number.isSafeInteger) &&
                (maxX - minX + 1) * (maxZ - minZ + 1) < index.tiles.length) {
                for (let x = minX; x <= maxX; x++) {
                    for (let z = minZ; z <= maxZ; z++) {
                        const tile = this.tileLookup.get(`${x},${z}`);
                        if (tile) {
                            visible.push(tile);
                        }
                    }
                }
            }
            else {
                for (const tile of index.tiles) {
                    if (tile.x >= minX && tile.x <= maxX && tile.z >= minZ && tile.z <= maxZ) {
                        visible.push(tile);
                    }
                }
            }
        }
        this.visible = visible;
        this.needed = new Set(visible.map((tile) => tile.hash));
        for (const [hash, controller] of this.requests) {
            if (!this.needed.has(hash)) {
                controller.abort();
            }
        }
        for (const hash of this.failures.keys()) {
            if (!this.needed.has(hash)) {
                this.failures.delete(hash);
            }
        }
        for (const hash of this.needed) {
            const cached = this.cache.get(hash);
            if (cached) {
                this.cache.delete(hash);
                this.cache.set(hash, cached);
            }
        }
        this.evict();
        this.pump();
        this.draw();
        if (!this.loaded && [...this.needed].every((hash) => this.cache.has(hash))) {
            this.loaded = true;
            this.dispatchEvent(new Event("load"));
        }
    }
    evict() {
        for (const hash of this.cache.keys()) {
            if (this.cache.size <= this.cachedTiles) {
                break;
            }
            if (!this.needed.has(hash)) {
                this.cache.delete(hash);
            }
        }
    }
    pump() {
        if (this.destroyed) {
            return;
        }
        for (const hash of this.needed) {
            if (this.requests.size >= this.concurrent) {
                break;
            }
            if (this.cache.has(hash) || this.requests.has(hash) || this.failures.has(hash)) {
                continue;
            }
            const controller = new AbortController();
            this.requests.set(hash, controller);
            void this.loadTile(hash, controller.signal).then((image) => {
                if (!controller.signal.aborted && !this.destroyed) {
                    this.cache.set(hash, image);
                }
            }).catch((cause) => {
                if (controller.signal.aborted || this.destroyed) {
                    return;
                }
                const error = new Error(`Cannot load tile ${hash}: ${cause instanceof Error ? cause.message : String(cause)}`, { cause });
                this.failures.set(hash, error);
                console.error(error);
                this.dispatchEvent(new MapErrorEvent(error));
            }).finally(() => {
                this.requests.delete(hash);
                if (!this.destroyed) {
                    this.pump();
                    this.schedule();
                }
            });
        }
    }
    async loadTile(hash, signal) {
        const index = this.index;
        if (!index) {
            throw new Error("Map index is not loaded");
        }
        const url = new URL(`${hash}.mpt`, this.baseUrl);
        const response = await fetch(url, { signal });
        if (!response.ok) {
            throw new Error(`${url}: HTTP ${response.status}`);
        }
        const { colors } = await decodeTile(await response.arrayBuffer(), index.tileSize);
        signal.throwIfAborted();
        const image = document.createElement("canvas");
        image.width = index.tileSize;
        image.height = index.tileSize;
        const context = image.getContext("2d");
        if (!context) {
            throw new Error(`Cannot allocate a ${index.tileSize}x${index.tileSize} tile canvas`);
        }
        const pixels = context.createImageData(index.tileSize, index.tileSize);
        for (let i = 0; i < colors.length; i++) {
            const color = index.palette[colors[i]];
            pixels.data[i * 4] = (color >>> 16) & 255;
            pixels.data[i * 4 + 1] = (color >>> 8) & 255;
            pixels.data[i * 4 + 2] = color & 255;
            pixels.data[i * 4 + 3] = color >>> 24;
        }
        context.putImageData(pixels, 0, 0);
        return image;
    }
    draw() {
        const context = this.context;
        const index = this.index;
        const canvas = this.canvas;
        if (!context || !index || !canvas) {
            return;
        }
        context.setTransform(1, 0, 0, 1, 0, 0);
        context.clearRect(0, 0, canvas.width, canvas.height);
        context.setTransform(this.dpr, 0, 0, this.dpr, 0, 0);
        context.imageSmoothingEnabled = false;
        for (const tile of this.visible) {
            const x = (tile.x * index.tileSize - this.x) * this.scale + this.width / 2;
            const y = (tile.z * index.tileSize - this.y) * this.scale + this.height / 2;
            const side = index.tileSize * this.scale;
            const image = this.cache.get(tile.hash);
            if (image) {
                context.drawImage(image, x, y, side, side);
            }
            else if (this.failures.has(tile.hash)) {
                context.fillStyle = "#991b1b";
                context.fillRect(x, y, side, side);
                context.strokeStyle = "#fca5a5";
                context.lineWidth = 2;
                context.beginPath();
                context.moveTo(x, y);
                context.lineTo(x + side, y + side);
                context.moveTo(x + side, y);
                context.lineTo(x, y + side);
                context.stroke();
            }
        }
    }
    pointerDown = (event) => {
        if (event.button !== 0 || this.drag) {
            return;
        }
        this.drag = {
            pointerId: event.pointerId, clientX: event.clientX, clientY: event.clientY,
            x: this.x, y: this.y,
        };
        this.canvas?.setPointerCapture(event.pointerId);
    };
    pointerMove = (event) => {
        const drag = this.drag;
        if (!drag || event.pointerId !== drag.pointerId) {
            return;
        }
        this.setView({
            x: drag.x - (event.clientX - drag.clientX) / this.scale,
            y: drag.y - (event.clientY - drag.clientY) / this.scale,
        });
    };
    pointerUp = (event) => {
        if (this.drag?.pointerId !== event.pointerId) {
            return;
        }
        this.drag = null;
        if (this.canvas?.hasPointerCapture(event.pointerId)) {
            this.canvas.releasePointerCapture(event.pointerId);
        }
    };
    wheel = (event) => {
        if (!this.canvas || this.minScale === this.maxScale) {
            return;
        }
        event.preventDefault();
        const unit = event.deltaMode === 1 ? 16 : event.deltaMode === 2 ? this.height : 1;
        const scale = Math.min(this.maxScale, Math.max(this.minScale, this.scale * Math.exp(-event.deltaY * unit * 0.002)));
        const rect = this.canvas.getBoundingClientRect();
        const x = event.clientX - rect.left - this.width / 2;
        const y = event.clientY - rect.top - this.height / 2;
        this.setView({
            x: this.x + x / this.scale - x / scale,
            y: this.y + y / this.scale - y / scale,
        }, scale);
        if (this.drag) {
            this.drag = {
                pointerId: this.drag.pointerId, clientX: event.clientX, clientY: event.clientY,
                x: this.x, y: this.y,
            };
        }
    };
}
