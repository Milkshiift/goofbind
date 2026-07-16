import { Goofbind } from '../goofbind.js';

const keys = new Goofbind('./target/debug/goofbind');

keys.on('error', (err) => console.error('Error:', err));

keys.on('pressed', (id) => {
    console.log(`Shortcut Activated! ID: ${id}`);
});

keys.on('released', (id) => {
    console.log(`Shortcut Released! ID: ${id}`);
});

keys.setKeybinds([
    {
        id: 'mute',
        name: 'Mute Mic',
        keycode: 77, // M
        shift: true,
        alt: true
    },
    {
        id: 'deafen',
        keycode: 68, // D
        shift: true,
        ctrl: true
    }
]);