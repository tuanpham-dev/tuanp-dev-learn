mod cartridge;
mod controller;
mod cpu;
mod ppu;

use cartridge::Cartridge;
use controller::*;
use cpu::Cpu;
use minifb::{Key, Scale, Window, WindowOptions};
use std::env;
use std::fs;

// Which keyboard key is which controller button.
const KEYS: [(Key, u8); 8] = [
    (Key::X, BUTTON_A),
    (Key::Z, BUTTON_B),
    (Key::RightShift, BUTTON_SELECT),
    (Key::Enter, BUTTON_START),
    (Key::Up, BUTTON_UP),
    (Key::Down, BUTTON_DOWN),
    (Key::Left, BUTTON_LEFT),
    (Key::Right, BUTTON_RIGHT),
];

// The state of the processor, laid out like a line of the nestest log.
fn trace_line(cpu: &Cpu) -> String {
    format!(
        "{:04X} A:{:02X} X:{:02X} Y:{:02X} P:{:02X} SP:{:02X} CYC:{}",
        cpu.pc, cpu.a, cpu.x, cpu.y, cpu.status(), cpu.sp, cpu.cycles
    )
}

// Run nestest step by step, comparing with a log from a known-good emulator.
fn check_against_log(cpu: &mut Cpu, path: &str) {
    let log = fs::read_to_string(path).expect("could not read the log file");
    cpu.pc = 0xC000;

    let mut count = 0;
    for line in log.lines() {
        count += 1;
        if line.as_bytes()[15] == b'*' {
            println!("line {}: the log reaches the unofficial opcodes; stopping here", count);
            println!("{} lines match", count - 1);
            return;
        }
        let cyc = line.find("CYC:").expect("no CYC in this log line");
        let expected = format!("{} {} {}", &line[0..4], &line[48..73], &line[cyc..]);
        let actual = trace_line(cpu);
        if actual != expected {
            println!("mismatch at line {}", count);
            println!("  expected {}", expected);
            println!("  got      {}", actual);
            return;
        }
        cpu.step();
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

    let mut options = WindowOptions::default();
    options.scale = Scale::X2;
    let mut window =
        Window::new("NES", ppu::WIDTH, ppu::HEIGHT, options).expect("could not open a window");
    window.set_target_fps(60);

    while window.is_open() && !window.is_key_down(Key::Escape) {
        let mut buttons = 0;
        for (key, button) in KEYS {
            if window.is_key_down(key) {
                buttons |= button;
            }
        }
        cpu.pad.buttons = buttons;

        cpu.run_frame();

        window
            .update_with_buffer(&cpu.ppu.frame, ppu::WIDTH, ppu::HEIGHT)
            .expect("could not draw to the window");
    }
}
