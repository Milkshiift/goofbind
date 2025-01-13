export class Venbind {
    startKeybinds(callback: (x: number) => void): Promise<void>;
    registerKeybind(keybind: string, keybindId: number): void;
    unregisterKeybind(keybindId: number): void;
}
