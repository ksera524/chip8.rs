mod chip8;
mod display;
mod keyboard;

use chip8::Cpu;
use display::{CUIDraw, Draw};
use getch_rs::{Getch, Key};
use keyboard::{GetchKeyboard, KeyboardInput};
use simplelog::*;
use std::fs::File;
use std::time::{Duration, Instant};

fn start<T: KeyboardInput, D: Draw>(mut cpu: Cpu<T>, drawer: D) {
    const CPU_FREQUENCY: u64 = 600; // 600命令/秒
    const TIMER_FREQUENCY: u64 = 60; // 60Hz固定
    
    let cpu_interval = Duration::from_nanos(1_000_000_000 / CPU_FREQUENCY);
    let timer_interval = Duration::from_nanos(1_000_000_000 / TIMER_FREQUENCY);
    
    let mut last_cpu_time = Instant::now();
    let mut last_timer_time = Instant::now();
    
    loop {
        let now = Instant::now();
        
        // CPU命令実行（600Hz）
        if now.duration_since(last_cpu_time) >= cpu_interval {
            cpu.update();
            drawer.draw(cpu.get_display());
            last_cpu_time = now;
        }
        
        // タイマー減算（60Hz）
        if now.duration_since(last_timer_time) >= timer_interval {
            cpu.decrement_timers();
            last_timer_time = now;
        }
        
        // CPU使用率を下げるため短時間スリープ
        std::thread::sleep(Duration::from_micros(100));
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
