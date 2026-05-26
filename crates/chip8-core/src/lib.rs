mod instructions;
mod rom;

use std::{
    fs::File,
    io::{BufReader, Read},
    path::Path,
};

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

pub struct FrameBuffer {
    buffer: [u8; 256],
}

impl Default for FrameBuffer {
    fn default() -> Self {
        Self { buffer: [0; 256] }
    }
}

impl FrameBuffer {
    pub fn get_pixel(&self, x: usize, y: usize) -> u8 {
        let pixel_index = y * WIDTH + x;
        let bit_index = 7 - pixel_index % 8;
        (self.buffer[pixel_index >> 3]) >> bit_index & 1
    }

    fn set_pixel(&mut self, x: usize, y: usize, val: bool) {
        let pixel_index = y * WIDTH + x;
        let byte_index = pixel_index >> 3;
        let bit_index = 7 - pixel_index % 8;
        if val {
            self.buffer[byte_index] |= 1 << bit_index;
        } else {
            self.buffer[byte_index] &= !(1 << bit_index);
        }
    }

    fn clear(&mut self) {
        self.buffer = [0; 256];
    }

    pub fn get(&self) -> &[u8; 256] {
        &self.buffer
    }
}

pub struct Chip8 {
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

impl Default for Chip8 {
    fn default() -> Self {
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
}

impl Chip8 {

    pub fn cycle(&mut self) {
        if self.delay_timer > 0 {
            self.delay_timer -= 1;
        }
        if self.sound_timer > 0 {
            self.sound_timer -= 1;
        }
        for _ in 0..12 {
            
            let opcode = self.fetch();
            let instruction = self.decode(opcode);
            self.execute(instruction);
        }
        self.check_keys();
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

    pub fn get_buffer(&self) -> &FrameBuffer {
        &self.frame_buffer
    }

    pub fn check_keys(&mut self) {
        for (index, key) in self.keypad.iter().enumerate() {
            if *key == 1 && self.prev_keys[index] == 0 {
                self.prev_keys[index] = 1;
            }
            if *key == 0 && self.prev_keys[index] == 1 {
                self.prev_keys[index] = 0;
            }
        }
    }

    pub fn input(&mut self, key: u8, pressed: bool) {
        if pressed {
            self.keypad[key as usize] = 1;
        } else {
            self.keypad[key as usize] = 0;
        }
    }

    pub fn load_rom(&mut self, path: &Path) -> Result<(), Box<dyn std::error::Error>> {
        let display = path.display();
        let file = match File::open(path) {
            Err(why) => panic!("couldn't open {}: {}", display, why),
            Ok(file) => file,
        };
        let reader = BufReader::new(file);
        // debug
        println!("reading rom byte by byte");
        for (index, byte_result) in reader.bytes().enumerate() {
            match byte_result {
                Ok(byte) => {
                    print!("0x{:02X} ", byte);
                    self.memory[index + 0x200] = byte;
                }
                Err(e) => {
                    eprintln!("\nError reading byte: {}", e);
                    return Err(e.into());
                }
            }
        }
        Ok(())
    }
} // impl Chip8
