# Rust by building a NES emulator — course code

This repository holds the code for the course "Rust by building a NES emulator". You learn Rust by writing a working NES emulator from scratch, one small step at a time. Each `code/part-NN` folder is a complete, runnable snapshot of the project at the end of that part, so you can compare your own work or jump in anywhere.

Read the course: https://tuanp.dev/learn/rust-nes-emulator/

## Folders

| Folder | Contents |
| --- | --- |
| `code/part-01` … `code/part-20` | Project snapshots, one per course part |
| `code/roms` | Cartridges (`.nes`) and their assembly sources (`.s`) |
| `code/hello.nes` | The first cartridge, as made by `mkrom` (same file as `code/roms/hello.nes`) |
| `code/nesasm` | The small assembler used to build the cartridges |
| `code/mkrom` | Tool that makes `hello.nes` |

The paths match the course: where a part says `code/part-01/src/main.rs`, that is `code/part-01/src/main.rs` here.

## Running a snapshot

```
cd code/part-NN
cargo run
```

From part 10 on, pass a cartridge:

```
cd code/part-10
cargo run -- ../roms/hello.nes
```

## License

MIT, see `../LICENSE`, for everything except `code/roms/nestest.nes` and `code/roms/nestest.log`. Those are the nestest CPU test by kevtris, credited to its author and not covered by the MIT licence.
