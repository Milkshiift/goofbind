export class Venbind {
  startKeybinds(callback: (id: string, keyup: boolean) => void, app_id: string | null): void;
  setKeybinds(keybinds: KeybindInfo[]): void;
  defineErrorHandle(callback: (error: string) => void): void;
  getCurrentShortcut(): string;
}
export interface KeybindInfo {
  id: string
  name?: string
  shortcut?: string
}
