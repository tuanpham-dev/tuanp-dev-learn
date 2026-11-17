mod cartridge;
mod cpu;
mod ppu;

use cartridge::Cartridge;
use cpu::Cpu;
use minifb::{Key, Scale, Window, WindowOptions};
use std::env;

fn main() {
    let path = env::args().nth(1).expect("usage: nes <file.nes>");
    let cart = Cartridge::load(&path);

    let mut cpu = Cpu::new();
    cpu.load(&cart.prg);
    cpu.ppu.load_chr(&cart.chr);
    cpu.reset();
    cpu.run();

    cpu.ppu.draw();

    let mut options = WindowOptions::default();
    options.scale = Scale::X2;
    let mut window =
        Window::new("NES", ppu::WIDTH, ppu::HEIGHT, options).expect("could not open a window");
    window.set_target_fps(60);

    while window.is_open() && !window.is_key_down(Key::Escape) {
        window
            .update_with_buffer(&cpu.ppu.frame, ppu::WIDTH, ppu::HEIGHT)
            .expect("could not draw to the window");
    }
}
