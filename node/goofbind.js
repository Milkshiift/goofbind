import { spawn } from 'child_process';
import { createInterface } from 'readline';
import { EventEmitter } from 'events';

export class Goofbind extends EventEmitter {
    #child;
    #lines;
    #destroyed = false;

    /**
     * @param {string} binaryPath - Path to the compiled goofbind Rust binary
     * @param {string} [appId] - Optional App ID (for XDG desktop portals)
     */
    constructor(binaryPath, appId = null) {
        super();

        const args = appId ? [appId] : [];

        this.#child = spawn(binaryPath, args, {
            stdio: ['pipe', 'pipe', 'inherit'], // stdin, stdout, stderr
            windowsHide: true,                   // Prevents a command prompt flash on Windows
        });

        if (!this.#child.stdin || !this.#child.stdout) {
            throw new Error('Goofbind failed to start with piped stdin/stdout');
        }

        // Prevent unhandled EPIPE errors if the child dies while Node is writing
        this.#child.stdin.on('error', (err) => {
            if (err.code !== 'EPIPE' && !this.#destroyed) {
                this.emit('error', err);
            }
        });

        this.#lines = createInterface({
            input: this.#child.stdout,
            crlfDelay: Infinity,
        });

        this.#lines.on('line', (line) => {
            this.#handleIncomingIPC(line);
        });

        this.#child.on('error', (error) => {
            if (!this.#destroyed) {
                this.emit('error', new Error(`Goofbind process error: ${error.message}`));
            }
        });

        this.#child.on('exit', (code, signal) => {
            this.#lines.close();

            if (!this.#destroyed) {
                const reason = signal ?? code ?? 'unknown';
                this.emit('error', new Error(`Goofbind unexpectedly exited (${reason})`));
            }
        });
    }

    #handleIncomingIPC(line) {
        if (!line.trim()) return;

        try {
            const data = JSON.parse(line);

            switch (data.event) {
                case 'pressed':
                    this.emit('pressed', data.id);
                    break;
                case 'released':
                    this.emit('released', data.id);
                    break;
                case 'error':
                    this.emit('error', new Error(data.message));
                    break;
                default:
                    console.warn('Unknown event from Goofbind:', data.event);
            }
        } catch (err) {
            console.error('Failed to parse IPC message from Goofbind:', line);
        }
    }

    /**
     * @param {Array<{id: string, name?: string, keycode?: number, ctrl?: boolean, alt?: boolean, shift?: boolean, meta?: boolean}>} keybinds
     */
    setKeybinds(keybinds) {
        // Ensure we don't write to a destroyed process or a closed stream
        if (this.#destroyed || !this.#child.stdin?.writable) return;

        const payload = JSON.stringify({
            command: 'set_keybinds',
            keybinds: keybinds,
        });

        this.#child.stdin.write(payload + '\n');
    }

    destroy() {
        if (this.#destroyed) return;
        this.#destroyed = true;

        if (this.#lines) {
            this.#lines.close();
        }

        if (this.#child) {
            // Ending stdin triggers EOF in the Rust stdin.lines() loop, forcing a clean Rust exit
            if (this.#child.stdin && !this.#child.stdin.destroyed) {
                this.#child.stdin.end();
            }

            // Only attempt a forceful kill if it hasn't exited yet
            if (this.#child.exitCode === null && this.#child.signalCode === null) {
                this.#child.kill();
            }
        }
    }
}