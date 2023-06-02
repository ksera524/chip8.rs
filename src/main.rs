struct CPU {
    registers: [u8; 16],
    position_in_memory: usize,
    memory: [u8; 0x1000],
    stack: [u16; 16],
    stack_pointer: usize,
}

impl CPU {
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

            println!(
                "mem: {:0x} c: {:0x}, x: {:0x}, y: {:0x}, d: {:0x}",
                self.position_in_memory, c, x, y, d
            );
            println!("register : {} , {}", self.registers[0], self.registers[1]);
            println!("stack_pointer : {}", self.stack_pointer);
            println!("stack : {:?}", self.stack);

            match (c, x, y, d) {
                (0, 0, 0, 0) => {return;}
                (0, _, _, _) => self.sys(nnn),
                (0, 0, 0xE, 0xE) => self.ret(),
                (0x2, _, _, _) => self.call(nnn),
                (0x8, _, _, 0x4) => self.add_xy(x, y),
                (0x8, _, _, 0x5) => self.sub_xy(x, y),
                (0x9, _, _, 0x0) => self.sne(x, y),
                _ => todo!("opcode {:04x}", opcode),
            }
        }
    }

    fn sys(&mut self, addr: u16) {
        todo!();
    }

    fn jp(&mut self, addr: u16) {
        self.position_in_memory = addr as usize;
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

    fn ret(&mut self) {
        if self.stack_pointer == 0 {
            panic!("Stack underflow")
        }

        self.stack_pointer -= 1;
        self.position_in_memory = self.stack[self.stack_pointer] as usize;
    }

    fn sne(&mut self, x: u8, y: u8) {
        if self.registers[x as usize] != self.registers[y as usize] {
            self.position_in_memory += 2;
        }
    }
}

fn main() {
    let mut cpu = CPU {
        registers: [0; 16],
        position_in_memory: 0,
        memory: [0; 0x1000],
        stack: [0; 16],
        stack_pointer: 0,
    };

    cpu.registers[0] = 5;
    cpu.registers[1] = 10;
    cpu.registers[2] = 45;

    let mem = &mut cpu.memory;

    mem[0x000] = 0x21;mem[0x001] = 0x00;
    mem[0x002] = 0x21;mem[0x003] = 0x00;
    mem[0x004] = 0x80;mem[0x005] = 0x25;
    mem[0x006] = 0x00;mem[0x007] = 0x00;


    mem[0x100] = 0x80;mem[0x101] = 0x14;
    mem[0x102] = 0x80;mem[0x103] = 0x14;
    mem[0x104] = 0x00;mem[0x105] = 0xEE;

    println!("registers: {:?}", cpu.registers);
    //println!("memory: {:?}",cpu.memory);

    cpu.run();

    println!("5 + (10 * 2) + (10 * 2) - 45  = {}", cpu.registers[0]);
}
