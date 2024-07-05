use getch_rs::{Getch, Key};
use log::{info, warn};
use rand::prelude::*;
use simplelog::LevelFilter;
use simplelog::*;
use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;
use std::sync::mpsc;
use std::sync::{Arc, Mutex};
use std::thread;

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
    display: [[bool; DISPLAY_WIDTH]; DISPLAY_HEIGHT],
    sound_timer: u8,
    delay_timer: Arc<Mutex<u8>>,
    key: Option<u8>,
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
            delay_timer: Arc::new(Mutex::new(0)),
            sound_timer: 0,
            display: [[false; DISPLAY_WIDTH]; DISPLAY_HEIGHT],
            key: None,
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

    fn key(&mut self, receiver: &mut mpsc::Receiver<u8>) -> Option<u8> {
        receiver.try_recv().ok().or(self.key).map(|k| {
            self.key = Some(k);
            k
        })
    }

    fn draw(&self) {
        // カーソルを非表示にし、画面の一番上に移動
        print!("\x1b[?25l\x1b[H");

        // 描画用のバッファを準備
        let mut buffer = String::with_capacity(DISPLAY_HEIGHT * (DISPLAY_WIDTH + 1));

        for row in &self.display {
            for &pixel in row {
                buffer.push(if pixel { '█' } else { ' ' });
            }
            buffer.push('\n');
        }

        // バッファの内容を一度に出力
        print!("{}", buffer);

        // カーソルを表示し、画面をフラッシュ
        print!("\x1b[?25h");
        std::io::stdout().flush().unwrap();
    }

    fn decrement_timers(&mut self) {
        let mut delay_timer = self.delay_timer.lock().unwrap();
        if *delay_timer > 0 {
            *delay_timer -= 1;
        }

        if self.sound_timer > 0 {
            self.sound_timer -= 1;
        }
    }

    fn update(&mut self, keyboard_receiver: &mut mpsc::Receiver<u8>) {
        let dt = *self.delay_timer.lock().unwrap();
        info!(
            "v={:?} i={}({:x}) stack={:?} sp={:x} pc={}({:x}) dt={:x} key={:?}",
            self.registers,
            self.index_register,
            self.index_register,
            self.stack,
            self.stack_pointer,
            self.position_in_memory,
            self.position_in_memory,
            dt,
            self.key
        );

        let opcode = self.read_opcode();

        self.position_in_memory += 2;

        let c = ((opcode & 0xF000) >> 12) as u8;
        let x = ((opcode & 0x0F00) >> 8) as u8;
        let y = ((opcode & 0x00F0) >> 4) as u8;
        let d = ((opcode & 0x000F) >> 0) as u8;

        let nnn = opcode & 0x0FFF;
        let kk: u8 = (opcode & 0x00FF) as u8;

        //opcode
        info!("Executing opcode: {:04x}", opcode);

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
                self.skp_vx(x, keyboard_receiver);
            }
            (0xE, _, 0xA, 1) => {
                self.sknp_vx(x, keyboard_receiver);
            }
            (0xF, _, 0, 7) => {
                self.ld_vx_dt(x);
            }
            (0xF, _, 0, 0xA) => {
                self.ld_vx_k(x, keyboard_receiver);
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
        let vx = self.registers[x as usize] as u16;
        let vy = self.registers[y as usize] as u16;

        if vx + vy > 0xFF {
            self.registers[0xF] = 1;
        } else {
            self.registers[0xF] = 0;
        }
        self.registers[x as usize] = ((vx + vy) & 0xFF) as u8;
    }

    fn sub_xy(&mut self, x: u8, y: u8) {
        info!("Executing function: sub_xy");
        let vx = self.registers[x as usize];
        let vy = self.registers[y as usize];

        let (val, overflow) = vx.overflowing_sub(vy);
        if !overflow {
            self.registers[0xF] = 1;
        } else {
            self.registers[0xF] = 0;
        }
        self.registers[x as usize] = val;
    }

    fn shr_xy(&mut self, x: u8) {
        info!("Executing function: shr_xy");
        let vx = self.registers[x as usize];
        // 最下位ビットをVfにセット
        self.registers[0xF] = vx & 0x01;
        // 右シフト（2で割る）
        self.registers[x as usize] = vx >> 1;
    }

    fn subn_xy(&mut self, x: u8, y: u8) {
        info!("Executing function: subn_xy");
        let vx = self.registers[x as usize];
        let vy = self.registers[y as usize];

        let (val, overflow) = vy.overflowing_sub(vx);
        if !overflow {
            self.registers[0xF] = 1;
        } else {
            self.registers[0xF] = 0;
        }
        self.registers[x as usize] = val;
    }

    fn shl_xy(&mut self, x: u8) {
        info!("Executing function: shl_xy");
        let vx = self.registers[x as usize];
        // 最上位ビット（7番目のビット）をVfにセット
        self.registers[0xF] = (vx & 0x80) >> 7;
        // 左シフト（2倍）
        self.registers[x as usize] = vx.overflowing_mul(2).0;
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
        let random_number: u8 = random();
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

    fn skp_vx(&mut self, x: u8, keyboard_receiver: &mut mpsc::Receiver<u8>) {
        let vx = self.registers[x as usize];
        if let Some(key) = self.key(keyboard_receiver) {
            if key == vx {
                self.position_in_memory += 2;
                self.key = None;
            }
        }
    }

    fn sknp_vx(&mut self, x: u8, keyboard_receiver: &mut mpsc::Receiver<u8>) {
        let vx = self.registers[x as usize];
        info!("Executing function: sknp_vx x: {} vx: {}", x, vx);
        if let Some(key) = self.key(keyboard_receiver) {
            if key != vx {
                self.position_in_memory += 2;
                self.key = None;
            }
        }
    }

    fn ld_vx_dt(&mut self, x: u8) {
        let vx = self.registers[x as usize];
        let delay_timer = self.delay_timer.lock().unwrap();
        info!("Executing function: ld_vx_dt x: {} vx: {}", x, vx);
        self.registers[x as usize] = *delay_timer
    }

    fn ld_vx_k(&mut self, x: u8, keyboard_receiver: &mut mpsc::Receiver<u8>) {
        info!("Executing function: ld_vx_k");
        let mut pressed = false;

        if let Some(key) = self.key(keyboard_receiver) {
            self.registers[x as usize] = key;
            pressed = true;
        }

        if !pressed {
            self.position_in_memory -= 2; //キーが押されるまで待つ
        }
    }

    fn ld_dt_vx(&mut self, x: u8) {
        info!(
            "Executing function: ld_dt_vx x: {} vx: {}",
            x, self.registers[x as usize]
        );
        let mut delay_timer = self.delay_timer.lock().unwrap();
        *delay_timer = self.registers[x as usize];
    }

    fn ld_st_vx(&mut self, x: u8) {
        info!("Executing function: ld_st_vx");
        self.sound_timer = self.registers[x as usize];
    }

    fn add_i_vx(&mut self, x: u8) {
        info!("Executing function: add_i_vx");
        let vx = self.registers[x as usize];
        self.index_register += vx as u16;
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

    fn start(&mut self) {
        let (keyboard_sender, mut keyboard_receiver) = mpsc::channel();

        thread::spawn(move || {
            let g = Getch::new();
            loop {
                match g.getch() {
                    Ok(Key::Char('1')) => keyboard_sender.send(0x1).unwrap(),
                    Ok(Key::Char('2')) => keyboard_sender.send(0x2).unwrap(),
                    Ok(Key::Char('3')) => keyboard_sender.send(0x3).unwrap(),
                    Ok(Key::Char('4')) => keyboard_sender.send(0xC).unwrap(),
                    Ok(Key::Char('q')) => keyboard_sender.send(0x4).unwrap(),
                    Ok(Key::Char('w')) => keyboard_sender.send(0x5).unwrap(),
                    Ok(Key::Char('e')) => keyboard_sender.send(0x6).unwrap(),
                    Ok(Key::Char('r')) => keyboard_sender.send(0xD).unwrap(),
                    Ok(Key::Char('a')) => keyboard_sender.send(0x7).unwrap(),
                    Ok(Key::Char('s')) => keyboard_sender.send(0x8).unwrap(),
                    Ok(Key::Char('d')) => keyboard_sender.send(0x9).unwrap(),
                    Ok(Key::Char('f')) => keyboard_sender.send(0xE).unwrap(),
                    Ok(Key::Char('z')) => keyboard_sender.send(0xA).unwrap(),
                    Ok(Key::Char('x')) => keyboard_sender.send(0x0).unwrap(),
                    Ok(Key::Char('c')) => keyboard_sender.send(0xB).unwrap(),
                    Ok(Key::Char('v')) => keyboard_sender.send(0xF).unwrap(),
                    Ok(Key::Esc) => std::process::exit(0),
                    _ => {}
                }
            }
        });

        loop {
            self.update(&mut keyboard_receiver);
            self.draw();
            self.decrement_timers();
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

    cpu.start();
}
