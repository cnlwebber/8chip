mod instructions;

use rand::rngs::ThreadRng;

use crate::instructions::Instruction;

const FONT: [u8; 80] = [
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
const FONT_OFFSET: usize = 0x050;

pub const WIDTH: usize = 64;
pub const HEIGHT: usize = 32;
pub const SPRITE_WIDTH: usize = 8;
pub const FLAG_REGISTER: usize = 15;

struct FrameBuffer {
    buffer: [u8; 256]
}

impl Default for FrameBuffer {
    fn default() -> Self {
        Self {
            buffer: [0; 256]
        }
    }
}

impl FrameBuffer {
    fn get_pixel(&self, x: usize, y: usize) -> u8 {
        let pixel_index = y * HEIGHT + x;
        (self.buffer[pixel_index >> 8]) >> (7 - (pixel_index % 8)) & 1
    }

    fn set_pixel(&mut self, x: usize, y: usize, val: bool) {
        let pixel_index = y * HEIGHT + x;
        let byte_index = pixel_index >> 8;
        let bit_index = 7 - pixel_index % 8;
        if val {
            self.buffer[byte_index] |= 1 << bit_index;
        } else {
            self.buffer[byte_index] &= !(1 << bit_index);
        }
    }

    fn clear(&mut self) {
        self.buffer = [0; 256] ;
    }
}

struct Chip8 {
    pc: u16,
    index: u16,
    memory: [u8; 4096],
    stack: [u16; 16],
    stack_ptr: u8,
    prev_keys: [u8; 16],
    keypad: [u8; 16],
    frame_buffer: FrameBuffer,
    registers: [u8; 16],
    delay_timer: u8,
    sound_timer: u8,
    rng: ThreadRng,
}

impl Chip8 {
    fn new() -> Self {
        let mut memory = [0; 4096];
        memory[FONT_OFFSET..FONT_OFFSET + FONT.len()].copy_from_slice(&FONT);
        let rng = rand::rng();
        Self {
            pc: 0x200,
            index: 0,
            memory,
            stack: [0; 16],
            stack_ptr: 0,
            prev_keys: [0; 16],
            keypad: [0; 16],
            frame_buffer: FrameBuffer::default(),
            registers: [0; 16],
            delay_timer: 0,
            sound_timer: 0,
            rng,
        }
    }

    fn tick(&mut self) {
        let opcode = self.fetch();
        let instruction = self.decode(opcode);
        self.execute(instruction);
    }

    fn fetch(&mut self) -> u16 {
        let pc = self.pc;
        self.pc += 2;
        u16::from(self.memory[pc as usize]) << 8 | u16::from(self.memory[pc as usize + 1])
    }

    fn decode(&mut self, opcode: u16) -> Instruction {
        Instruction {
            group: (opcode >> 12) as u8 & 0x000f,
            x: (opcode >> 8) as u8 & 0x000f,
            y: (opcode >> 4) as u8 & 0x000f,
            N: opcode as u8 & 0x000f,
            kk: opcode as u8,
            nnn: opcode & 0x0fff,
        }
    }
} // impl Chip8
