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
const SPRITE_WIDTH: usize = 8;

const MEMORY_SIZE: usize = 4096;
const NUM_KEYS: usize = 16;
const NUM_REGISTERS: usize = 16;
const STACK_CAPACITY: usize = 16;

pub const FLAG_REGISTER: usize = 15;

// not using vector because original CHIP8 had limited stack anyways - shouldn't be an issue
struct EightStack {
    stack: [u16; STACK_CAPACITY],
    size: usize,
}

impl EightStack {
    fn new() -> Self {
        let stack = [0; STACK_CAPACITY];
        Self { stack, size: 0 }
    }

    pub fn size(&self) -> usize {
        self.size
    }

    pub fn is_empty(&self) -> bool {
        self.size == 0
    }

    pub fn is_full(&self) -> bool {
        self.size == STACK_CAPACITY
    }

    pub fn pop(&mut self) -> Option<u16> {
        if self.is_empty() {
            return None;
        }

        let size = self.size - 1;
        self.size -= 1;
        Some(self.stack[size])
    }

    pub fn peek(&self) -> Option<u16> {
        if self.is_empty() {
            return None;
        }
        Some(self.stack[self.size - 1])
    }

    pub fn push(&mut self, val: u16) {
        if self.is_full() {
            panic!("Stack Overflow");
        }

        self.stack[self.size] = val;
        self.size += 1;
    }
} // impl EightStack

struct Chip8 {
    pc: u16,
    index: u16,
    memory: [u8; MEMORY_SIZE],
    stack: EightStack,
    prev_keys: [u8; NUM_KEYS],
    keypad: [u8; NUM_KEYS],
    display: [u8; WIDTH * HEIGHT],
    registers: [u8; NUM_REGISTERS],
    delay_timer: u8,
    sound_timer: u8,
    rng: ThreadRng,
}

impl Chip8 {
    fn new() -> Self {
        let stack = EightStack::new();
        let mut memory = [0; MEMORY_SIZE];
        memory[FONT_OFFSET..FONT_OFFSET + FONT.len()].copy_from_slice(&FONT);
        let rng = rand::rng();
        Self {
            pc: 0x200,
            index: 0,
            memory,
            stack,
            prev_keys: [0; NUM_KEYS],
            keypad: [0; NUM_KEYS],
            display: [0; WIDTH * HEIGHT],
            registers: [0; NUM_REGISTERS],
            delay_timer: 0,
            sound_timer: 0,
            rng,
        }
    }

    fn pixel_coords(&self, x: u8, y: u8) -> usize {
        return usize::from(y) * WIDTH + usize::from(x);
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
        let n1 = (opcode >> 12) as u8 & 0x000f;
        let x = (opcode >> 8) as u8 & 0x000f;
        let y = (opcode >> 4) as u8 & 0x000f;
        let n = opcode as u8 & 0x000f;

        let nn = opcode as u8;
        let nnn = opcode & 0x0fff;

        match (n1, x, y, n) {
            (0x0, 0x0, 0xE, 0x0) => Instruction::Clear,
            (0x0, 0x0, 0xE, 0xE) => Instruction::Return,
            (0x1, _, _, _) => Instruction::Jump { nnn },
            (0x2, _, _, _) => Instruction::Call { nnn },
            (0x3, _, _, _) => Instruction::SkipEq { x, nn },
            (0x4, _, _, _) => Instruction::SkipNEq { x, nn },
            (0x5, _, _, 0x0) => Instruction::SkipEqVy { x, y },
            (0x6, _, _, _) => Instruction::Load { x, nn },
            (0x7, _, _, _) => Instruction::Add { x, nn },
            (0x8, _, _, 0x0) => Instruction::LoadVy { x, y },
            (0x8, _, _, 0x1) => Instruction::Or { x, y },
            (0x8, _, _, 0x2) => Instruction::And { x, y },
            (0x8, _, _, 0x3) => Instruction::Xor { x, y },
            (0x8, _, _, 0x4) => Instruction::AddVy { x, y },
            (0x8, _, _, 0x5) => Instruction::SubVy { x, y },
            (0x8, _, _, 0x6) => Instruction::ShiftR { x },
            (0x8, _, _, 0x7) => Instruction::SubVyVx { x, y },
            (0x8, _, _, 0xE) => Instruction::ShiftL { x },
            (0x9, _, _, 0x0) => Instruction::SkipNEqVy { x, y },
            (0xA, _, _, _) => Instruction::LoadI { nnn },
            (0xB, _, _, _) => Instruction::JumpV0 { nnn },
            (0xC, _, _, _) => Instruction::Rand { x, nn },
            (0xD, _, _, _) => Instruction::Draw { x, y, n },
            (0xE, _, 0x9, 0xE) => Instruction::SkipKeyPress { x },
            (0xE, _, 0xA, 0x1) => Instruction::SkipKeyNPress { x },
            (0xF, _, 0x0, 0x7) => Instruction::GetDelayTimer { x },
            (0xF, _, 0x0, 0xA) => Instruction::WaitKey { x },
            (0xF, _, 0x1, 0x5) => Instruction::SetDelayTimer { x },
            (0xF, _, 0x1, 0x8) => Instruction::SetSoundTimer { x },
            (0xF, _, 0x1, 0xE) => Instruction::AddI { x },
            (0xF, _, 0x2, 0x9) => Instruction::LoadFont { x },
            (0xF, _, 0x3, 0x3) => Instruction::Bcd { x },
            (0xF, _, 0x5, 0x5) => Instruction::Store { x },
            (0xF, _, 0x6, 0x5) => Instruction::Fill { x },
            _ => Instruction::Unknown(opcode),
        }
    }


} // impl Chip8
