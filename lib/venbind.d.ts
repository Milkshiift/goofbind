export class Venbind {
  startKeybinds(callback: (id: string, keyup: boolean) => void, app_id: string | null): void;
  registerKeybind(keybind: string, keybindId: string): void;
  unregisterKeybind(keybindId: string): void;
  preregisterKeybinds(actions: PreRegisterAction[]): void;
  defineErrorHandle(callback: (error: string) => void): void;
}
export interface PreRegisterAction {
  id: string
  name: string
}
