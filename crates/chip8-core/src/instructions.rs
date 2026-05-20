use crate::{Chip8, FLAG_REGISTER, FONT_OFFSET, HEIGHT, SPRITE_WIDTH, WIDTH};

use rand::RngExt;

pub enum Instruction {
    Clear,
    Return,
    Jump { nnn: u16 },
    Call { nnn: u16 },
    SkipEq { x: u8, nn: u8 },
    SkipNEq { x: u8, nn: u8 },
    SkipEqVy { x: u8, y: u8 },
    Load { x: u8, nn: u8 },
    Add { x: u8, nn: u8 },
    LoadVy { x: u8, y: u8 },
    Or { x: u8, y: u8 },
    And { x: u8, y: u8 },
    Xor { x: u8, y: u8 },
    AddVy { x: u8, y: u8 },
    SubVy { x: u8, y: u8 },
    ShiftR { x: u8 },
    SubVyVx { x: u8, y: u8 },
    ShiftL { x: u8 },
    SkipNEqVy { x: u8, y: u8 },
    LoadI { nnn: u16 },
    JumpV0 { nnn: u16 },
    Rand { x: u8, nn: u8 },
    Draw { x: u8, y: u8, n: u8 },
    SkipKeyPress { x: u8 },
    SkipKeyNPress { x: u8 },
    GetDelayTimer { x: u8 },
    WaitKey { x: u8 },
    SetDelayTimer { x: u8 },
    SetSoundTimer { x: u8 },
    AddI { x: u8 },
    LoadFont { x: u8 },
    Bcd { x: u8 },
    Store { x: u8 },
    Fill { x: u8 },
    Unknown(u16),
}

impl Chip8 {
    pub(super) fn execute(&mut self, instruction: Instruction) {
        match instruction {
            Instruction::Clear => self.op_00e0(),
            Instruction::Return => self.op_00ee(),
            Instruction::Jump { nnn } => self.op_1nnn(nnn),
            Instruction::Call { nnn } => self.op_2nnn(nnn),
            Instruction::SkipEq { x, nn } => self.op_3xnn(x, nn),
            Instruction::SkipNEq { x, nn } => self.op_4xnn(x, nn),
            Instruction::SkipEqVy { x, y } => self.op_5xy0(x, y),
            Instruction::Load { x, nn } => self.op_6xnn(x, nn),
            Instruction::Add { x, nn } => self.op_7xnn(x, nn),
            Instruction::LoadVy { x, y } => self.op_8xy0(x, y),
            Instruction::Or { x, y } => self.op_8xy1(x, y),
            Instruction::And { x, y } => self.op_8xy2(x, y),
            Instruction::Xor { x, y } => self.op_8xy3(x, y),
            Instruction::AddVy { x, y } => self.op_8xy4(x, y),
            Instruction::SubVy { x, y } => self.op_8xy5(x, y),
            Instruction::ShiftR { x } => self.op_8xy6(x),
            Instruction::SubVyVx { x, y } => self.op_8xy7(x, y),
            Instruction::ShiftL { x } => self.op_8xye(x),
            Instruction::SkipNEqVy { x, y } => self.op_9xy0(x, y),
            Instruction::LoadI { nnn } => self.op_annn(nnn),
            Instruction::JumpV0 { nnn } => self.op_bnnn(nnn),
            Instruction::Rand { x, nn } => self.op_cxnn(x, nn),
            Instruction::Draw { x, y, n } => self.op_dxyn(x, y, n),
            Instruction::SkipKeyPress { x } => self.op_ex9e(x),
            Instruction::SkipKeyNPress { x } => self.op_exa1(x),
            Instruction::GetDelayTimer { x } => self.op_fx07(x),
            Instruction::WaitKey { x } => self.op_fx0a(x),
            Instruction::SetDelayTimer { x } => self.op_fx15(x),
            Instruction::SetSoundTimer { x } => self.op_fx18(x),
            Instruction::AddI { x } => self.op_fx1e(x),
            Instruction::LoadFont { x } => self.op_fx29(x),
            Instruction::Bcd { x } => self.op_fx33(x),
            Instruction::Store { x } => self.op_fx55(x),
            Instruction::Fill { x } => self.op_fx65(x),
            Instruction::Unknown(opcode) => eprintln!("unknown opcode: {:#06x}", opcode),
        }
    }

    // clear screen
    fn op_00e0(&mut self) {
        self.frame_buffer.fill(0);
    } // Clear

    // exit subroutine
    fn op_00ee(&mut self) {
        self.pc = self
            .stack
            .pop()
            .expect("Returned from subroutine with empty stack");
    } // Return

    // jump
    fn op_1nnn(&mut self, nnn: u16) {
        self.pc = nnn;
    } // Jump

    // enter subroutine
    fn op_2nnn(&mut self, nnn: u16) {
        self.stack.push(self.pc);
        self.pc = nnn;
    } // Call

    // skip one instruction iff vx == nn
    fn op_3xnn(&mut self, x: u8, nn: u8) {
        if self.registers[x as usize] == nn {
            self.pc += 2;
        }
    } // SkipEq

    // skip one instruction iff vx != nn
    fn op_4xnn(&mut self, x: u8, nn: u8) {
        if self.registers[x as usize] != nn {
            self.pc += 2;
        }
    } // SkipNEq

    // skip one instruction iff vx == vy
    fn op_5xy0(&mut self, x: u8, y: u8) {
        if self.registers[x as usize] == self.registers[y as usize] {
            self.pc += 2;
        }
    } // SkipEqVy

    // sets vx to nn
    fn op_6xnn(&mut self, x: u8, nn: u8) {
        self.registers[x as usize] = nn;
    } // Load

    // adds nn to vx (carry flag is not changed)
    fn op_7xnn(&mut self, x: u8, nn: u8) {
        let x = x as usize;
        self.registers[x] = self.registers[x].wrapping_add(nn);
    } // Add

    // sets vx to the value of vy
    fn op_8xy0(&mut self, x: u8, y: u8) {
        self.registers[x as usize] = self.registers[y as usize];
    } // LoadVy

    // sets vx to vx | vy
    fn op_8xy1(&mut self, x: u8, y: u8) {
        self.registers[x as usize] |= self.registers[y as usize];
    } // Or

    // sets vx to vx & vy
    fn op_8xy2(&mut self, x: u8, y: u8) {
        self.registers[x as usize] &= self.registers[y as usize];
    } // And

    // sets vx to vx ^ vy
    fn op_8xy3(&mut self, x: u8, y: u8) {
        self.registers[x as usize] ^= self.registers[y as usize];
    } // Xor

    // adds vy to vx. vf is set to 1 when there's an overflow, and to 0 when there's not
    fn op_8xy4(&mut self, x: u8, y: u8) {
        let x = x as usize;
        let (val, carry) = self.registers[x].overflowing_add(self.registers[y as usize]);
        self.registers[FLAG_REGISTER] = carry as u8;
        self.registers[x] = val;
    } // AddVy

    // vy is subtracted from vx. vf is set to 0 when there's an underflow, and 1 when there is not (ie vf is set to 1 if VX >= vy and 0 if not)
    fn op_8xy5(&mut self, x: u8, y: u8) {
        let x = x as usize;
        let (val, carry) = self.registers[x].overflowing_sub(self.registers[y as usize]);
        self.registers[FLAG_REGISTER] = !carry as u8;
        self.registers[x] = val;
    } // SubVy

    // shfits vx to the right by 1, then stores the least significant bit of vx prior to the shift into vf
    fn op_8xy6(&mut self, x: u8) {
        let x = x as usize;
        self.registers[FLAG_REGISTER] = self.registers[x] & 0x01;
        self.registers[x] >>= 1;
    } // ShiftR

    // sets vx to vy minus vx. vf is set to 0 when there's an underflow, and 1 when there's not
    fn op_8xy7(&mut self, x: u8, y: u8) {
        let x = x as usize;
        let (val, carry) = self.registers[y as usize].overflowing_sub(self.registers[x]);
        self.registers[FLAG_REGISTER] = !carry as u8;
        self.registers[x] = val;
    } // SubVyVx

    // shifts vx to the left by 1, then sets vf to 1 if the most significant bit of vx prior to that shift was set, or to 0 if it was unset
    fn op_8xye(&mut self, x: u8) {
        let x = x as usize;
        self.registers[FLAG_REGISTER] = self.registers[x] >> 7;
        self.registers[x] <<= 1;
    } // ShiftL

    // skip one instruction iff vx != vy
    fn op_9xy0(&mut self, x: u8, y: u8) {
        if self.registers[x as usize] != self.registers[y as usize] {
            self.pc += 2;
        }
    } // SkipNEqVyVx

    // sets I to the address nnn
    fn op_annn(&mut self, nnn: u16) {
        self.index = nnn;
    } // LoadI

    // jumps to the address nnn plus v0
    fn op_bnnn(&mut self, nnn: u16) {
        self.pc = nnn + u16::from(self.registers[0]);
    } // JumpV0

    // sets vx to the results of a bitwise and operation on a random number (typically 0 to 255) and nn
    fn op_cxnn(&mut self, x: u8, nn: u8) {
        self.registers[x as usize] = nn & self.rng.random::<u8>();
    } // Rand

    // draws a sprite at coordinate (vx, vy) that has a width of 8 pixels and a height of N pixels.
    // each row of 8 pixels is read as bit-coded starting from memory location I;
    // I value does not change after the execution of this instruction.
    // as described above, vf is set to 1 if any screen pixels are flipped
    // from set to unset when the sprite is drawn, and to 0 if that does not happen

    // pixel data: memory[index + i] N u8 integers
    // display data: frame_buffer[y * WIDTH + x]
    #[allow(non_snake_case)]
    fn op_dxyn(&mut self, x: u8, y: u8, N: u8) {
        let x = x as usize;
        let y = y as usize;
        let index = self.index as usize;
        self.registers[FLAG_REGISTER] = 0;

        for (y, i) in (self.registers[y] % HEIGHT as u8..).zip(0..N) {
            let pixel_data = self.memory[index + i as usize];
            if y > HEIGHT as u8 - 1 {
                break;
            }
            for (x, j) in (self.registers[x] % WIDTH as u8..).zip(0..SPRITE_WIDTH) {
                if x > WIDTH as u8 - 1 {
                    break;
                }
                let frame_buffer_index = self.pixel_coords(x, y);
                let frame_buffer_pixel = self.frame_buffer[frame_buffer_index];
                let sprite_pixel = (pixel_data >> (7 - j)) & 0x01;
                if self.registers[FLAG_REGISTER] == 0
                    && sprite_pixel == 1
                    && frame_buffer_pixel == 1
                {
                    self.registers[FLAG_REGISTER] = 1;
                }
                self.frame_buffer[frame_buffer_index] = sprite_pixel ^ frame_buffer_pixel;
            }
        }
    } // Draw

    // skips the next instruction if the key stored in vx (only consider the lowest nibble) is pressed
    fn op_ex9e(&mut self, x: u8) {
        if self.keypad[(self.registers[x as usize] & 0x0f) as usize] > 0 {
            self.pc += 2;
        }
    } // SkipKeyPress

    // skips the next instruciton if the key stored in vx (only consider the lowest nibble) is not pressed
    fn op_exa1(&mut self, x: u8) {
        if self.keypad[(self.registers[x as usize] & 0x0f) as usize] == 0 {
            self.pc += 2;
        }
    } // SkipKeyNPress

    // sets vx to the value of the delay timer
    fn op_fx07(&mut self, x: u8) {
        self.registers[x as usize] = self.delay_timer;
    } // SetVxToDTimer

    // a key press is awaited, and then stored in vx (blocking operation, all instruction is halted until next key event)
    // delay and sound timers should continue processing
    fn op_fx0a(&mut self, x: u8) {
        for (index, &key) in self.keypad.iter().enumerate() {
            if key != 0 && self.prev_keys[index] == 0 {
                self.registers[x as usize] = index as u8;
                return;
            }
        }
        self.pc -= 2;
    } // LoadKey

    // sets the delay timer to vx
    fn op_fx15(&mut self, x: u8) {
        self.delay_timer = self.registers[x as usize];
    } // LoadDTimer

    // sets the sound timer to vx
    fn op_fx18(&mut self, x: u8) {
        self.sound_timer = self.registers[x as usize];
    } // LoadSTimer

    // adds vx to I. vf is not affected
    fn op_fx1e(&mut self, x: u8) {
        self.index += u16::from(self.registers[x as usize]);
    } // AddI

    // sets I to the location of the sprite for the character in vx (only consider the lowest nibble).
    // characters 0-f (in hex) are represnted by a 4x5 font
    fn op_fx29(&mut self, x: u8) {
        let nibble = self.registers[x as usize] & 0x0f;
        self.index = FONT_OFFSET as u16 + u16::from(nibble * 5);
    } // SetI

    // stores the binary-coded decimal representation of vx, with the hundreds digit in memory at location in I
    // the tens digit at location I + 1, and the ones digit at location I + 2.
    fn op_fx33(&mut self, x: u8) {
        // 034
        let mut temp = self.registers[x as usize];
        for i in (0..3).rev() {
            self.memory[self.index as usize + i] = temp % 10;
            temp /= 10;
        }
    } // Bcd

    // stores from v0..=vx in memory, starting at address I. The offset from I is increased by 1 for each value written,
    // but I itself is left unmodified
    fn op_fx55(&mut self, x: u8) {
        let base = self.index as usize;
        for i in 0..=x {
            self.memory[base + i as usize] = self.registers[i as usize];
        }
    } // Store

    // fills from v0..=vx with values from memory, starting at address I. The offset from I is increased by 1 for each value read,
    // but I itself is left unmodified
    fn op_fx65(&mut self, x: u8) {
        for i in 0..=x {
            self.registers[i as usize] = self.memory[(self.index + i as u16) as usize];
        }
    } // Fill
} // impl Chip8
