mod chip8;
mod display;
mod keyboard;

use chip8::Cpu;
use display::{CUIDraw, Draw};
use getch_rs::{Getch, Key};
use keyboard::{GetchKeyboard, KeyboardInput};
use simplelog::*;
use std::fs::File;

fn start<T: KeyboardInput, D: Draw>(mut cpu: Cpu<T>, drawer: D) {
    loop {
        cpu.update();
        cpu.decrement_timers();
        drawer.draw(cpu.get_display());
    }
}

fn main() {
    CombinedLogger::init(vec![WriteLogger::new(
        LevelFilter::Debug,
        Config::default(),
        File::create("chip8.log").unwrap(),
    )])
    .unwrap();

    println!("Please select a game to play");
    println!("1. BRIX");
    println!("2. INVADERS");
    println!("3. GUESS");

    let g = Getch::new();
    let rom = match g.getch() {
        Ok(Key::Char('1')) => "rom/BRIX",
        Ok(Key::Char('2')) => "rom/INVADERS",
        Ok(Key::Char('3')) => "rom/GUESS",
        _ => {
            println!("Invalid option");
            std::process::exit(0)
        }
    };

    let keyboard = GetchKeyboard::new();
    let cpu = Cpu::new(rom, keyboard);

    let drawer = CUIDraw;
    start(cpu, drawer);
}
