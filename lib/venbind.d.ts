export class Venbind {
  startKeybinds(callback: (err: null | Error, id: number) => void): void;
  registerKeybind(keybind: string, keybindId: number): void;
  unregisterKeybind(keybindId: number): void;
  preregisterKeybinds(actions: PreRegisterAction[]): void;
}
export interface PreRegisterAction {
  id: number
  name: string
}
