# goofbind

A fork of [venbind](https://github.com/tuxinal/venbind) tailored for usage in [GoofCord](https://github.com/Milkshiift/GoofCord)

An all-in-one library made to handle shortcuts globally across multiple operating systems and desktops.

## Compiling

This project uses bindgen, which requires [libclang/LLVM](https://rust-lang.github.io/rust-bindgen/requirements.html). [Node](https://nodejs.org) is also required to build the project using napi-rs.

```sh
git clone --recurse-submodules https://github.com/Milkshiift/goofbind.git
cd goofbind

# if you cloned without submodules
git submodule update --init --recursive

# build
cargo build
```

## list of features / TODO

- [x] support linux x11
- [x] support being called through Node API
- [x] support linux wayland
- [x] support windows
- [ ] support macos
- [ ] better error handling
