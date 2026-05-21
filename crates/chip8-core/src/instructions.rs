use crate::{Chip8, FLAG_REGISTER, FONT_OFFSET, HEIGHT, SPRITE_WIDTH, WIDTH};

use rand::RngExt;

#[allow(non_snake_case)]
pub(super) struct Instruction {
    pub(super) group: u8,
    pub(super) x: u8,
    pub(super) y: u8,
    pub(super) N: u8,
    pub(super) kk: u8,
    pub(super) nnn: u16,
}

impl Chip8 {
    // todo: error handling
    pub(super) fn execute(&mut self, instruction: Instruction) {
        let Instruction {
            group,
            x,
            y,
            N,
            kk,
            nnn,
        } = instruction;

        match (group, x, y, N) {
            (0x0, 0x0, 0xE, 0x0) => self.cls(),
            (0x0, 0x0, 0xE, 0xE) => self.ret(),
            (0x1, _, _, _) => self.jp(nnn),
            (0x2, _, _, _) => self.call(nnn),
            (0x3, _, _, _) => self.se(x, kk),
            (0x4, _, _, _) => self.sne(x, kk),
            (0x5, _, _, 0x0) => self.sne_vy(x, y),
            (0x6, _, _, _) => self.ld_kk(x, kk),
            (0x7, _, _, _) => self.add(x, kk),
            (0x8, _, _, 0x0) => self.ld_vy(x, y),
            (0x8, _, _, 0x1) => self.or(x, y),
            (0x8, _, _, 0x2) => self.and(x, y),
            (0x8, _, _, 0x3) => self.xor(x, y),
            (0x8, _, _, 0x4) => self.add_vy(x, y),
            (0x8, _, _, 0x5) => self.sub_vy(x, y),
            (0x8, _, _, 0x6) => self.shr(x),
            (0x8, _, _, 0x7) => self.subn(x, y),
            (0x8, _, _, 0xE) => self.shl(x),
            (0x9, _, _, 0x0) => self.sne_vy(x, y),
            (0xA, _, _, _) => self.ld_i(nnn),
            (0xB, _, _, _) => self.jp_v0(nnn),
            (0xC, _, _, _) => self.rnd(x, kk),
            (0xD, _, _, _) => self.drw(x, y, N),
            (0xE, _, 0x9, 0xE) => self.skp(x),
            (0xE, _, 0xA, 0x1) => self.sknp(x),
            (0xF, _, 0x0, 0x7) => self.ldf_delay(x),
            (0xF, _, 0x0, 0xA) => self.ldf_key(x),
            (0xF, _, 0x1, 0x5) => self.ld_delay(x),
            (0xF, _, 0x1, 0x8) => self.ld_sound(x),
            (0xF, _, 0x1, 0xE) => self.add_i(x),
            (0xF, _, 0x2, 0x9) => self.ld_i_sprite(x),
            (0xF, _, 0x3, 0x3) => self.ld_bcd(x),
            (0xF, _, 0x5, 0x5) => self.ldf_registers(x),
            (0xF, _, 0x6, 0x5) => self.ld_registers(x),
            _ => panic!("Unknown opcode"),
        }
    }

    // clear screen
    fn cls(&mut self) {
        self.frame_buffer.clear();
    } // Clear

    // exit subroutine
    fn ret(&mut self) {
        if self.stack_ptr < 1 {
            panic!("Stack underflow")
        }
        self.stack_ptr -= 1;
        self.pc = self.stack[self.stack_ptr as usize];
    } // Return

    // jump
    fn jp(&mut self, nnn: u16) {
        self.pc = nnn;
    } // Jump

    // enter subroutine
    fn call(&mut self, nnn: u16) {
        if self.stack_ptr > 15 {
            panic!("Stack overflow")
        }
        self.stack[self.stack_ptr as usize] = self.pc;
        self.stack_ptr += 1;
        self.pc = nnn;
    } // Call

    // skip one instruction iff vx == kk
    fn se(&mut self, x: u8, kk: u8) {
        if self.registers[x as usize] == kk {
            self.pc += 2;
        }
    } // SkipEq

    // skip one instruction iff vx != kk
    fn sne(&mut self, x: u8, kk: u8) {
        if self.registers[x as usize] != kk {
            self.pc += 2;
        }
    } // SkipNEq

    // skip one instruction iff vx == vy
    fn se_vy(&mut self, x: u8, y: u8) {
        if self.registers[x as usize] == self.registers[y as usize] {
            self.pc += 2;
        }
    } // SkipEqVy

    // sets vx to kk
    fn ld_kk(&mut self, x: u8, kk: u8) {
        self.registers[x as usize] = kk;
    } // Load

    // adds kk to vx (carry flag is not changed)
    fn add(&mut self, x: u8, kk: u8) {
        let x = x as usize;
        self.registers[x] = self.registers[x].wrapping_add(kk);
    } // Add

    // sets vx to the value of vy
    fn ld_vy(&mut self, x: u8, y: u8) {
        self.registers[x as usize] = self.registers[y as usize];
    } // LoadVy

    // sets vx to vx | vy
    fn or(&mut self, x: u8, y: u8) {
        self.registers[x as usize] |= self.registers[y as usize];
    } // Or

    // sets vx to vx & vy
    fn and(&mut self, x: u8, y: u8) {
        self.registers[x as usize] &= self.registers[y as usize];
    } // And

    // sets vx to vx ^ vy
    fn xor(&mut self, x: u8, y: u8) {
        self.registers[x as usize] ^= self.registers[y as usize];
    } // Xor

    // adds vy to vx. vf is set to 1 when there's an overflow, and to 0 when there's not
    fn add_vy(&mut self, x: u8, y: u8) {
        let x = x as usize;
        let (val, carry) = self.registers[x].overflowing_add(self.registers[y as usize]);
        self.registers[FLAG_REGISTER] = carry as u8;
        self.registers[x] = val;
    } // AddVy

    // vy is subtracted from vx. vf is set to 0 when there's an underflow, and 1 when there is not (ie vf is set to 1 if VX >= vy and 0 if not)
    fn sub_vy(&mut self, x: u8, y: u8) {
        let x = x as usize;
        let (val, carry) = self.registers[x].overflowing_sub(self.registers[y as usize]);
        self.registers[FLAG_REGISTER] = !carry as u8;
        self.registers[x] = val;
    } // SubVy

    // shfits vx to the right by 1, then stores the least significant bit of vx prior to the shift into vf
    fn shr(&mut self, x: u8) {
        let x = x as usize;
        self.registers[FLAG_REGISTER] = self.registers[x] & 0x01;
        self.registers[x] >>= 1;
    } // ShiftR

    // sets vx to vy minus vx. vf is set to 0 when there's an underflow, and 1 when there's not
    fn subn(&mut self, x: u8, y: u8) {
        let x = x as usize;
        let (val, carry) = self.registers[y as usize].overflowing_sub(self.registers[x]);
        self.registers[FLAG_REGISTER] = !carry as u8;
        self.registers[x] = val;
    } // SubVyVx

    // shifts vx to the left by 1, then sets vf to 1 if the most significant bit of vx prior to that shift was set, or to 0 if it was unset
    fn shl(&mut self, x: u8) {
        let x = x as usize;
        self.registers[FLAG_REGISTER] = self.registers[x] >> 7;
        self.registers[x] <<= 1;
    } // ShiftL

    // skip one instruction iff vx != vy
    fn sne_vy(&mut self, x: u8, y: u8) {
        if self.registers[x as usize] != self.registers[y as usize] {
            self.pc += 2;
        }
    } // SkipNEqVyVx

    // sets I to the address nnn
    fn ld_i(&mut self, nnn: u16) {
        self.index = nnn;
    } // LoadI

    // jumps to the address nnn plus v0
    fn jp_v0(&mut self, nnn: u16) {
        self.pc = nnn + u16::from(self.registers[0]);
    } // JumpV0

    // sets vx to the results of a bitwise and operation on a random number (typically 0 to 255) and kk
    fn rnd(&mut self, x: u8, kk: u8) {
        self.registers[x as usize] = kk & self.rng.random::<u8>();
    } // Rand

    // draws a sprite at coordinate (vx, vy) that has a width of 8 pixels and a height of N pixels.
    // each row of 8 pixels is read as bit-coded starting from memory location I;
    // I value does not change after the execution of this instruction.
    // as described above, vf is set to 1 if any screen pixels are flipped
    // from set to unset when the sprite is drawn, and to 0 if that does not happen

    // pixel data: memory[index + i] N u8 integers
    // display data: frame_buffer[y * WIDTH + x]
    #[allow(non_snake_case)]
    fn drw(&mut self, x: u8, y: u8, N: u8) {
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
                let x = x as usize;
                let y = y as usize;
                if x > WIDTH - 1 {
                    break;
                }
                let sprite_pixel = (pixel_data >> (7 - j)) & 0x01;
                let buffer_pixel = self.frame_buffer.get_pixel(x, y)
                if self.registers[FLAG_REGISTER] == 0
                    && sprite_pixel == 1
                    && buffer_pixel == 1
                {
                    self.registers[FLAG_REGISTER] = 1;
                }
                self.frame_buffer.set_pixel(x, y, sprite_pixel ^ buffer_pixel == 1);
            }
        }
    } // Draw

    // skips the next instruction if the key stored in vx (only consider the lowest nibble) is pressed
    fn skp(&mut self, x: u8) {
        if self.keypad[(self.registers[x as usize] & 0x0f) as usize] > 0 {
            self.pc += 2;
        }
    } // SkipKeyPress

    // skips the next instruciton if the key stored in vx (only consider the lowest nibble) is not pressed
    fn sknp(&mut self, x: u8) {
        if self.keypad[(self.registers[x as usize] & 0x0f) as usize] == 0 {
            self.pc += 2;
        }
    } // SkipKeyNPress

    // sets vx to the value of the delay timer
    fn ldf_delay(&mut self, x: u8) {
        self.registers[x as usize] = self.delay_timer;
    } // SetVxToDTimer

    // a key press is awaited, and then stored in vx (blocking operation, all instruction is halted until next key event)
    // delay and sound timers should continue processing
    fn ldf_key(&mut self, x: u8) {
        for (index, &key) in self.keypad.iter().enumerate() {
            if key != 0 && self.prev_keys[index] == 0 {
                self.registers[x as usize] = index as u8;
                return;
            }
        }
        self.pc -= 2;
    } // LoadKey

    // sets the delay timer to vx
    fn ld_delay(&mut self, x: u8) {
        self.delay_timer = self.registers[x as usize];
    } // LoadDTimer

    // sets the sound timer to vx
    fn ld_sound(&mut self, x: u8) {
        self.sound_timer = self.registers[x as usize];
    } // LoadSTimer

    // adds vx to I. vf is not affected
    fn add_i(&mut self, x: u8) {
        self.index += u16::from(self.registers[x as usize]);
    } // AddI

    // sets I to the location of the sprite for the character in vx (only consider the lowest nibble).
    // characters 0-f (in hex) are represnted by a 4x5 font
    fn ld_i_sprite(&mut self, x: u8) {
        let nibble = self.registers[x as usize] & 0x0f;
        self.index = FONT_OFFSET as u16 + u16::from(nibble * 5);
    } // SetI

    // stores the binary-coded decimal representation of vx, with the hundreds digit in memory at location in I
    // the tens digit at location I + 1, and the ones digit at location I + 2.
    fn ld_bcd(&mut self, x: u8) {
        // 034
        let mut temp = self.registers[x as usize];
        for i in (0..3).rev() {
            self.memory[self.index as usize + i] = temp % 10;
            temp /= 10;
        }
    } // Bcd

    // stores from v0..=vx in memory, starting at address I. The offset from I is increased by 1 for each value written,
    // but I itself is left unmodified
    fn ldf_registers(&mut self, x: u8) {
        let base = self.index as usize;
        for i in 0..=x {
            self.memory[base + i as usize] = self.registers[i as usize];
        }
    } // Store

    // fills from v0..=vx with values from memory, starting at address I. The offset from I is increased by 1 for each value read,
    // but I itself is left unmodified
    fn ld_registers(&mut self, x: u8) {
        for i in 0..=x {
            self.registers[i as usize] = self.memory[(self.index + i as u16) as usize];
        }
    } // Fill
} // impl Chip8
