# CHIP-8 Emulator on the browser with WASM

## Features

### Debugger View

### Stepping through code

### Variable speed

Yes it can even run at 1e+308 if your computer can handle it

### Sound 🔊

### All the Chip 8 games you would need

All graciously taken from the [Chip 8 Archive](https://johnearnest.github.io/chip8Archive)

## Compile it yourself

1. Install [wasm-pack](https://github.com/rustwasm/wasm-pack)
2. Run `./build.sh`
3. Serve the `web/index.html` from a web server and enjoy!

## Thanks

This couldn't have been possible without these amazing websites.

- The guide that started it all: [Guide to making a CHIP-8 emulator](https://tobiasvl.github.io/blog/write-a-chip-8-emulator/)
- The concrete guide that is great for writting better code: [Building a CHIP-8 Emulator [C++]](https://austinmorlan.com/posts/chip8_emulator/)
- Quick ref: [CHIP-8 Instruction Set](https://johnearnest.github.io/Octo/docs/chip8ref.pdf)
- The archive from the amazing John: [Chip-8 Archive](https://johnearnest.github.io/chip8Archive/)
- The very cool assembler: [Octo](https://johnearnest.github.io/Octo/)
- All the other awesome stuff: [Awesome Chip-8](https://github.com/tobiasvl/awesome-chip-8)

## TODO

- save breakpoints and actually stop on them
