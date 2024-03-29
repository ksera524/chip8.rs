use getch_rs::{Getch, Key};
use log::{info, warn};
use rand::Rng;
use simplelog::LevelFilter;
use simplelog::*;
use std::fs::File;
use std::io::Read;
use std::path::Path;
use std::thread;
use std::time::{Duration, Instant};

const DISPLAY_WIDTH: usize = 64;
const DISPLAY_HEIGHT: usize = 32;
const FONTSET: [u8; 80] = [
    0xF0, 0x90, 0x90, 0x90, 0xF0, // 0
    0x20, 0x60, 0x20, 0x20, 0x70, // 1
    0xF0, 0x10, 0xF0, 0x80, 0xF0, // 2
    0xF0, 0x10, 0xF0, 0x10, 0xF0, // 3
    0x90, 0x90, 0xF0, 0x10, 0x10, // 4
    0xF0, 0x80, 0xF0, 0x10, 0xF0, // 5
    0xF0, 0x80, 0xF0, 0x90, 0xF0, // 6
    0xF0, 0x10, 0x20, 0x40, 0x40, // 7
    0xF0, 0x90, 0xF0, 0x90, 0xF0, // 8
    0xF0, 0x90, 0xF0, 0x10, 0xF0, // 9
    0xF0, 0x90, 0xF0, 0x90, 0x90, // A
    0xE0, 0x90, 0xE0, 0x90, 0xE0, // B
    0xF0, 0x80, 0x80, 0x80, 0xF0, // C
    0xE0, 0x90, 0x90, 0x90, 0xE0, // D
    0xF0, 0x80, 0xF0, 0x80, 0xF0, // E
    0xF0, 0x80, 0xF0, 0x80, 0x80, // F
];

struct Cpu {
    registers: [u8; 16],
    position_in_memory: usize, //program counter
    memory: [u8; 0x1000],
    stack: [u16; 16],
    stack_pointer: usize,
    index_register: u16,
    delay_timer: u8,
    display: [[bool; DISPLAY_WIDTH]; DISPLAY_HEIGHT],
    sound_timer: u8,
    key: [bool; 16],
}

impl Cpu {
    fn new(file_path: &str) -> Cpu {
        let mut cpu = Cpu {
            registers: [0; 16],
            position_in_memory: 0x200,
            memory: [0; 0x1000],
            stack: [0; 16],
            stack_pointer: 0,
            index_register: 0,
            delay_timer: 0,
            sound_timer: 0,
            display: [[false; DISPLAY_WIDTH]; DISPLAY_HEIGHT],
            key: [false; 16],
        };

        let mut file = File::open(Path::new(file_path)).expect("Failed to open the file");
        let mut buffer: Vec<u8> = Vec::new();
        file.read_to_end(&mut buffer)
            .expect("Failed to read the file");

        for (i, byte) in FONTSET.iter().enumerate() {
            cpu.memory[i] = *byte;
        }

        for (i, byte) in buffer.iter().enumerate() {
            cpu.memory[0x200 + i] = *byte;
        }

        cpu
    }

    fn read_opcode(&self) -> u16 {
        let p = self.position_in_memory;
        let op_byte_1 = self.memory[p] as u16;
        let op_byte_2 = self.memory[p + 1] as u16;
        op_byte_1 << 8 | op_byte_2
    }

    fn read_keyboard_input(&mut self) {
        info!("Executing function: read_keyboard_input");
        let g = Getch::new();
        match g.getch() {
            Ok(Key::Char('1')) => {
                self.key[0x1] = true;
            }
            Ok(Key::Char('2')) => {
                self.key[0x2] = true;
            }
            Ok(Key::Char('3')) => {
                self.key[0x3] = true;
            }
            Ok(Key::Char('4')) => {
                self.key[0xC] = true;
            }
            Ok(Key::Char('q')) => {
                self.key[0x4] = true;
            }
            Ok(Key::Char('w')) => {
                self.key[0x5] = true;
            }
            Ok(Key::Char('e')) => {
                self.key[0x6] = true;
            }
            Ok(Key::Char('r')) => {
                self.key[0xD] = true;
            }
            Ok(Key::Char('a')) => {
                self.key[0x7] = true;
            }
            Ok(Key::Char('s')) => {
                self.key[0x8] = true;
            }
            Ok(Key::Char('d')) => {
                self.key[0x9] = true;
            }
            Ok(Key::Char('f')) => {
                self.key[0xE] = true;
            }
            Ok(Key::Char('z')) => {
                self.key[0xA] = true;
            }
            Ok(Key::Char('x')) => {
                self.key[0x0] = true;
            }
            Ok(Key::Char('c')) => {
                self.key[0xB] = true;
            }
            Ok(Key::Char('v')) => {
                self.key[0xF] = true;
            }
            Ok(Key::Esc) => {
                std::process::exit(0);
            }
            _ => {}
        }
    }

    fn render_display(&self) {
        println!("\x1b[2J\x1b[H\x1b[?25l");
        println!("\x1b[H");
        for row in &self.display {
            for &pixel in row {
                if pixel {
                    print!("#");
                } else {
                    print!(" ");
                }
            }
            println!();
        }
    }

    fn decrement_timers(&mut self) {
        if self.delay_timer > 0 {
            self.delay_timer -= 1;
        }

        if self.sound_timer > 0 {
            self.sound_timer -= 1;
        }
    }

    fn run(&mut self) {
        info!(
            "v={:?} i={}({:x}) stack={:?} sp={:x} pc={}({:x}) dt={:x}",
            self.registers,
            self.index_register,
            self.index_register,
            self.stack,
            self.stack_pointer,
            self.position_in_memory,
            self.position_in_memory,
            self.delay_timer
        );

        self.render_display();
        let opcode = self.read_opcode();

        self.position_in_memory += 2;

        let c = ((opcode & 0xF000) >> 12) as u8;
        let x = ((opcode & 0x0F00) >> 8) as u8;
        let y = ((opcode & 0x00F0) >> 4) as u8;
        let d = ((opcode & 0x000F) >> 0) as u8;

        let nnn = opcode & 0x0FFF;
        let kk: u8 = (opcode & 0x00FF) as u8;

        match (c, x, y, d) {
            (0, 0, 0xE, 0) => {
                self.cls();
            }
            (0, 0, 0xE, 0xE) => {
                self.ret();
            }
            (0, _, _, _) => {
                self.sys_addr(nnn);
            }
            (0x1, _, _, _) => {
                self.jp_addr(nnn);
            }
            (0x2, _, _, _) => {
                self.call(nnn);
            }
            (0x3, _, _, _) => {
                self.se_byte(x, kk);
            }
            (0x4, _, _, _) => {
                self.sne_byte(x, kk);
            }
            (0x5, _, _, 0) => {
                self.se_xy(x, y);
            }
            (0x6, _, _, _) => {
                self.ld_byte(x, kk);
            }
            (0x7, _, _, _) => {
                self.add_byte(x, kk);
            }
            (0x8, _, _, 0) => {
                self.ld_xy(x, y);
            }
            (0x8, _, _, 1) => {
                self.or_xy(x, y);
            }
            (0x8, _, _, 2) => {
                self.and_xy(x, y);
            }
            (0x8, _, _, 3) => {
                self.xor_xy(x, y);
            }
            (0x8, _, _, 4) => {
                self.add_xy(x, y);
            }
            (0x8, _, _, 5) => {
                self.sub_xy(x, y);
            }
            (0x8, _, _, 6) => {
                self.shr_xy(x);
            }
            (0x8, _, _, 7) => {
                self.subn_xy(x, y);
            }
            (0x8, _, _, 0xE) => {
                self.shl_xy(x);
            }
            (0x9, _, _, 0) => {
                self.sne_xy(x, y);
            }
            (0xA, _, _, _) => {
                self.ld_i_addr(nnn);
            }
            (0xB, _, _, _) => {
                self.jp_v0_addr(nnn);
            }
            (0xC, _, _, _) => {
                self.rnd_byte(x, kk);
            }
            (0xD, _, _, _) => {
                self.drw_xy(x, y, d);
            }
            (0xE, _, 9, 0xE) => {
                self.skp_vx(x);
            }
            (0xE, _, 0xA, 1) => {
                self.sknp_vx(x);
            }
            (0xF, _, 0, 7) => {
                self.ld_vx_dt(x);
            }
            (0xF, _, 0, 0xA) => {
                self.ld_vx_k(x);
            }
            (0xF, _, 1, 5) => {
                self.ld_dt_vx(x);
            }
            (0xF, _, 1, 8) => {
                self.ld_st_vx(x);
            }
            (0xF, _, 1, 0xE) => {
                self.add_i_vx(x);
            }
            (0xF, _, 2, 9) => {
                self.ld_f_vx(x);
            }
            (0xF, _, 3, 3) => {
                self.ld_b_vx(x);
            }
            (0xF, _, 5, 5) => {
                self.ld_i_vx(x);
            }
            (0xF, _, 6, 5) => {
                self.ld_vx_i(x);
            }
            _ => {
                warn!("Executing unknown opcode");
                todo!("opcode {:04x}", opcode);
            }
        }
        println!("\x1b[?25h");
    }

    fn sys_addr(&mut self, nnn: u16) {
        info!("Executing function: sys_addr");
        self.position_in_memory = nnn as usize;
    }

    fn cls(&mut self) {
        info!("Executing function: cls");
        self.display = [[false; DISPLAY_WIDTH]; DISPLAY_HEIGHT];
    }

    fn ret(&mut self) {
        info!("Executing function: ret");
        if self.stack_pointer == 0 {
            panic!("Stack underflow")
        }

        self.stack_pointer -= 1;
        self.position_in_memory = self.stack[self.stack_pointer] as usize;
    }

    fn jp_addr(&mut self, nnn: u16) {
        info!("Executing function: jp_addr :{}", nnn);
        self.position_in_memory = nnn as usize;
    }

    fn call(&mut self, nnn: u16) {
        info!("Executing function: call nnn: {}", nnn);
        let sp = self.stack_pointer;
        let stack = &mut self.stack;

        if sp > stack.len() {
            panic!("Stack overflow")
        }

        stack[sp] = self.position_in_memory as u16;
        self.stack_pointer += 1;
        self.position_in_memory = nnn as usize;
    }

    fn se_byte(&mut self, x: u8, kk: u8) {
        let vx = self.registers[x as usize];
        info!("Executing function: se_byte x: {} vx: {} kk: {}", x, vx, kk);
        if vx == kk {
            self.position_in_memory += 2;
        }
    }

    fn sne_byte(&mut self, x: u8, kk: u8) {
        info!("Executing function: sne_byte");
        if self.registers[x as usize] != kk {
            self.position_in_memory += 2;
        }
    }

    fn se_xy(&mut self, x: u8, y: u8) {
        info!("Executing function: se_xy");
        let vx = self.registers[x as usize];
        let vy = self.registers[y as usize];

        if vx == vy {
            self.position_in_memory += 2;
        }
    }

    fn ld_byte(&mut self, x: u8, kk: u8) {
        info!("Executing function: ld_byte");
        self.registers[x as usize] = kk;
    }

    fn add_byte(&mut self, x: u8, kk: u8) {
        info!("Executing function: add_byte");
        let vx = self.registers[x as usize];
        self.registers[x as usize] = vx.overflowing_add(kk).0;
    }

    fn ld_xy(&mut self, x: u8, y: u8) {
        info!("Executing function: ld_xy");
        self.registers[x as usize] = self.registers[y as usize];
    }

    fn or_xy(&mut self, x: u8, y: u8) {
        info!("Executing function: or_xy");
        self.registers[x as usize] |= self.registers[y as usize];
    }

    fn and_xy(&mut self, x: u8, y: u8) {
        info!("Executing function: and_xy");
        self.registers[x as usize] &= self.registers[y as usize];
    }

    fn xor_xy(&mut self, x: u8, y: u8) {
        info!("Executing function: xor_xy");
        self.registers[x as usize] ^= self.registers[y as usize];
    }

    fn add_xy(&mut self, x: u8, y: u8) {
        info!("Executing function: add_xy");
        let vx = self.registers[x as usize];
        let vy = self.registers[y as usize];

        let (val, overflow) = vx.overflowing_add(vy);
        self.registers[x as usize] = val;

        if overflow {
            self.registers[0xF] = 1;
        } else {
            self.registers[0xF] = 0;
        }
    }

    fn sub_xy(&mut self, x: u8, y: u8) {
        info!("Executing function: sub_xy");
        let vx = self.registers[x as usize];
        let vy = self.registers[y as usize];

        let (val, overflow) = vx.overflowing_sub(vy);
        self.registers[x as usize] = val;

        if !overflow {
            self.registers[0xF] = 1;
        } else {
            self.registers[0xF] = 0;
        }
    }

    fn shr_xy(&mut self, x: u8) {
        info!("Executing function: shr_xy");
        let vx = self.registers[x as usize];

        self.registers[0xF] = vx & 0x01;
        self.registers[x as usize] /= 2;
    }

    fn subn_xy(&mut self, x: u8, y: u8) {
        info!("Executing function: subn_xy");
        let vx = self.registers[x as usize];
        let vy = self.registers[y as usize];

        let (val, overflow) = vy.overflowing_sub(vx);
        self.registers[x as usize] = val;
        if !overflow {
            self.registers[0xF] = 1;
        } else {
            self.registers[0xF] = 0;
        }
    }

    fn shl_xy(&mut self, x: u8) {
        info!("Executing function: shl_xy");
        let vx = self.registers[x as usize];
        let (val, overflow) = vx.overflowing_shl(1);
        self.registers[x as usize] = val;

        if overflow {
            self.registers[0xF] = 1;
        } else {
            self.registers[0xF] = 0;
        }
    }

    fn sne_xy(&mut self, x: u8, y: u8) {
        info!("Executing function: sne_xy");
        if self.registers[x as usize] != self.registers[y as usize] {
            self.position_in_memory += 2;
        }
    }

    fn ld_i_addr(&mut self, nnn: u16) {
        info!("Executing function: ld_i_addr");
        self.index_register = nnn;
    }

    fn jp_v0_addr(&mut self, nnn: u16) {
        info!("Executing function: jp_v0_addr");
        self.position_in_memory = (self.registers[0] as u16 + nnn) as usize;
    }

    fn rnd_byte(&mut self, x: u8, kk: u8) {
        info!("Executing function: rnd_byte");
        let mut rng = rand::thread_rng();
        let random_number: u8 = rng.gen();
        self.registers[x as usize] = random_number & kk;
    }

    fn drw_xy(&mut self, x: u8, y: u8, n: u8) {
        info!("Executing function: drw_xy");
        let vx = self.registers[x as usize] as usize;
        let vy = self.registers[y as usize] as usize;

        self.registers[0xF] = 0;

        for byte_offset in 0..n {
            let byte = self.memory[self.index_register as usize + byte_offset as usize];
            for bit_offset in 0..8 {
                let bit = (byte >> (7 - bit_offset)) & 1;
                let curr_x = (vx + bit_offset) % DISPLAY_WIDTH;
                let curr_y = (vy + byte_offset as usize) % DISPLAY_HEIGHT;
                let prev = self.display[curr_y][curr_x];
                self.display[curr_y][curr_x] ^= bit == 1;
                if prev && !self.display[curr_y][curr_x] {
                    self.registers[0xF] = 1;
                }
            }
        }
    }

    fn skp_vx(&mut self, x: u8) {
        let vx = self.registers[x as usize];
        info!("Executing function: skp_vx x: {} vx: {}", x, vx);
        if self.key[vx as usize] {
            self.position_in_memory += 2;
            self.key = [false; 16];
        }
    }

    fn sknp_vx(&mut self, x: u8) {
        let vx = self.registers[x as usize];
        info!("Executing function: sknp_vx x: {} vx: {}", x, vx);
        if !self.key[vx as usize] {
            self.position_in_memory += 2;
            self.key = [false; 16];
        }
    }

    fn ld_vx_dt(&mut self, x: u8) {
        let vx = self.registers[x as usize];
        info!("Executing function: ld_vx_dt x: {} vx: {}", x, vx);
        self.registers[x as usize] = self.delay_timer;
    }

    fn ld_vx_k(&mut self, x: u8) {
        info!("Executing function: ld_vx_k");
        self.read_keyboard_input();
        for i in 0..16 {
            if self.key[i] {
                self.registers[x as usize] = i as u8;
                return;
            }
        }
        self.key[self.registers[x as usize] as usize] = false;


        self.read_keyboard_input();
        for i in 0..16 {
            if self.key[i] {
                self.registers[x as usize] = i as u8;
                return;
            }
        }
        self.key[self.registers[x as usize] as usize] = false;
    }

    fn ld_dt_vx(&mut self, x: u8) {
        info!("Executing function: ld_dt_vx");
        self.delay_timer = self.registers[x as usize];
    }

    fn ld_st_vx(&mut self, x: u8) {
        info!("Executing function: ld_st_vx");
        self.sound_timer = self.registers[x as usize];
    }

    fn add_i_vx(&mut self, x: u8) {
        info!("Executing function: add_i_vx");
        self.index_register += self.registers[x as usize] as u16;
    }

    fn ld_f_vx(&mut self, x: u8) {
        info!("Executing function: ld_f_vx");
        self.index_register = self.registers[x as usize] as u16 * 5;
    }

    fn ld_b_vx(&mut self, x: u8) {
        info!("Executing function: ld_b_vx");
        let vx = self.registers[x as usize];
        self.memory[self.index_register as usize] = (vx / 100) % 10;
        self.memory[self.index_register as usize + 1] = (vx / 10) % 10;
        self.memory[self.index_register as usize + 2] = vx % 10;
    }

    fn ld_i_vx(&mut self, x: u8) {
        info!("Executing function: ld_i_vx");
        for i in 0..=x {
            self.memory[self.index_register as usize + i as usize] = self.registers[i as usize];
        }
    }

    fn ld_vx_i(&mut self, x: u8) {
        info!("Executing function: ld_vx_i");
        for i in 0..=x {
            self.registers[i as usize] = self.memory[self.index_register as usize + i as usize];
        }
    }
}

fn main() {
    CombinedLogger::init(vec![
        TermLogger::new(
            LevelFilter::Warn,
            Config::default(),
            TerminalMode::Mixed,
            ColorChoice::Auto,
        ),
        WriteLogger::new(
            LevelFilter::Info,
            Config::default(),
            File::create("chip8.log").unwrap(),
        ),
    ])
    .unwrap();

    info!("Starting the emulator");
    let mut cpu = Cpu::new("rom/tetris.ch8");

    loop {
        let start = Instant::now();
        cpu.run();
        cpu.decrement_timers();
        let duration = start.elapsed();
        let target_duration = Duration::from_secs_f64(1.0 / 60.0);
        if duration < target_duration {
            thread::sleep(target_duration - duration);
        }
    }
}
