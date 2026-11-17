mod cartridge;
mod cpu;
mod ppu;

use cartridge::Cartridge;
use cpu::Cpu;
use minifb::{Key, Scale, Window, WindowOptions};
use std::env;
use std::fs;

// The state of the processor, laid out like a line of the nestest log.
fn trace_line(cpu: &Cpu) -> String {
    format!("{:04X} A:{:02X} X:{:02X}", cpu.pc, cpu.a, cpu.x)
}

// Run nestest step by step, comparing with a log from a known-good emulator.
fn check_against_log(cpu: &mut Cpu, path: &str) {
    let log = fs::read_to_string(path).expect("could not read the log file");
    cpu.pc = 0xC000;

    let mut count = 0;
    for line in log.lines() {
        count += 1;
        let expected = format!("{} {}", &line[0..4], &line[48..57]);
        let actual = trace_line(cpu);
        if actual != expected {
            println!("mismatch at line {}", count);
            println!("  expected {}", expected);
            println!("  got      {}", actual);
            return;
        }
        cpu.step();
        if cpu.stopped {
            println!("stopped at line {}", count);
            return;
        }
    }
    println!("all {} lines match", count);
}

fn main() {
    let path = env::args().nth(1).expect("usage: nes <file.nes> [nestest.log]");
    let cart = Cartridge::load(&path);
    let mut cpu = Cpu::new(cart);
    cpu.reset();

    if let Some(log) = env::args().nth(2) {
        check_against_log(&mut cpu, &log);
        return;
    }

    while !cpu.stopped {
        cpu.step();
    }
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
