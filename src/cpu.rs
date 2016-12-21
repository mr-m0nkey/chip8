extern crate rand;

use std::time::{Duration, Instant};
use std::process;

pub struct Cpu {
    opcode: u16,
    v: [u8; 16],
    i: u16,
    sound_timer: u8,
    delay_timer: u8,
    pc: usize,
    sp: usize,
    stack: [u16; 16],
    memory: [u8; 4096],
    pub key_buff: [bool; 16],
    pub disp_buff: [[bool; 64]; 32],
    time_at_last_timer_count: Instant
}

impl Cpu {
    pub fn new() -> Cpu {

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
            key_buff: [false; 16],
            disp_buff: [[false; 64]; 32],
            time_at_last_timer_count: Instant::now()
        };

        return cpu;
    }

    pub fn emulate_cycle(&mut self) {
        self.fetch_opcode();
        self.opcode_execute();
        self.count_timers();
    }

    fn count_timers(&mut self) {
        if Instant::now() - self.time_at_last_timer_count >= Duration::from_millis(17) {
            self.time_at_last_timer_count = Instant::now();
            if self.sound_timer > 0 {
                self.sound_timer = self.sound_timer - 1;
            }
            if self.delay_timer > 0 {
                self.delay_timer = self.delay_timer - 1;
            }
        }
    }

    fn load_bytes(&mut self, data: Vec<u8>) {
        for (index, &byte) in data.iter().enumerate() {
            self.memory[index] = byte;
        }
    }

    pub fn load_data(cpu: &mut Cpu, data_to_load: Vec<u8>) {
        let mut data = vec![0; 0x200];
        for i in 0..80 {
            data[i] = FONT_SPRITES[i];
        }
        for byte in data_to_load {
            data.push(byte)
        }
        cpu.load_bytes(data);
    }

    fn fetch_opcode(&mut self) {
        self.opcode = (self.memory[self.pc] as u16) << 8 | (self.memory[self.pc + 1] as u16);
    }

    fn inc_pc(&mut self) {
        self.pc += 2;
    }

    fn opcode_unimplemented(&self) {
        println!("opcode {:X} is not implemented.", self.opcode);
        println!("Emulator exiting.");
        process::exit(0);
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
            0x7000 => self.op_add_vx_byte(),
            0x8000 => self.op_8xxx(),
            0x9000 => self.op_sne_vx_vy(),
            0xA000 => self.op_ld_i_addr(),
            0xB000 => self.op_jp_v0_addr(),
            0xC000 => self.op_rnd_vx_byte(),
            0xD000 => self.op_drw_vx_vy_n(),
            0xE000 => self.op_exxx(),
            0xF000 => self.op_fxxx(),
            _      => self.opcode_unimplemented()
        }
    }

    fn op_0xxx(&mut self) {
        match self.opcode {
            0x00E0 => self.op_cls(),
            0x00EE => self.op_ret(),
            0x0000 => {
                println!("Reached a 0000 instruction. Emulation terminated.");
                process::exit(0);
            }
            _      => self.opcode_unimplemented()
        }
    }

    fn op_8xxx(&mut self) {
        match self.opcode & 0x000f {
            0   => self.op_ld_vx_vy(),
            1   => self.op_or(),
            2   => self.op_and(),
            3   => self.op_xor(),
            4   => self.op_add_vx_vy(),
            5   => self.op_sub_vx_vy(),
            6   => self.op_shr_vx_vy(),
            7   => self.op_subn_vx_vy(),
            0xE => self.op_shl_vx_vy(),
            _   => self.opcode_unimplemented()
        }
    }

    fn op_fxxx(&mut self) {
        match self.opcode & 0x00FF {
            0x07 => self.op_ld_vx_dt(),
            0x0A => self.op_ld_vx_k(),
            0x15 => self.op_ld_dt_vx(),
            0x18 => self.op_ld_st_vx(),
            0x29 => self.op_ld_f_vx(),
            0x33 => self.op_ld_b_vx(),
            0x55 => self.op_ld_i_vx(),
            0x65 => self.op_ld_vx_i(),
            _    => self.opcode_unimplemented()
        }
    }

    fn op_exxx(&mut self) {
        match self.opcode & 0x00FF {
            0x9E => self.op_skp_vx(),
            0xA1 => self.op_sknp_vx(),
            _    => self.opcode_unimplemented()
        }
    }

    // 00E0 - CLS -- Clear the display.
    fn op_cls(&mut self) {
        self.disp_buff = [[false; 64]; 32];

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

    // 7xkk - ADD Vx, byte
    // Adds the value kk to the value of register Vx, then stores result in Vx.
    // In case of overflow, just add, and take the 8 rightmost bits.
    fn op_add_vx_byte(&mut self) {
        let x = self.get_x() as usize;
        // So rust lets us add without overflowing, cast each number to u16.
        // Then, as our register only accepts u8, cast back to u8.
        // casting to u8 is defined to truncate for us.
        self.v[x] = ((self.v[x] as u16) + (self.get_kk() as u16)) as u8;
        self.inc_pc();
    }

    // 8xy0 - LD Vx, Vy -- Set Vx = Vy.
    // Stores the value of register Vy in register Vx.
    fn op_ld_vx_vy(&mut self) {
        let x = self.get_x() as usize;
        let y = self.get_y() as usize;
        self.v[x] = self.v[y];
        self.inc_pc();
    }

    // 8xy1 - OR Vx, Vy -- Set Vx = Vx OR Vy
    // Perform bitwise OR on values of Vx and Vy, store result in Vx.
    fn op_or(&mut self) {
        let x = self.get_x() as usize;
        let y = self.get_y() as usize;
        self.v[x] = self.v[x] | self.v[y];
        self.inc_pc();
    }

    // 8xy2 - AND Vx, Vy -- Set Vx = Vx AND Vy
    // Perform bitwise AND on values of Vx and Vy, store result in Vx.
    fn op_and(&mut self) {
        let x = self.get_x() as usize;
        let y = self.get_y() as usize;
        self.v[x] = self.v[x] & self.v[y];
        self.inc_pc();
    }

    // 8xy3 - XOR Vx, Vy -- Set Vx = Vx XOR Vy
    // Perform bitwise XOR on values of Vx and Vy, store result in Vx.
    fn op_xor(&mut self) {
        let x = self.get_x() as usize;
        let y = self.get_y() as usize;
        self.v[x] = self.v[x] ^ self.v[y];
        self.inc_pc();
    }

    // 8xy4 -- ADD Vx, Vy -- Set Vx = Vx + Vy, set VF = carry
    // Values of Vx and Vy are added. If result is greater than 8 bits, then
    // VF is set to 1, otherwise 0. The lowest 8 bits of result are kept and
    // stored in Vx.
    fn op_add_vx_vy(&mut self) {
        let x = self.get_x() as usize;
        let y = self.get_y() as usize;
        // As the addition could overflow the u8 bit values of the register, we need
        // to cast as u16s.
        let sum = (self.v[x] as u16) + (self.v[y] as u16);

        if sum > 0xFF { // 0xFF is maximum value of a u8
            self.v[0xF] = 1;
        } else {
            self.v[0xF] = 0;
        }

        self.v[x] = sum as u8;
        self.inc_pc();
    }

    // 8xy5 - SUB Vx, Vy -- Set Vx = Vx - Vy, set VF = not borrow
    // If Vx > Vy, VF is set to 1, otherwise 0. Then Vy is subtracted from Vx
    // (using wrap-around arithmetic), and the result is stored in Vx.
    fn op_sub_vx_vy(&mut self) {
        let x = self.get_x() as usize;
        let y = self.get_y() as usize;

        if self.v[x] > self.v[y] { 
            self.v[0xf] = 1;
        } else {
            self.v[0xf] = 0;
        }

        self.v[x] = self.v[x].wrapping_sub(self.v[y]);
        self.inc_pc();
    }

    // 8xy6 - SHR Vx, Vy -- Set Vx = Vy SHR 1
    // Set VF to least significant bit of Vy, shift value of Vy right by one,
    // and store the result to Vx.
    fn op_shr_vx_vy(&mut self) {
        let x = self.get_x() as usize;
        let y = self.get_y() as usize;

        self.v[0xf] = self.v[y] & 1;
        self.v[x] = self.v[y] >> 1;
        self.inc_pc();
    }

    // 8xy7 -- SUBN Vx, Vy -- Set Vx = Vy - Vx, set VF = NOT borrow.
    // If Vy > Vx, VF is set to 1, otherwise 0. Then Vx is subtracted from Vy
    // (using wrap-around arithmetic), and the result is stored in Vx.
    fn op_subn_vx_vy(&mut self) {
        let x = self.get_x() as usize;
        let y = self.get_y() as usize;

        if self.v[y] > self.v[x] { 
            self.v[0xf] = 1;
        } else {
            self.v[0xf] = 0;
        }

        self.v[x] = self.v[y].wrapping_sub(self.v[x]);
        self.inc_pc();
    }

    // 8xyE - SHL Vx, Vy -- Set Vx = Vy SHL 1
    // Set VF to most significant bit of Vy, shift value of Vy left by one,
    // and store the result to Vx.
    fn op_shl_vx_vy(&mut self) {
        let x = self.get_x() as usize;
        let y = self.get_y() as usize;

        self.v[0xf] = self.v[y]>> 7;
        self.v[x] = self.v[y] << 1;
        self.inc_pc();
    }

    //9xy0 - SNE Vx, Vy -- Skip next instruction if Vx != Vy
    // Values of Vx and Vy are compared. If not equal, program counter
    // is increased by two.
    fn op_sne_vx_vy(&mut self) {
        if self.v[self.get_x() as usize] != self.v[self.get_y() as usize] {
            self.inc_pc();
        }

        self.inc_pc();
    }

    //Annn - LD I, addr -- Set I = nnn.
    // The value of register I is set to nnn.
    fn op_ld_i_addr(&mut self) {
        self.i = self.get_nnn();

        self.inc_pc();
    }

    // Bnnn - JP V0, addr -- Jump to location nnn + V0
    // Program counter set to nnn plus the value of V0.
    fn op_jp_v0_addr(&mut self) {
        self.pc = self.v[0] as usize + self.get_nnn() as usize;
    }

    // Cxkk - RND Vx, byte -- Set Vx = random byte AND kk
    // Generate random value from 0 to 255, AND with value kk. Store result in Vx.
    fn op_rnd_vx_byte(&mut self) {
        self.v[self.get_x() as usize] = rand::random::<u8>() & self.get_kk();
        self.inc_pc();
    }

    // Dxyn - DRW Vx, Vy, nibble -- Display n-byte sprite starting at mem location
    // I at (Vx, Vy), set VF = collision.
    // Reads n bytes from memory, starting at address stored in I. These bytes
    // are then displayed as sprites on the screen at coords (Vx, Vy).
    // Sprites are XOR'd onto the screen. If this causes any pixels to be erased,
    // VF is set to 1, else 0. If the sprite is positioned so part of it is outside
    // of the coordinates of the display, it wraps around to the other side of the
    // screen. If sprite is to be displayted on the screen, Vx must be between
    // 00 and 3F and Vy must be between 00 and 1F.
    fn op_drw_vx_vy_n(&mut self) {
        let vx = self.v[self.get_x() as usize] as usize;
        let vy = self.v[self.get_y() as usize] as usize;
        let n = self.get_n() as usize;
        let i = self.i as usize;
        let mut flipped = false;

        if (vx > 0x3F) | (vy > 0x1F) { return; }
        {
            // Read n bytes from memory -- this is the sprite.
            // n is number of bytes, where each row of the sprite is 1 byte.
            let sprite = &self.memory[i .. i + n];

            // find our (x, y) to display pixel of sprite at
            // This gets the row we're on...
            for row_index in 0 .. n {
                if vy + row_index > 31 { break; }
                let row = &mut self.disp_buff[row_index + vy];

                // get the slice of the row we'll be modifying, starting at x = Vx
                let vxp8; // this clips "vx + 8" so that the maximum value is 64.
                if vx + 8 > 64 { vxp8 = 64; } else { vxp8 = vx + 8; }
                let row_slice = &mut row[vx .. vxp8];
                // Get the current row of the slice
                let cur_row_sprite = &sprite[row_index];

                // now apply it to display buffer's rows by XOR, flipping if necessary
                for pixel in 0..row_slice.len() {
                        // mask and shift to get current bit of sprite
                        let sprite_pixel = cur_row_sprite & (0x80 >> pixel) != 0;

                        if row_slice[pixel] & sprite_pixel {
                            flipped = true;
                        }

                        row_slice[pixel] = row_slice[pixel] ^ sprite_pixel;
                }

            }
        }

        if flipped { self.v[0xF] = 1} else { self.v[0xF] = 0 }
        self.inc_pc();
    }

    // Ex9E - SKP Vx -- Skip next instruction if key with value of Vx is pressed.
    // Checks the keyboard, and if the key corresponding to the value of
    // Vx is currently in the down position, the PC is incremented by two
    // (but since each instruction is manually incrementing pc, four)
    fn op_skp_vx(&mut self) {
        let key = self.v[self.get_x() as usize] as usize;
        if self.key_buff[key] {
            self.inc_pc();
        }
        self.inc_pc();
    }

    // ExA1 - SKNP Vx -- Skip next instruction if key with value of Vx is not pressed.
    // Checks the keyboard, and if the key corresponding to the value of
    // Vx is currently in the up position, the PC is incremented by two
    // (but since each instruction is manually incrementing pc, four)
    fn op_sknp_vx(&mut self) {
        let key = self.v[self.get_x() as usize] as usize;
        if !self.key_buff[key] {
            self.inc_pc();
        }
        self.inc_pc();
    }

    // Fx07 - LD Vx, DT -- Set Vx = delay timer value.
    // Value of DT is placed into Vx.
    fn op_ld_vx_dt(&mut self) {
        self.v[self.get_x() as usize] = self.delay_timer;
        self.inc_pc();
    }

    // Fx0A - LD Vx, K -- Wait for a key press, store the value of the key in Vx.
    // All execution stops until a key is pressed, then the value of that key is stored in Vx.
    fn op_ld_vx_k(&mut self) {
        let mut continue_exec = false;
        for (key, pressed) in self.key_buff.iter().enumerate() {
            if *pressed {
                self.v[self.get_x() as usize] = key as u8;
                continue_exec = true;
            }
        }
        if continue_exec {
            self.inc_pc();
        }
    }

    // Fx15 - LD DT, Vx -- Set delay timer = Vx
    // DT is set equal to the value of Vx.
    fn op_ld_dt_vx(&mut self) {
        self.delay_timer = self.v[self.get_x() as usize];
        self.inc_pc();
    }

    // Fx18 - LD ST, Vx -- Set sound timer = Vx
    // DT is set equal to the value of Vx.
    fn op_ld_st_vx(&mut self) {
        self.sound_timer = self.v[self.get_x() as usize];
        self.inc_pc();
    }

    // Fx29 - LD F, Vx -- Set I = location of sprite for digit Vx.
    // Value of I is set to location for hex sprite corresponding to value of
    // Vx.
    fn op_ld_f_vx(&mut self) {
        self.i = (self.v[self.get_x() as usize] * 5) as u16;
        self.inc_pc();
    }

    // Fx33 - LD, B, Vx -- Store BCD representation of Vx in memory locations I, 
    // I+1 and I+1.
    // Take the decimal value of Vx, place the hundreds digit in memory at location I,
    // the tens digit at I+1, and the ones digit at I+2.
    fn op_ld_b_vx(&mut self) {
        let vx = self.v[self.get_x() as usize];
        let i = self.i as usize;
        self.memory[i] = vx / 100;
        self.memory[i + 1] = (vx / 10) % 10;
        self.memory[i + 2] = (vx %100) %10;
        self.inc_pc();
    }

    // Fx55 - LD [I], Vx -- Store registers V0 through Vx in memory starting at location I.
    // The interpreter copies the values of registers V0 through Vx into memory, starting at
    // the address in I.
    fn op_ld_i_vx(&mut self) {
        let x = self.get_x() as u16;
        let i = self.i;
        for n in 0...x {
            self.memory[(i + n) as usize] = self.v[n as usize];
        }
        self.i = i + x + 1;
        self.inc_pc();
    }
    
    // Fx65 - LD Vx, [I] -- Read register V0 through Vx from memory starting @ I.
    // Reads values from memory starting at location I into register V0 through Vx.
    // Then set I to I + X + 1.
    fn op_ld_vx_i(&mut self) {
        let x = self.get_x() as u16;
        let i = self.i;
        for n in 0...x {
            self.v[n as usize] = self.memory[(i + n) as usize];
        }
        self.i = i + x + 1;
        self.inc_pc();
    }

    fn get_nnn(&self) -> u16 { self.opcode & 0x0fff }
    fn get_kk(&self) -> u8 { (self.opcode & 0x00ff) as u8 }
    fn get_x(&self) -> u8 { ((self.opcode & 0x0f00) >> 8) as u8 }
    fn get_y(&self) -> u8 { ((self.opcode & 0x00f0) >> 4) as u8 }
    fn get_n(&self) -> u8 { (self.opcode & 0x000f) as u8 }

}

static FONT_SPRITES: [u8; 80] = [0xF0, 0x90, 0x90, 0x90, 0xF0,  // 0
                                 0x20, 0x60, 0x20, 0x20, 0x70,  // 1
                                 0xF0, 0x10, 0xF0, 0x80, 0xF0,  // 2
                                 0xF0, 0x10, 0xF0, 0x10, 0xF0,  // 3
                                 0x90, 0x90, 0xF0, 0x10, 0x10,  // 4
                                 0xF0, 0x80, 0xF0, 0x10, 0xF0,  // 5
                                 0xF0, 0x80, 0xF0, 0x90, 0xF0,  // 6
                                 0xF0, 0x10, 0x20, 0x40, 0x40,  // 7
                                 0xF0, 0x90, 0xF0, 0x90, 0xF0,  // 8
                                 0xF0, 0x90, 0xF0, 0x10, 0xF0,  // 9
                                 0xF0, 0x90, 0xF0, 0x90, 0x90,  // A
                                 0xE0, 0x90, 0xE0, 0x90, 0xE0,  // B
                                 0xF0, 0x80, 0x80, 0x80, 0xF0,  // C
                                 0xE0, 0x90, 0x90, 0x90, 0xE0,  // D
                                 0xF0, 0x80, 0xF0, 0x80, 0xF0,  // E
                                 0xF0, 0x80, 0xF0, 0x80, 0x80]; // F

#[cfg(test)]
mod tests {
    use super::*;

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
        Cpu::load_data(&mut cpu, data);
        cpu.fetch_opcode();
        assert_eq!(257, cpu.opcode)
    }

    #[test]
    fn test_op_cls() {
        let mut cpu = Cpu::new();
        cpu.disp_buff = [[true; 64]; 32];
        cpu.op_cls();
        for i in 0..32 {
            for ii in 0..64 {
                assert_eq!(false, cpu.disp_buff[i][ii])
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
        Cpu::load_data(&mut cpu, vec![0x00, 0xE0, 0x23, 0x86]);
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
        Cpu::load_data(&mut cpu, vec![0x13, 0x86]);
        cpu.emulate_cycle();
        assert_eq!(cpu.pc, 0x386);
    }

    #[test]
    fn test_execute_call() {
        let mut cpu = Cpu::new();
        Cpu::load_data(&mut cpu, vec![0x00, 0xE0, 0x23, 0x86]);
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
        Cpu::load_data(&mut cpu, vec![0x33, 0x88]);
        cpu.emulate_cycle();
        assert_eq!(cpu.pc, 0x200 + 4);
    }

    #[test]
    fn test_se_if_false() {
        let mut cpu = Cpu::new();
        cpu.v[3] = 0x84;
        Cpu::load_data(&mut cpu, vec![0x33, 0x88]);
        cpu.emulate_cycle();
        assert_eq!(cpu.pc, 0x200 + 2);
    }

    #[test]
    fn test_sne_if_true() {
        let mut cpu = Cpu::new();
        cpu.v[3] = 0x84;
        Cpu::load_data(&mut cpu, vec![0x43, 0x88]);
        cpu.emulate_cycle();
        assert_eq!(cpu.pc, 0x200 + 4);
    }
    
    #[test]
    fn test_sne_if_false() {
        let mut cpu = Cpu::new();
        cpu.v[3] = 0x88;
        Cpu::load_data(&mut cpu, vec![0x43, 0x88]);
        cpu.emulate_cycle();
        assert_eq!(cpu.pc, 0x200 + 2);
    }

    #[test]
    fn test_se_vs_if_true() {
        let mut cpu = Cpu::new();
        cpu.v[3] = 0x88;
        cpu.v[6] = 0x88;
        Cpu::load_data(&mut cpu, vec![0x53, 0x60]);
        cpu.emulate_cycle();
        assert_eq!(cpu.pc, 0x200 + 4);
    }

    #[test]
    fn test_se_vs_if_false() {
        let mut cpu = Cpu::new();
        cpu.v[3] = 0x84;
        cpu.v[6] = 0x88;
        Cpu::load_data(&mut cpu, vec![0x33, 0x60]);
        cpu.emulate_cycle();
        assert_eq!(cpu.pc, 0x200 + 2);
    }

    #[test]
    fn test_ld_vx_byte() {
        let mut cpu = Cpu::new();
        Cpu::load_data(&mut cpu, vec![0x63, 0x92]);
        cpu.emulate_cycle();
        assert_eq!(cpu.v[3], 0x92);
    }

    #[test]
    fn test_add_vx_byte() {
        let mut cpu = Cpu::new();
        Cpu::load_data(&mut cpu, vec![0x73, 0x10]);
        cpu.v[3] = 0x70;
        cpu.emulate_cycle();
        assert_eq!(cpu.v[3], 0x10 + 0x70);
    }

    #[test]
    fn test_add_vx_byte_overflow() {
        let mut cpu = Cpu::new();
        Cpu::load_data(&mut cpu, vec![0x73, 0x01]);
        cpu.v[3] = 0xff;
        cpu.emulate_cycle();
        assert_eq!(cpu.v[3], 0);
    }

    #[test]
    fn test_ld_vx_vy() {
        let mut cpu = Cpu::new();
        Cpu::load_data(&mut cpu, vec![0x83, 0x70]);
        cpu.v[7] = 0x82;
        cpu.emulate_cycle();
        assert_eq!(cpu.v[3], 0x82);
    }

    #[test]
    fn test_or() {
        let mut cpu = Cpu::new();
        Cpu::load_data(&mut cpu, vec![0x80, 0xA1]);
        cpu.v[0]   = 0b10110011;
        cpu.v[0xA] = 0b01101001;
        //      OR = 0b11111011
        cpu.emulate_cycle();
        assert_eq!(cpu.v[0], 0b11111011);
    }

    #[test]
    fn test_and() {
        let mut cpu = Cpu::new();
        Cpu::load_data(&mut cpu, vec![0x8B, 0xA2]);
        cpu.v[0xB] = 0b10110011;
        cpu.v[0xA] = 0b01101001;
        //     AND = 0b00100001;
        cpu.emulate_cycle();
        assert_eq!(cpu.v[0xB], 0b00100001);
    }

    #[test]
    fn test_xor() {
        let mut cpu = Cpu::new();
        Cpu::load_data(&mut cpu, vec![0x8B, 0xA3]);
        cpu.v[0xB] = 0b10110011;
        cpu.v[0xA] = 0b01101001;
        //     XOR = 0b11011010;
        cpu.emulate_cycle();
        assert_eq!(cpu.v[0xB], 0b11011010);
    }

    #[test]
    fn test_add_vx_vy() {
        let mut cpu = Cpu::new();
        Cpu::load_data(&mut cpu, vec![0x8A, 0xB4]);
        cpu.v[0xA] = 0x5;
        cpu.v[0xB] = 0x3;
        cpu.emulate_cycle();
        assert_eq!(cpu.v[0xA], 0x8);
        assert_eq!(cpu.v[0xF], 0);
    }

    #[test]
    fn test_add_vx_vy_overflow() {
        let mut cpu = Cpu::new();
        Cpu::load_data(&mut cpu, vec![0x8A, 0xB4]);
        cpu.v[0xA] = 0xFF;
        cpu.v[0xB] = 0x1;
        cpu.emulate_cycle();
        assert_eq!(cpu.v[0xA], 0);
        assert_eq!(cpu.v[0xF], 1);
    }

    #[test]
    fn test_sub_vx_vy() {
        let mut cpu = Cpu::new();
        Cpu::load_data(&mut cpu, vec![0x8A, 0xB5]);
        cpu.v[0xA] = 5;
        cpu.v[0xB] = 1;
        cpu.emulate_cycle();
        assert_eq!(cpu.v[0xA], 4);
        assert_eq!(cpu.v[0xF], 1);
    }

    #[test]
    fn test_sub_vx_vy_overflow() {
        let mut cpu = Cpu::new();
        Cpu::load_data(&mut cpu, vec![0x8A, 0xB5]);
        cpu.v[0xA] = 0;
        cpu.v[0xB] = 1;
        cpu.emulate_cycle();
        assert_eq!(cpu.v[0xA], 255);
        assert_eq!(cpu.v[0xF], 0);
    }

    #[test]
    fn test_shr_shift_x() {
        let mut cpu = Cpu::new();
        Cpu::load_data(&mut cpu, vec![0x84, 0x46]);
        cpu.v[4] = 0b11;
        cpu.emulate_cycle();
        assert_eq!(cpu.v[4], 1);
    }

    #[test]
    fn test_shr_shift_y_to_x() {
        let mut cpu = Cpu::new();
        Cpu::load_data(&mut cpu, vec![0x84, 0x56]);
        cpu.v[5] = 0b10;
        cpu.emulate_cycle();
        assert_eq!(cpu.v[4], 1);
    }

    #[test]
    fn test_shr_carry_0() {
        let mut cpu = Cpu::new();
        Cpu::load_data(&mut cpu, vec![0x84, 0x46]);
        cpu.v[4] = 0b10;
        cpu.emulate_cycle();
        assert_eq!(cpu.v[0xf], 0);
    }

    #[test]
    fn test_shr_carry_1() {
        let mut cpu = Cpu::new();
        Cpu::load_data(&mut cpu, vec![0x84, 0x46]);
        cpu.v[4] = 0b11;
        cpu.emulate_cycle();
        assert_eq!(cpu.v[0xf], 1);
    }

    #[test]
    fn test_subn_vx_vy() {
        let mut cpu = Cpu::new();
        Cpu::load_data(&mut cpu, vec![0x84, 0x57]);
        cpu.v[0x4] = 1;
        cpu.v[0x5] = 5;
        cpu.emulate_cycle();
        assert_eq!(cpu.v[0x4], 4);
        assert_eq!(cpu.v[0xF], 1);
    }

    #[test]
    fn test_subn_vx_vy_overflow() {
        let mut cpu = Cpu::new();
        Cpu::load_data(&mut cpu, vec![0x84, 0x57]);
        cpu.v[0x4] = 1;
        cpu.v[0x5] = 0;
        cpu.emulate_cycle();
        assert_eq!(cpu.v[0x4], 255);
        assert_eq!(cpu.v[0xF], 0);
    }

    #[test]
    fn test_shl_shift_x() {
        let mut cpu = Cpu::new();
        Cpu::load_data(&mut cpu, vec![0x84, 0x4E]);
        cpu.v[4] = 0b01;
        cpu.emulate_cycle();
        assert_eq!(cpu.v[4], 0b10);
    }

    #[test]
    fn test_shl_shift_y_to_x() {
        let mut cpu = Cpu::new();
        Cpu::load_data(&mut cpu, vec![0x84, 0x5E]);
        cpu.v[5] = 0b10;
        cpu.emulate_cycle();
        assert_eq!(cpu.v[4], 0b100);
    }

    #[test]
    fn test_shl_carry_0() {
        let mut cpu = Cpu::new();
        Cpu::load_data(&mut cpu, vec![0x84, 0x4E]);
        cpu.v[4] = 0b1;
        cpu.emulate_cycle();
        assert_eq!(cpu.v[0xf], 0);
    }

    #[test]
    fn test_shl_carry_1() {
        let mut cpu = Cpu::new();
        Cpu::load_data(&mut cpu, vec![0x84, 0x4E]);
        cpu.v[4] = 0b11000000;
        cpu.emulate_cycle();
        assert_eq!(cpu.v[0xf], 1);
    }

    #[test]
    fn test_sne_vx_vy_if_equal() {
        let mut cpu = Cpu::new();
        Cpu::load_data(&mut cpu, vec![0x93, 0x40]);
        cpu.v[3] = 0x88;
        cpu.v[4] = 0x88;
        cpu.emulate_cycle();
        assert_eq!(cpu.pc, 0x200 + 2);
    }

    #[test]
    fn test_sne_vx_vy_if_not_equal() {
        let mut cpu = Cpu::new();
        Cpu::load_data(&mut cpu, vec![0x93, 0x40]);
        cpu.v[3] = 0x88;
        cpu.v[4] = 0x87;
        cpu.emulate_cycle();
        assert_eq!(cpu.pc, 0x200 + 4);
    }

    #[test]
    fn test_ld_i_addr() {
        let mut cpu = Cpu::new();
        Cpu::load_data(&mut cpu, vec![0xAA, 0xAA]);
        cpu.emulate_cycle();
        assert_eq!(cpu.i, 0xAAA);
    }

    #[test]
    fn test_op_jp_v0_addr() {
        let mut cpu = Cpu::new();
        Cpu::load_data(&mut cpu, vec![0xB3, 0x86]);
        cpu.v[0] = 0x25;
        cpu.emulate_cycle();
        assert_eq!(cpu.pc, 0x386 + 0x25);
    }

    #[test]
    fn test_rnd_vx_byte_masks_binary() {
        let mut cpu = Cpu::new();
        Cpu::load_data(&mut cpu, vec![0xC3, 0x01]);
        cpu.v[3] = 2;
        cpu.emulate_cycle();
        assert!(cpu.v[3] <= 1);
    }

    // #[test]
    // fn test_drw_vx_vy_n() {
    //     let mut cpu = Cpu::new();
    //     Cpu::load_data(&mut cpu, vec![0x62, 0x02, 0x63, 0x03, 0xF3, 0x29, 0xD2, 0x35]);
    //     for _ in 0..4 {
    //         cpu.emulate_cycle();
    //     }
    //     let mut expected = [[false; 64]; 32];
    //     // put a 3 into the mock display buffer
    //     // starting on (2, 3)
    //     expected[3][2] = true;
    //     expected[3][3] = true;
    //     expected[3][4] = true;
    //     expected[3][5] = true;
    //     expected[4][2] = false;
    //     expected[4][3] = false;
    //     expected[4][4] = false;
    //     expected[4][5] = true;
    //     expected[5][2] = true;
    //     expected[5][3] = true;
    //     expected[5][4] = true;
    //     expected[5][5] = true;
    //     expected[6][2] = false;
    //     expected[6][3] = false;
    //     expected[6][4] = false;
    //     expected[6][5] = true;
    //     expected[7][2] = true;
    //     expected[7][3] = true;
    //     expected[7][4] = true;
    //     expected[7][5] = true;
    //     use display::Display;
    //     let display = Display::new(cpu.disp_buff);
    //     println!("Display:");
    //     display.render_to_raw_terminal();
    //     for i in 0..32 {
    //         for ii in 0..64 {
    //             assert_eq!(cpu.disp_buff[i][ii], expected[i][ii]);
    //         }
    //     }
    // }

    #[test]
    fn test_drw_vx_vy_n_erases() {
        let mut cpu = Cpu::new();
        Cpu::load_data(&mut cpu, vec![0x62, 0x02, 0x63, 0x03, 0xF3, 0x29, 0xD2, 0x35,
                                      0xD2, 0x35]);
        for _ in 0..5 {
            cpu.emulate_cycle();
        }

        let empty_disp = [[false; 64]; 32];
        for i in 0..32 {
            for ii in 0..64 {
                assert_eq!(cpu.disp_buff[i][ii], empty_disp[i][ii]);
            }
        }
    }

    #[test]
    fn test_drw_vx_vy_n_sets_flip() {
        let mut cpu = Cpu::new();
        Cpu::load_data(&mut cpu, vec![0x62, 0x02, 0x63, 0x03, 0xF3, 0x29, 0xD2, 0x35,
                                      0xD2, 0x35]);
        for _ in 0..5 {
            cpu.emulate_cycle();
        }

        assert_eq!(cpu.v[0xF], 1);
    }

    #[test]
    fn test_drw_vx_vy_n_unsets_flip() {
        let mut cpu = Cpu::new();
        Cpu::load_data(&mut cpu, vec![0x62, 0x02, 0x63, 0x03, 0xF3, 0x29, 0xD2, 0x35,
                                      0xD2, 0x35, 0xD2, 0x35]);
        for _ in 0..6 {
            cpu.emulate_cycle();
        }

        assert_eq!(cpu.v[0xF], 0);
    }

    // #[test]
    // fn test_drw_vx_vy_n_clips() {
    //     let mut cpu = Cpu::new();
    //     Cpu::load_data(&mut cpu, vec![0x62, 0x3F, 0x63, 0x1F, 0xF0, 0x29, 0xD2, 0x35]);
    //     for _ in 0..4 {
    //         cpu.emulate_cycle();
    //     }
    //     let mut expected = [[false; 64]; 32];
    //     expected[31][63] = true;
    //     use display::Display;
    //     let display = Display::new(cpu.disp_buff);
    //     println!("Display:");
    //     display.render_to_raw_terminal();
    //     println!("Display end");
    //     for i in 0..32 {
    //         for ii in 0..64 {
    //             assert_eq!(cpu.disp_buff[i][ii], expected[i][ii]);
    //         }
    //     }
    // }

    // #[test]
    // fn test_ld_f_vx_0() {
    //     let mut cpu = Cpu::new();
    //     Cpu::load_data(&mut cpu, vec![0x61, 0x00, 0xF1, 0x29, 0xD0, 0x05]);
    //     for _ in 0..3 {
    //         cpu.emulate_cycle();
    //     }
    //     let mut expected = [[false; 64]; 32];
    //     expected[0][0] = true;
    //     expected[0][1] = true;
    //     expected[0][2] = true;
    //     expected[0][3] = true;
    //     expected[1][0] = true;
    //     expected[1][1] = false;
    //     expected[1][2] = false;
    //     expected[1][3] = true;
    //     expected[2][0] = true;
    //     expected[2][1] = false;
    //     expected[2][2] = false;
    //     expected[2][3] = true;
    //     expected[3][0] = true;
    //     expected[3][1] = false;
    //     expected[3][2] = false;
    //     expected[3][3] = true;
    //     expected[4][0] = true;
    //     expected[4][1] = true;
    //     expected[4][2] = true;
    //     expected[4][3] = true;
    //     use display::Display;
    //     let display = Display::new(cpu.disp_buff);
    //     println!("Display:");
    //     display.render_to_raw_terminal();
    //     for i in 0..32 {
    //         for ii in 0..64 {
    //             assert_eq!(cpu.disp_buff[i][ii], expected[i][ii]);
    //         }
    //     }
    // }


    // #[test]
    // fn test_ld_f_vx_1() {
    //     let mut cpu = Cpu::new();
    //     Cpu::load_data(&mut cpu, vec![0x61, 0x01, 0xF1, 0x29, 0xD0, 0x05]);
    //     for _ in 0..3 {
    //         cpu.emulate_cycle();
    //     }
    //     let mut expected = [[false; 64]; 32];
    //     expected[0][0] = false;
    //     expected[0][1] = false;
    //     expected[0][2] = true;
    //     expected[0][3] = false;
    //     expected[1][0] = false;
    //     expected[1][1] = true;
    //     expected[1][2] = true;
    //     expected[1][3] = false;
    //     expected[2][0] = false;
    //     expected[2][1] = false;
    //     expected[2][2] = true;
    //     expected[2][3] = false;
    //     expected[3][0] = false;
    //     expected[3][1] = false;
    //     expected[3][2] = true;
    //     expected[3][3] = false;
    //     expected[4][0] = false;
    //     expected[4][1] = true;
    //     expected[4][2] = true;
    //     expected[4][3] = true;
    //     use display::Display;
    //     let display = Display::new(cpu.disp_buff);
    //     println!("Display:");
    //     display.render_to_raw_terminal();
    //     for i in 0..32 {
    //         for ii in 0..64 {
    //             assert_eq!(cpu.disp_buff[i][ii], expected[i][ii]);
    //         }
    //     }
    // }

    // I should test the whole font set? But I'm confident it works at this point.

    #[test]
    fn test_timers() {
        use std::thread;
        let mut cpu = Cpu::new();
        Cpu::load_data(&mut cpu, vec![0x61, 0x01, 0x61, 0x01]);
        cpu.delay_timer = 120;
        use std::{time};
        let millis_18 = time::Duration::from_millis(18);
        thread::sleep(millis_18);
        cpu.emulate_cycle();
        assert_eq!(cpu.delay_timer, 119);
    }

    #[test]
    fn test_skp_vx_if_pressed() {
        let mut cpu = Cpu::new();
        Cpu::load_data(&mut cpu, vec![0xE0, 0x9E]);
        cpu.key_buff[0] = true;
        cpu.emulate_cycle();
        assert_eq!(cpu.pc, 0x204);
    }
    
    #[test]
    fn test_skp_vx_if_not_pressed() {
        let mut cpu = Cpu::new();
        Cpu::load_data(&mut cpu, vec![0xE0, 0x9E]);
        cpu.emulate_cycle();
        assert_eq!(cpu.pc, 0x202);
    }

    #[test]
    fn test_sknp_vx_if_pressed() {
        let mut cpu = Cpu::new();
        Cpu::load_data(&mut cpu, vec![0xE0, 0xA1]);
        cpu.key_buff[0] = true;
        cpu.emulate_cycle();
        assert_eq!(cpu.pc, 0x202);
    }
    
    #[test]
    fn test_sknp_vx_if_not_pressed() {
        let mut cpu = Cpu::new();
        Cpu::load_data(&mut cpu, vec![0xE0, 0xA1]);
        cpu.emulate_cycle();
        assert_eq!(cpu.pc, 0x204);
    }

    #[test]
    fn test_ld_b_vx() {
        let mut cpu = Cpu::new();
        cpu.i = 0x500;
        cpu.v[0] = 136;
        Cpu::load_data(&mut cpu, vec![0xF0, 0x33]);
        cpu.emulate_cycle();
        println!("");
        println!("0x500: {}", cpu.memory[0x500]);
        println!("0x501: {}", cpu.memory[0x501]);
        println!("0x502: {}", cpu.memory[0x502]);
        assert_eq!(cpu.memory[0x500], 1);
        assert_eq!(cpu.memory[0x501], 3);
        assert_eq!(cpu.memory[0x502], 6);
    }

    #[test]
    fn test_ld_vx_i() {
        let mut cpu = Cpu::new();
        cpu.i = 0x500;
        cpu.memory[0x500] = 0;
        cpu.memory[0x501] = 1;
        cpu.memory[0x502] = 2;
        cpu.memory[0x503] = 3;
        Cpu::load_data(&mut cpu, vec![0xF3, 0x65]);
        cpu.emulate_cycle();
        assert_eq!(cpu.v[0], 0);
        assert_eq!(cpu.v[1], 1);
        assert_eq!(cpu.v[2], 2);
        assert_eq!(cpu.v[3], 3);
    }

    #[test]
    fn test_ld_i_vx() {
        let mut cpu = Cpu::new();
        cpu.i = 0x500;
        cpu.v[0] = 0;
        cpu.v[1] = 1;
        cpu.v[2] = 2;
        Cpu::load_data(&mut cpu, vec![0xF2, 0x55]);
        cpu.emulate_cycle();
        assert_eq!(cpu.memory[0x500], 0);
        assert_eq!(cpu.memory[0x501], 1);
        assert_eq!(cpu.memory[0x502], 2);
    }

    #[test]
    fn test_ld_dt_vx() {
        let mut cpu = Cpu::new();
        cpu.v[3] = 0x20;
        Cpu::load_data(&mut cpu, vec![0xF3, 0x15]);
        cpu.emulate_cycle();
        assert_eq!(cpu.delay_timer, 0x20);
    }

    #[test]
    fn test_ld_vx_k() {
        let cpu = &mut Cpu::new();
        Cpu::load_data(cpu, vec![0xFF, 0x0A]);
        cpu.emulate_cycle();
        assert_eq!(cpu.pc, 0x200);
        cpu.key_buff[3] = true;
        cpu.emulate_cycle();
        assert_eq!(cpu.v[0xF], 3);
    }
}
