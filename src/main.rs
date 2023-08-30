use rand::Rng;
use std::fs::File;
use std::io::Read;
use std::path::Path;

struct CPU {
    registers: [u8; 16],
    position_in_memory: usize,//program counter
    memory: [u8; 0x1000],
    stack: [u16; 16],
    stack_pointer: usize,
    index_register: u16, 
    delay_timer: u8,
}

impl CPU {
    fn new(file_path: &str) -> CPU {
        let mut cpu = CPU {
            registers: [0; 16],
            position_in_memory: 0x200,
            memory: [0; 0x1000],
            stack: [0; 16],
            stack_pointer: 0,
            index_register: 0,
            delay_timer: 0,
        };

        let mut file = File::open(Path::new(file_path)).expect("Failed to open the file");
        let mut buffer: Vec<u8> = Vec::new();
        file.read_to_end(&mut buffer).expect("Failed to read the file");

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

    fn run(&mut self) {
        loop {
            let opcode = self.read_opcode();
            self.position_in_memory += 2;

            let c = ((opcode & 0xF000) >> 12) as u8;
            let x = ((opcode & 0x0F00) >> 8) as u8;
            let y = ((opcode & 0x00F0) >> 4) as u8;
            let d = ((opcode & 0x000F) >> 0) as u8;

            let nnn = opcode & 0x0FFF;
            let kk = opcode & 0x00FF;

            match (c, x, y, d) {
                (0, 0, 0, 0) => {return;}
                (0, _, _, _) => self.sus_addr(nnn),
                (0, 0, 0xE, 0xE) => self.ret(),
                (0x1, _, _, _) => self.jp_addr(nnn),
                (0x2, _, _, _) => self.call(nnn),
                (0x3, _, _, _) => self.se_byte(x, kk as u8),
                (0x4, _, _, _) => self.sne_byte(x, kk as u8),
                (0x5, _, _, 0) => self.se_xy(x,y),
                (0x6, _, _, _) => self.ld_byte(x,kk as u8),
                (0x7, _, _, _) => self.add_byte(x,kk as u8),
                (0x8, _, _, 0) => self.ld_xy(x,y),
                (0x8, _, _, 1) => self.or_xy(x,y),
                (0x8, _, _, 2) => self.and_xy(x,y),
                (0x8, _, _, 3) => self.xor_xy(x,y),
                (0x8, _, _, 4) => self.add_xy(x,y),
                (0x8, _, _, 5) => self.sub_xy(x,y),
                (0x8, _, _, 6) => self.shr_xy(x),
                (0x8, _, _, 7) => self.subn_xy(x,y),
                (0x8, _, _, 0xE) => self.shl_xy(x),
                (0x9, _, _, 0) => self.sne_xy(x,y),
                (0xA, _, _, _) => self.ld_i_addr(nnn),
                (0xB, _, _, _) => self.jp_v0_addr(nnn),
                (0xC, _, _, _) => self.rnd_byte(x,kk as u8),
                (0xD, _, _, _) => self.drw_xy(x,y,d),
                (0xE, _, 9, 0xE) => self.skp_vx(x),
                (0xE, _, 0xA, 1) => self.sknp_vx(x),
                (0xF, _, 0, 7) => self.ld_vx_dt(x),
                (0xF, _, 0, 0xA) => self.ld_vx_k(x),
                (0xF, _, 1, 5) => self.ld_dt_vx(x),
                (0xF, _, 1, 8) => self.ld_st_vx(x),
                (0xF, _, 1, 0xE) => self.add_i_vx(x),
                (0xF, _, 2, 9) => self.ld_f_vx(x),
                (0xF, _, 3, 3) => self.ld_b_vx(x),
                (0xF, _, 5, 5) => self.ld_i_vx(x),
                (0xF, _, 6, 5) => self.ld_vx_i(x),
                _ => todo!("opcode {:04x}", opcode),
            }
        }
    }

    fn sus_addr(&mut self, addr: u16) {
        todo!();
    }

    fn ret(&mut self) {
        if self.stack_pointer == 0 {
            panic!("Stack underflow")
        }

        self.stack_pointer -= 1;
        self.position_in_memory = self.stack[self.stack_pointer] as usize;
    }

    fn jp_addr(&mut self, addr: u16) {
        self.position_in_memory = addr as usize;
    }

    fn call(&mut self, addr: u16) {
        let sp = self.stack_pointer;
        let stack = &mut self.stack;

        if sp > stack.len() {
            panic!("Stack overflow")
        }

        stack[sp] = self.position_in_memory as u16;
        self.stack_pointer += 1;
        self.position_in_memory = addr as usize;
    }

    fn se_byte(&mut self, x: u8, byte: u8) {
        if self.registers[x as usize] == byte {
            self.position_in_memory += 2;
        }
    }

    fn sne_byte(&mut self, x: u8, byte: u8) {
        if self.registers[x as usize] != byte {
            self.position_in_memory += 2;
        }
    }

    fn se_xy(&mut self, x: u8, y: u8) {
        if self.registers[x as usize] == self.registers[y as usize] {
            self.position_in_memory += 2;
        }
    }

    fn ld_byte(&mut self, x: u8, byte: u8) {
        self.registers[x as usize] = byte;
    }

    fn add_byte(&mut self, x: u8, byte: u8) {
        let args1 = self.registers[x as usize];
        let (val, overflow) = args1.overflowing_add(byte);
        self.registers[x as usize] = val;

        if overflow {
            self.registers[0xF] = 1;
        } else {
            self.registers[0xF] = 0;
        }
    }

    fn ld_xy(&mut self, x: u8, y: u8) {
        self.registers[x as usize] = self.registers[y as usize];
    }

    fn or_xy(&mut self, x: u8, y: u8) {
        self.registers[x as usize] |= self.registers[y as usize];
    }

    fn and_xy(&mut self, x: u8, y: u8) {
        self.registers[x as usize] &= self.registers[y as usize];
    }

    fn xor_xy(&mut self, x: u8, y: u8) {
        self.registers[x as usize] ^= self.registers[y as usize];
    }

    fn add_xy(&mut self, x: u8, y: u8) {
        let args1 = self.registers[x as usize];
        let args2 = self.registers[y as usize];

        let (val, overflow) = args1.overflowing_add(args2);
        self.registers[x as usize] = val;

        if overflow {
            self.registers[0xF] = 1;
        } else {
            self.registers[0xF] = 0;
        }
    }

    fn sub_xy(&mut self,x:u8,y:u8){
        let args1 = self.registers[x as usize];
        let args2 = self.registers[y as usize];

        let (val, overflow) = args1.overflowing_sub(args2);
        self.registers[x as usize] = val;

        if overflow {
            self.registers[0xF] = 1;
        } else {
            self.registers[0xF] = 0;
        }
    }

    fn shr_xy(&mut self, x: u8) {
        let args1 = self.registers[x as usize];
        let (val, overflow) = args1.overflowing_shr(1);
        self.registers[x as usize] = val;

        if overflow {
            self.registers[0xF] = 1;
        } else {
            self.registers[0xF] = 0;
        }
    }

    fn subn_xy(&mut self, x: u8, y: u8) {
        let args1 = self.registers[x as usize];
        let args2 = self.registers[y as usize];

        let (val, overflow) = args2.overflowing_sub(args1);
        self.registers[x as usize] = val;

        if overflow {
            self.registers[0xF] = 1;
        } else {
            self.registers[0xF] = 0;
        }
    }

    fn shl_xy(&mut self, x: u8) {
        let args1 = self.registers[x as usize];
        let (val, overflow) = args1.overflowing_shl(1);
        self.registers[x as usize] = val;

        if overflow {
            self.registers[0xF] = 1;
        } else {
            self.registers[0xF] = 0;
        }
    }

    fn sne_xy(&mut self, x: u8, y: u8) {
        if self.registers[x as usize] != self.registers[y as usize] {
            self.position_in_memory += 2;
        }
    }

    fn ld_i_addr(&mut self, addr: u16) {
        self.index_register = addr;
    }

    fn jp_v0_addr(&mut self, addr: u16) {
        self.position_in_memory = (self.registers[0] as u16 + addr) as usize;
    }

    fn rnd_byte(&mut self, x: u8, byte: u8) {
        let mut rng = rand::thread_rng();
        let random_number: u8 = rng.gen();
        self.registers[x as usize] = random_number & byte;
    }

    fn drw_xy(&mut self, x: u8, y: u8, n: u8) {
        todo!("Implement this")
    }

    fn skp_vx(&mut self, x: u8) {
        todo!("Implement this")
    }

    fn sknp_vx(&mut self, x: u8) {
        todo!("Implement this")
    }

    fn ld_vx_dt(&mut self, x: u8) {
        self.registers[x as usize] = self.delay_timer;
    }

    fn ld_vx_k(&mut self, x: u8) {
        todo!("Implement this")
    }

    fn ld_dt_vx(&mut self, x: u8) {
        self.delay_timer = self.registers[x as usize];
    }

    fn ld_st_vx(&mut self, x: u8) {
        todo!("Implement this")
    }

    fn add_i_vx(&mut self, x: u8) {
        self.index_register += self.registers[x as usize] as u16;
    }

    fn ld_f_vx(&mut self, x: u8) {
        todo!("Implement this")
    }

    fn ld_b_vx(&mut self, x: u8) {
        self.memory[self.index_register as usize] = self.registers[x as usize] / 100;
        self.memory[self.index_register as usize + 1] = (self.registers[x as usize] / 10) % 10;
        self.memory[self.index_register as usize + 2] = (self.registers[x as usize] % 100) % 10;
    }

    fn ld_i_vx(&mut self, x: u8) {
        for i in 0..=x {
            self.memory[self.index_register as usize + i as usize] = self.registers[i as usize];
        }
    }

    fn ld_vx_i(&mut self, x: u8) {
        for i in 0..=x {
            self.registers[i as usize] = self.memory[self.index_register as usize + i as usize];
        }
    }
}

fn main() {
    let mut cpu = CPU::new("tetris.ch8");
    cpu.run();
}
