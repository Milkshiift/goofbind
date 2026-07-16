import { EventEmitter } from 'events';

export interface Keybind {
    /** Unique identifier for this keybind. */
    id: string;
    /** Optional human-readable name, used primarily for Wayland XDG desktop portals. */
    name?: string;
    /** The raw keycode to bind to. */
    keycode?: number;
    /** Whether the Control key must be held. Default: false. */
    ctrl?: boolean;
    /** Whether the Alt key must be held. Default: false. */
    alt?: boolean;
    /** Whether the Shift key must be held. Default: false. */
    shift?: boolean;
    /** Whether the Meta/Super/Windows key must be held. Default: false. */
    meta?: boolean;
}

export interface GoofbindEvents {
    pressed: (id: string) => void;
    released: (id: string) => void;
    error: (error: Error) => void;
}

export class Goofbind extends EventEmitter {
    /**
     * Spawns the Goofbind backend and opens IPC.
     *
     * @param binaryPath Path to the compiled Goofbind Rust binary.
     * @param appId Optional App ID (for Wayland XDG desktop portals).
     */
    constructor(binaryPath: string, appId?: string | null);

    /**
     * Updates the active keybinds.
     *
     * @param keybinds An array of keybind configurations.
     */
    setKeybinds(keybinds: Keybind[]): void;

    /**
     * Safely shuts down the Goofbind process and releases resources.
     * Should be called during your app's cleanup/exit phase.
     */
    destroy(): void;
    
    on<U extends keyof GoofbindEvents>(event: U, listener: GoofbindEvents[U]): this;
    once<U extends keyof GoofbindEvents>(event: U, listener: GoofbindEvents[U]): this;
    emit<U extends keyof GoofbindEvents>(event: U, ...args: Parameters<GoofbindEvents[U]>): boolean;
    off<U extends keyof GoofbindEvents>(event: U, listener: GoofbindEvents[U]): this;
    addListener<U extends keyof GoofbindEvents>(event: U, listener: GoofbindEvents[U]): this;
    removeListener<U extends keyof GoofbindEvents>(event: U, listener: GoofbindEvents[U]): this;
    removeAllListeners(event?: keyof GoofbindEvents): this;
}