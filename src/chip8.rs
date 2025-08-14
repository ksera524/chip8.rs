use log::{debug, warn};
use rand::random;
use std::{fs::File, io::Read, path::Path};

use crate::keyboard::KeyboardInput;

pub const DISPLAY_WIDTH: usize = 64;
pub const DISPLAY_HEIGHT: usize = 32;
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

pub struct Cpu<T: KeyboardInput> {
    registers: [u8; 16],
    program_counter: usize,
    memory: [u8; 0x1000],
    stack: [u16; 16],
    stack_pointer: usize,
    index_register: u16,
    display: [[bool; DISPLAY_WIDTH]; DISPLAY_HEIGHT],
    sound_timer: u8, //使わない
    delay_timer: u8,
    key: Option<u8>,
    keyboard: T,
}

impl<T: KeyboardInput> Cpu<T> {
    pub fn new(file_path: &str, keyboard: T) -> Cpu<T> {
        let mut file = File::open(Path::new(file_path)).expect("Failed to open the file");
        let mut buffer: Vec<u8> = Vec::new();
        file.read_to_end(&mut buffer)
            .expect("Failed to read the file");

        Self::from_bytes(&buffer, keyboard)
    }

    pub fn from_bytes(rom_data: &[u8], keyboard: T) -> Cpu<T> {
        let mut cpu = Cpu {
            registers: [0; 16],
            program_counter: 0x200,
            memory: [0; 0x1000],
            stack: [0; 16],
            stack_pointer: 0,
            index_register: 0,
            delay_timer: 0,
            sound_timer: 0,
            display: [[false; DISPLAY_WIDTH]; DISPLAY_HEIGHT],
            key: None,
            keyboard,
        };

        for (i, byte) in FONTSET.iter().enumerate() {
            cpu.memory[i] = *byte;
        }

        for (i, byte) in rom_data.iter().enumerate() {
            cpu.memory[0x200 + i] = *byte;
        }

        cpu
    }

    fn read_opcode(&self) -> u16 {
        let p = self.program_counter;
        let op_byte_1 = self.memory[p] as u16;
        let op_byte_2 = self.memory[p + 1] as u16;
        op_byte_1 << 8 | op_byte_2
    }

    pub fn decrement_timers(&mut self) {
        if self.delay_timer > 0 {
            self.delay_timer -= 1;
        }

        if self.sound_timer > 0 {
            self.sound_timer -= 1;
        }
    }

    pub fn get_display(&self) -> &[[bool; DISPLAY_WIDTH]; DISPLAY_HEIGHT] {
        &self.display
    }

    fn logging(&self, text: &str) {
        debug!(
            "{} v={:?} i={}({:x}) stack={:?} sp={:x} pc={}({:x}) dt={:x} key={:?}",
            text,
            self.registers,
            self.index_register,
            self.index_register,
            self.stack,
            self.stack_pointer,
            self.program_counter,
            self.program_counter,
            self.delay_timer,
            self.key
        );
    }

    pub fn update(&mut self) {
        let opcode = self.read_opcode();

        self.program_counter += 2;

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
                panic!("opcode {:04x}", opcode);
            }
        }
    }

    fn sys_addr(&mut self, nnn: u16) {
        self.logging(&format!("0nnn - SYS {}", nnn));
        self.program_counter = nnn as usize;
    }

    fn cls(&mut self) {
        self.logging("00E0 - CLS");
        self.display = [[false; DISPLAY_WIDTH]; DISPLAY_HEIGHT];
    }

    fn ret(&mut self) {
        self.logging("00EE - RET");
        if self.stack_pointer == 0 {
            panic!("Stack underflow")
        }

        self.stack_pointer -= 1;
        self.program_counter = self.stack[self.stack_pointer] as usize;
    }

    fn jp_addr(&mut self, nnn: u16) {
        self.logging(&format!("1nnn - JP {}", nnn));
        self.program_counter = nnn as usize;
    }

    fn call(&mut self, nnn: u16) {
        self.logging(&format!("2nnn - CALL {}", nnn));
        let sp = self.stack_pointer;
        let stack = &mut self.stack;

        if sp > stack.len() {
            panic!("Stack overflow")
        }

        stack[sp] = self.program_counter as u16;
        self.stack_pointer += 1;
        self.program_counter = nnn as usize;
    }

    fn se_byte(&mut self, x: u8, kk: u8) {
        self.logging(&format!("3xkk - SE V{} KK:{}", x, kk));
        let vx = self.registers[x as usize];
        if vx == kk {
            self.program_counter += 2;
        }
    }

    fn sne_byte(&mut self, x: u8, kk: u8) {
        self.logging(&format!("4xkk - SNE V{} KK:{}", x, kk));
        let vx = self.registers[x as usize];
        if vx != kk {
            self.program_counter += 2;
        }
    }

    fn se_xy(&mut self, x: u8, y: u8) {
        self.logging(&format!("5xy0 - SE V{} V{}", x, y));
        let vx = self.registers[x as usize];
        let vy = self.registers[y as usize];

        if vx == vy {
            self.program_counter += 2;
        }
    }

    fn ld_byte(&mut self, x: u8, kk: u8) {
        self.logging(&format!("6xkk - LD V{} KK:{}", x, kk));
        self.registers[x as usize] = kk;
    }

    fn add_byte(&mut self, x: u8, kk: u8) {
        self.logging(&format!("7xkk - ADD V{} KK:{}", x, kk));
        let vx = self.registers[x as usize];
        self.registers[x as usize] = vx.overflowing_add(kk).0;
    }

    fn ld_xy(&mut self, x: u8, y: u8) {
        self.logging(&format!("8xy0 - LD V{} V{}", x, y));
        self.registers[x as usize] = self.registers[y as usize];
    }

    fn or_xy(&mut self, x: u8, y: u8) {
        self.logging(&format!("8xy1 - OR V{} V{}", x, y));
        self.registers[x as usize] |= self.registers[y as usize];
    }

    fn and_xy(&mut self, x: u8, y: u8) {
        self.logging(&format!("8xy2 - AND V{} V{}", x, y));
        self.registers[x as usize] &= self.registers[y as usize];
    }

    fn xor_xy(&mut self, x: u8, y: u8) {
        self.logging(&format!("8xy3 - XOR V{} V{}", x, y));
        self.registers[x as usize] ^= self.registers[y as usize];
    }

    fn add_xy(&mut self, x: u8, y: u8) {
        self.logging(&format!("8xy4 - ADD V{} V{}", x, y));
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
        self.logging(&format!("8xy5 - SUB V{} V{}", x, y));
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
        self.logging(&format!("8xy6 - SHR V{}", x));
        let vx = self.registers[x as usize];
        // 最下位ビットをVfにセット
        self.registers[0xF] = vx & 0x01;
        // 右シフト（2で割る）
        self.registers[x as usize] = vx >> 1;
    }

    fn subn_xy(&mut self, x: u8, y: u8) {
        self.logging(&format!("8xy7 - SUBN V{} V{}", x, y));
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
        self.logging(&format!("8xyE - SHL V{}", x));
        let vx = self.registers[x as usize];
        // 最上位ビット（7番目のビット）をVfにセット
        self.registers[0xF] = (vx & 0x80) >> 7;
        // 左シフト（2倍）
        self.registers[x as usize] = vx.overflowing_mul(2).0;
    }

    fn sne_xy(&mut self, x: u8, y: u8) {
        self.logging(&format!("9xy0 - SNE V{} V{}", x, y));
        if self.registers[x as usize] != self.registers[y as usize] {
            self.program_counter += 2;
        }
    }

    fn ld_i_addr(&mut self, nnn: u16) {
        self.logging(&format!("Annn - LD I {}", nnn));
        self.index_register = nnn;
    }

    fn jp_v0_addr(&mut self, nnn: u16) {
        self.logging(&format!("Bnnn - JP V0 {}", nnn));
        self.program_counter = (self.registers[0] as u16 + nnn) as usize;
    }

    fn rnd_byte(&mut self, x: u8, kk: u8) {
        self.logging(&format!("Cxkk - RND V{} {}", x, kk));
        let random_number: u8 = random();
        self.registers[x as usize] = random_number & kk;
    }

    fn drw_xy(&mut self, x: u8, y: u8, n: u8) {
        self.logging(&format!("Dxyn - DRW V{} V{} {}", x, y, n));
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
        self.logging(&format!("Ex9E - SKP V{}", x));
        let vx = self.registers[x as usize];
        if let Some(key) = self.keyboard.get_key() {
            if key == vx {
                self.program_counter += 2;
                self.key = None;
            }
        }
    }

    fn sknp_vx(&mut self, x: u8) {
        self.logging(&format!("ExA1 - SKNP V{}", x));
        let vx = self.registers[x as usize];
        if let Some(key) = self.keyboard.get_key() {
            if key != vx {
                self.program_counter += 2;
                self.key = None;
            }
        }
    }

    fn ld_vx_dt(&mut self, x: u8) {
        self.logging(&format!("Fx07 - LD V{} DT", x));
        self.registers[x as usize] = self.delay_timer;
    }

    fn ld_vx_k(&mut self, x: u8) {
        self.logging(&format!("Fx0A - LD V{} K", x));
        let mut pressed = false;

        if let Some(key) = self.keyboard.get_key() {
            self.registers[x as usize] = key;
            pressed = true;
        }

        if !pressed {
            self.program_counter -= 2; //キーが押されるまで待つ
        }
    }

    fn ld_dt_vx(&mut self, x: u8) {
        self.logging(&format!("Fx15 - LD DT V{}", x));
        self.delay_timer = self.registers[x as usize];
    }

    fn ld_st_vx(&mut self, x: u8) {
        self.logging(&format!("Fx18 - LD ST V{}", x));
        self.sound_timer = self.registers[x as usize];
    }

    fn add_i_vx(&mut self, x: u8) {
        self.logging(&format!("Fx1E - ADD I V{}", x));
        let vx = self.registers[x as usize];
        self.index_register += vx as u16;
    }

    fn ld_f_vx(&mut self, x: u8) {
        self.logging(&format!("Fx29 - LD F V{}", x));
        self.index_register = self.registers[x as usize] as u16 * 5;
    }

    fn ld_b_vx(&mut self, x: u8) {
        self.logging(&format!("Fx33 - LD B V{}", x));
        let vx = self.registers[x as usize];
        self.memory[self.index_register as usize] = (vx / 100) % 10;
        self.memory[self.index_register as usize + 1] = (vx / 10) % 10;
        self.memory[self.index_register as usize + 2] = vx % 10;
    }

    fn ld_i_vx(&mut self, x: u8) {
        self.logging(&format!("Fx55 - LD [I] V{}", x));
        for i in 0..=x {
            self.memory[self.index_register as usize + i as usize] = self.registers[i as usize];
        }
    }

    fn ld_vx_i(&mut self, x: u8) {
        self.logging(&format!("Fx65 - LD V{} [I]", x));
        for i in 0..=x {
            self.registers[i as usize] = self.memory[self.index_register as usize + i as usize];
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::mpsc;

    // テスト用のモックキーボード構造体
    struct MockKeyboard {
        key: Option<u8>,
    }

    impl KeyboardInput for MockKeyboard {
        fn start_keyboard_thread(_sender: mpsc::Sender<u8>) {
            // テストでは何もしない
        }

        fn get_key(&self) -> Option<u8> {
            self.key
        }
    }

    fn setup_cpu() -> Cpu<MockKeyboard> {
        // テスト用の空ファイルを作成
        let temp_path = "test.ch8";
        std::fs::write(temp_path, vec![0; 10]).expect("Failed to create test file");
        
        let keyboard = MockKeyboard { key: None };
        let cpu = Cpu::new(temp_path, keyboard);
        
        std::fs::remove_file(temp_path).expect("Failed to remove test file");
        cpu
    }

    #[test]
    fn test_add_xy() {
        let mut cpu = setup_cpu();
        
        // ケース1: 通常の加算（オーバーフローなし）
        cpu.registers[0] = 0x10;  // 16
        cpu.registers[1] = 0x20;  // 32
        cpu.add_xy(0, 1);
        assert_eq!(cpu.registers[0], 0x30);  // 48
        assert_eq!(cpu.registers[0xF], 0);   // キャリーフラグはセットされない

        // ケース2: オーバーフローする加算
        cpu.registers[0] = 0xFF;  // 255
        cpu.registers[1] = 0x02;  // 2
        cpu.add_xy(0, 1);
        assert_eq!(cpu.registers[0], 0x01);  // 257 % 256 = 1
        assert_eq!(cpu.registers[0xF], 1);   // キャリーフラグがセットされる

        // ケース3: ちょうど256になる加算
        cpu.registers[0] = 0xFE;  // 254
        cpu.registers[1] = 0x02;  // 2
        cpu.add_xy(0, 1);
        assert_eq!(cpu.registers[0], 0x00);  // 256 % 256 = 0
        assert_eq!(cpu.registers[0xF], 1);   // キャリーフラグがセットされる
    }
}
