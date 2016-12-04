fn main() {
    println!("Hello, world!");
}

struct Cpu {
    opcode: u16,
    v: [u8; 16],
    i: u16,
    sound_timer: u8,
    delay_timer: u8,
    pc: usize,
    sp: usize,
    stack: [u16; 16],
    memory: [u8; 4096],
    keypad: [bool; 16],
    display: [[u8; 32]; 64]
}

impl Cpu {
    fn new() -> Cpu {

        let cpu = Cpu {
            opcode: 0,
            v: [0; 16],
            i: 0x200,
            sound_timer: 0,
            delay_timer: 0,
            pc: 0x200,
            sp: 0,
            stack: [0; 16],
            memory: [0; 4096],
            keypad: [false; 16],
            display: [[0; 32]; 64]
        };

        return cpu;
    }

    fn emulate_cycle(&mut self) {
        self.fetch_opcode();
        self.opcode_execute();
    }

    fn load_bytes(&mut self, data: Vec<u8>) {
        for (index, &byte) in data.iter().enumerate() {
            self.memory[index] = byte;
        }
    }

    fn fetch_opcode(&mut self) {
        self.opcode = (self.memory[self.pc] as u16) << 8 | (self.memory[self.pc + 1] as u16);
    }

    fn inc_pc(&mut self) {
        self.pc += 2;
    }

    fn opcode_execute(&mut self) {
        match self.opcode & 0xf000 {
            0x0000 => self.op_0xxx(),
            0x1000 => self.op_jp(),
            0x2000 => self.op_call(),
            0x3000 => self.op_se(),
            0x4000 => self.op_sne(),
            0x5000 => self.op_se_vx_vy(),
            0x6000 => self.op_ld_vx_byte(),
            _      => {
                println!("opcode {}, masked {} not implemented.", self.opcode, self.opcode & 0xf000); 
                unimplemented!()
            }
        }
    }

    fn op_0xxx(&mut self) {
        match self.opcode {
            0x00E0 => self.op_cls(),
            0x00EE => self.op_ret(),
            _      => unimplemented!()
        }
    }

    // 00E0 - CLS -- Clear the display.
    fn op_cls(&mut self) {
        self.display = [[0; 32]; 64];

        self.inc_pc();
    }

    // 00EE - RET -- Return from a subroutine.
    // Sets program counter to address at the top of the stack, then subtracts 1 from
    // the stack pointer.
    fn op_ret(&mut self) {
        self.sp -= 1;
        self.pc = self.stack[self.sp] as usize;
        self.inc_pc();
    }

    // 1nnn - JP addr -- Jump to location nnn
    // Sets the program counter to nnn.
    fn op_jp(&mut self) {
        self.pc = self.get_nnn() as usize;
    }

    // 2nnn - CALL addr -- Call subroutine at nnn
    // Increments the stack pointer, then puts the current PC on the top of the stack.
    // The PC is then set to nnn.
    fn op_call(&mut self) {
        self.stack[self.sp] = self.pc as u16;
        self.sp += 1;
        self.pc = self.get_nnn() as usize;
    }

    // 3xkk - SE Vx, byte -- Skip next instruction if Vx = kk
    // Compare register Vx to kk, and if equal, increment the program counter by 2.
    fn op_se(&mut self) {
        if self.v[self.get_x() as usize] == self.get_kk() {
            self.inc_pc();
        }

        self.inc_pc();
    }

    // 4xkk - SNE Vx, byte -- Skip next instruction if Vx != kk
    // Compare register Vx to kk, and if not equal, increment the program counter by 2.
    fn op_sne(&mut self) {
        if self.v[self.get_x() as usize] != self.get_kk() {
            self.inc_pc();
        }

        self.inc_pc();
    }

    // 5xy0 - SE Vx, Vy -- Skip next instruction if Vx = Vy
    // Compare register Vx to register Vy, and if they are equal, increment
    // the program counter by 2.
    // For now we're gonna be lazy. 5xyz is considered equivalent to 5xy0 for all z.
    fn op_se_vx_vy(&mut self) {
        if self.v[self.get_x() as usize] == self.v[self.get_y() as usize] {
            self.inc_pc();
        }

        self.inc_pc();
    }

    // 6xkk - LD Vx, byte -- Set Vx = kk
    // Puts the value kk into register Vx.
    fn op_ld_vx_byte(&mut self) {
        self.v[self.get_x() as usize] = self.get_kk();
        self.inc_pc();
    }

    fn get_nnn(&self) -> u16 { self.opcode & 0x0fff }
    fn get_kk(&self) -> u8 { (self.opcode & 0x00ff) as u8 }
    fn get_x(&self) -> u8 { ((self.opcode & 0x0f00) >> 8) as u8 }
    fn get_y(&self) -> u8 { ((self.opcode & 0x00f0) >> 4) as u8 }
    fn get_n(&self) -> u8 { (self.opcode & 0x000f) as u8 }


}

#[cfg(test)]
mod tests {
    use super::*;

    fn load_data(cpu: &mut Cpu, data_to_load: Vec<u8>) {
        let mut data = vec![0; 0x200];
        for byte in data_to_load {
            data.push(byte)
        }
        cpu.load_bytes(data);
    }

    #[test]
    fn test_loading_bytes_from_vector() {
        let data = vec![0x1, 0x2, 0x3, 0x4];
        let mut cpu = Cpu::new();
        let mut results = [0; 4096];
        for (index, &byte) in data.iter().enumerate() {
            results[index] = byte;
        }
        cpu.load_bytes(data);
        for i in 0..4096 {
            assert_eq!(results[i], cpu.memory[i])
        }
    }

    #[test]
    fn test_fetching_opcode() {
        let data = vec![1, 1];
        let mut cpu = Cpu::new();
        load_data(&mut cpu, data);
        cpu.fetch_opcode();
        assert_eq!(257, cpu.opcode)
    }

    #[test]
    fn test_op_cls() {
        let mut cpu = Cpu::new();
        cpu.display = [[1; 32]; 64];
        cpu.op_cls();
        for i in 0..64 {
            for ii in 0..32 {
                assert_eq!(0, cpu.display[i][ii])
            }
        }
    }

    #[test]
    fn test_execute_ret() {
        let mut cpu = Cpu::new();

        // Load the following program into the CPU.
        // 0x200: 00E0 
        // 0x202: 2386 
        // 0x386: 00EE

        // 0x200: CLS
        // 0x202: CALL 386
        // 0x386: 00EE
        load_data(&mut cpu, vec![0x00, 0xE0, 0x23, 0x86]);
        cpu.memory[0x386] = 0x00;
        cpu.memory[0x387] = 0xEE;
        for _ in 0..3 {
            cpu.emulate_cycle();
        }
        assert_eq!(cpu.pc, 0x204);
        assert_eq!(cpu.sp, 0);
        assert_eq!(cpu.stack[0], 0x202);
    }

    #[test]
    fn test_op_jp() {
        let mut cpu = Cpu::new();
        cpu.opcode = 0x1386;
        cpu.op_jp();
        assert_eq!(cpu.pc, 0x386);
    }

    #[test]
    fn test_execute_jp() {
        let mut cpu = Cpu::new();
        load_data(&mut cpu, vec![0x13, 0x86]);
        cpu.emulate_cycle();
        assert_eq!(cpu.pc, 0x386);
    }

    #[test]
    fn test_execute_call() {
        let mut cpu = Cpu::new();
        load_data(&mut cpu, vec![0x00, 0xE0, 0x23, 0x86]);
        cpu.emulate_cycle();
        cpu.emulate_cycle();
        assert_eq!(cpu.pc, 0x386);
        assert_eq!(cpu.sp, 1);
        assert_eq!(cpu.stack[0], 0x202);
    }

    #[test]
    fn test_se_if_true() {
        let mut cpu = Cpu::new();
        cpu.v[3] = 0x88;
        load_data(&mut cpu, vec![0x33, 0x88]);
        cpu.emulate_cycle();
        assert_eq!(cpu.pc, 0x200 + 4);
    }

    #[test]
    fn test_se_if_false() {
        let mut cpu = Cpu::new();
        cpu.v[3] = 0x84;
        load_data(&mut cpu, vec![0x33, 0x88]);
        cpu.emulate_cycle();
        assert_eq!(cpu.pc, 0x200 + 2);
    }

    #[test]
    fn test_sne_if_true() {
        let mut cpu = Cpu::new();
        cpu.v[3] = 0x84;
        load_data(&mut cpu, vec![0x43, 0x88]);
        cpu.emulate_cycle();
        assert_eq!(cpu.pc, 0x200 + 4);
    }
    
    #[test]
    fn test_sne_if_false() {
        let mut cpu = Cpu::new();
        cpu.v[3] = 0x88;
        load_data(&mut cpu, vec![0x43, 0x88]);
        cpu.emulate_cycle();
        assert_eq!(cpu.pc, 0x200 + 2);
    }

    #[test]
    fn test_se_vs_if_true() {
        let mut cpu = Cpu::new();
        cpu.v[3] = 0x88;
        cpu.v[6] = 0x88;
        load_data(&mut cpu, vec![0x53, 0x60]);
        cpu.emulate_cycle();
        assert_eq!(cpu.pc, 0x200 + 4);
    }

    #[test]
    fn test_se_vs_if_false() {
        let mut cpu = Cpu::new();
        cpu.v[3] = 0x84;
        cpu.v[6] = 0x88;
        load_data(&mut cpu, vec![0x33, 0x60]);
        cpu.emulate_cycle();
        assert_eq!(cpu.pc, 0x200 + 2);
    }

    #[test]
    fn test_ld_vx_byte() {
        let mut cpu = Cpu::new();
        load_data(&mut cpu, vec![0x63, 0x92]);
        cpu.emulate_cycle();
        assert_eq!(cpu.v[3], 0x92);
    }
}