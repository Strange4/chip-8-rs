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
}

const START_ADDRESS: u16 = 0x200;
impl Program {
    pub fn new() -> Self {
        let p = Self {
            memory: [0; 4096],
            display: [0; 2048],
            program_counter: START_ADDRESS,
            index_register: 0,
            call_stack: Vec::new(),
            delay_timer: 0xFF,
            sound_timer: 0xFF,
            variable_regsiters: [0; 16],
        };
        p.set_font()
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
        self.program_counter += 2;
        match instr_first_byte & 0xF0 {
            0x00 => {
                let entire_instruction =
                    ((instr_first_byte as u16) << 8) + instr_second_byte as u16;
                match entire_instruction {
                    // clear screen
                    0x00E0 => {
                        self.display.fill(0);
                    },
                    _ => panic!("Encountered an execute machine language routine instruction. This isn't implemented")
                }
            }
            // jump
            0x10 => {
                let jump_location =
                    (((instr_first_byte & 0x0F) as u16) << 8) + instr_second_byte as u16;
                self.program_counter = jump_location;
            }
            // set register
            0x60 => {
                let register_name = instr_first_byte & 0x0F;
                self.variable_regsiters[register_name as usize] = instr_second_byte;
            }
            // add to register
            0x70 => {
                let register_name = instr_first_byte & 0x0F;
                self.variable_regsiters[register_name as usize] += instr_second_byte;
            }
            // set the I register for drawing fonts
            0xA0 => {
                let location = (((instr_first_byte & 0x0F) as u16) << 8) + instr_second_byte as u16;
                self.index_register = location;
            }
            // display
            0xD0 => {
                let x_register = instr_first_byte & 0x0F;
                let y_register = (instr_second_byte & 0xF0) >> 4;

                // we use modulo in case the variable goes off screen
                let x_start = self.variable_regsiters[x_register as usize] % DISPLAY_WIDTH;
                let y_start = self.variable_regsiters[y_register as usize] % DISPLAY_HEIGHT;

                let rows = instr_second_byte & 0x0F;
                self.variable_regsiters[0xF as usize] = 0;
                for y in 0..rows {
                    let y_location = y_start + y;
                    if y_location >= DISPLAY_HEIGHT {
                        break;
                    }

                    let sprite_row = self.memory[(self.index_register + y as u16) as usize];

                    for x in 0 as u8..8 {
                        let x_location = x_start + x;
                        if x_location >= DISPLAY_WIDTH {
                            break;
                        }
                        if ((sprite_row >> (7 - x)) & 0b1) == 1 {
                            let pixel_location =
                                Self::pixel_location(x_location, y_location) as usize;
                            if self.display[pixel_location] == 1 {
                                self.variable_regsiters[0xF as usize] = 1;
                            }
                            // if it's on turn it off; if it's off turn it on
                            self.display[pixel_location] ^= 1;
                        }
                    }
                }
            }
            _ => panic!("Encountered an unexpected instruction"),
        }
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

    fn set_font(mut self) -> Self {
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
        self
    }
}
