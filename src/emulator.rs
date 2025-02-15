use std::time::{Duration, Instant};

use wasm_bindgen::{prelude::wasm_bindgen, JsValue};

const DISPLAY_WIDTH: u8 = 64;
const DISPLAY_HEIGHT: u8 = 32;

// #[wasm_bindgen]
pub struct Program {
    memory: [u8; 4096],
    display: [u8; DISPLAY_WIDTH as usize * DISPLAY_HEIGHT as usize],
    program_counter: u16,
    index_register: u16,
    call_stack: Vec<u16>,
    delay_timer: u8,
    sound_timer: u8,
    variable_regsiters: [u8; 16],
    function_table: [OpCodeFn; 0xF + 1],
}

const START_ADDRESS: u16 = 0x200;

type OpCodeFn = fn(program: &mut Program, instruction: u16);

impl Program {
    pub fn new() -> Self {
        const NULL_OP: OpCodeFn = |_, __| {};
        let mut p = Self {
            memory: [0; 4096],
            display: [0; 2048],
            program_counter: START_ADDRESS,
            index_register: 0,
            call_stack: Vec::new(),
            delay_timer: 0xFF,
            sound_timer: 0xFF,
            variable_regsiters: [0; 16],
            function_table: [NULL_OP; 0xF + 1],
        };
        p.set_font();
        p.set_instruction_table();
        p
    }

    pub fn load_rom(&mut self, rom: &[u8]) {
        for i in 0..rom.len() {
            self.memory[i + START_ADDRESS as usize] = rom[i];
        }
    }

    pub fn timer_tick(&mut self) {
        self.delay_timer = self.delay_timer.wrapping_sub(1);
        self.sound_timer = self.sound_timer.wrapping_sub(1);
    }

    pub fn tick(&mut self) {
        let instr_first_byte = self.memory[self.program_counter as usize];
        let instr_second_byte = self.memory[(self.program_counter + 1) as usize];
        let entire_instruction = ((instr_first_byte as u16) << 8) | instr_second_byte as u16;
        let first_nible = instr_first_byte >> 4;
        self.program_counter += 2;
        self.function_table[first_nible as usize](self, entire_instruction);
    }

    pub fn get_display(&self) -> Vec<u8> {
        self.display.into()
    }

    pub fn width() -> u8 {
        DISPLAY_WIDTH
    }
    pub fn height() -> u8 {
        DISPLAY_HEIGHT
    }

    #[inline]
    pub fn pixel_location(x: u8, y: u8) -> usize {
        (y as usize * DISPLAY_WIDTH as usize) + x as usize
    }

    fn set_font(&mut self) {
        // each character is 5 tall by 4 wide.
        // every bytes is a new character
        const CHARACTER_FONTS: [u8; 80] = [
            0xF, 0x9, 0x9, 0x9, 0xF, // 0
            0x2, 0x6, 0x2, 0x2, 0x7, // 1
            0xF, 0x1, 0xF, 0x8, 0xF, // 2
            0xF, 0x1, 0xF, 0x1, 0xF, // 3
            0x9, 0x9, 0xF, 0x1, 0x1, // 4
            0xF, 0x8, 0xF, 0x1, 0xF, // 5
            0xF, 0x8, 0xF, 0x9, 0xF, // 6
            0xF, 0x1, 0x2, 0x4, 0x4, // 7
            0xF, 0x9, 0xF, 0x9, 0xF, // 8
            0xF, 0x9, 0xF, 0x1, 0xF, // 9
            0xF, 0x9, 0xF, 0x9, 0x9, // A
            0xE, 0x9, 0xE, 0x9, 0xE, // B
            0xF, 0x8, 0x8, 0x8, 0xF, // C
            0xE, 0x9, 0x9, 0x9, 0xE, // D
            0xF, 0x8, 0xF, 0x8, 0xF, // E
            0xF, 0x8, 0xF, 0x8, 0x8, // F
        ];
        const FONT_START: usize = 0x050;
        for i in 0..CHARACTER_FONTS.len() {
            self.memory[i + FONT_START] = CHARACTER_FONTS[i];
        }
    }

    pub fn set_instruction_table(&mut self) {
        self.function_table[0x0] = Program::op_0;
        self.function_table[0x1] = Program::op_1;
        self.function_table[0x2] = Program::op_2;
        self.function_table[0x3] = Program::op_3;
        self.function_table[0x4] = Program::op_4;
        self.function_table[0x5] = Program::op_5;
        self.function_table[0x6] = Program::op_6;
        self.function_table[0x7] = Program::op_7;
        self.function_table[0x8] = Program::op_8;
        self.function_table[0x9] = Program::op_9;
        self.function_table[0xA] = Program::op_A;
        self.function_table[0xB] = Program::op_B;
        self.function_table[0xC] = Program::op_C;
        self.function_table[0xD] = Program::op_D;
        self.function_table[0xE] = Program::op_E;
        self.function_table[0xF] = Program::op_F;
    }

    // Instructions bellow
    fn op_0(program: &mut Program, instruction: u16) {
        match instruction {
            // clear screen
            0x00E0 => {
                program.display.fill(0);
            },
            _ => panic!("Encountered an execute machine language routine instruction. This isn't implemented")
        }
    }
    fn op_1(program: &mut Program, instruction: u16) {
        let jump_location = instruction & 0x0FFF;
        program.program_counter = jump_location;
    }
    fn op_2(program: &mut Program, instruction: u16) {}
    fn op_3(program: &mut Program, instruction: u16) {}
    fn op_4(program: &mut Program, instruction: u16) {}
    fn op_5(program: &mut Program, instruction: u16) {}
    fn op_6(program: &mut Program, instruction: u16) {
        let register_name = (instruction & 0x0F00) >> 8;
        let value = (instruction & 0x00FF) as u8;
        program.variable_regsiters[register_name as usize] = value;
    }
    fn op_7(program: &mut Program, instruction: u16) {
        let register_name = (instruction & 0x0F00) >> 8;
        let value = (instruction & 0x00FF) as u8;
        program.variable_regsiters[register_name as usize] += value;
    }
    fn op_8(program: &mut Program, instruction: u16) {}
    fn op_9(program: &mut Program, instruction: u16) {}
    fn op_A(program: &mut Program, instruction: u16) {
        let location = instruction & 0x0FFF;
        program.index_register = location;
    }
    fn op_B(program: &mut Program, instruction: u16) {}
    fn op_C(program: &mut Program, instruction: u16) {}
    fn op_D(program: &mut Program, instruction: u16) {
        let x_register = (instruction & 0x0F00) >> 8;
        let y_register = (instruction & 0x00F0) >> 4;

        // we use modulo in case the variable goes off screen
        let x_start = program.variable_regsiters[x_register as usize] % DISPLAY_WIDTH;
        let y_start = program.variable_regsiters[y_register as usize] % DISPLAY_HEIGHT;
        let rows = (instruction & 0x000F) as u8;

        program.variable_regsiters[0xF as usize] = 0;
        for y in 0..rows {
            let y_location = y_start + y;
            if y_location >= DISPLAY_HEIGHT {
                break;
            }

            let sprite_row = program.memory[(program.index_register + y as u16) as usize];

            for x in 0 as u8..8 {
                let x_location = x_start + x;
                if x_location >= DISPLAY_WIDTH {
                    break;
                }
                if ((sprite_row >> (7 - x)) & 0b1) == 1 {
                    let pixel_location = Program::pixel_location(x_location, y_location) as usize;
                    if program.display[pixel_location] == 1 {
                        program.variable_regsiters[0xF as usize] = 1;
                    }
                    // if it's on turn it off; if it's off turn it on
                    program.display[pixel_location] ^= 1;
                }
            }
        }
    }
    fn op_E(program: &mut Program, instruction: u16) {}
    fn op_F(program: &mut Program, instruction: u16) {}
}
