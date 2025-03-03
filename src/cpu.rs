use std::vec;

#[derive(Debug)]
pub struct CPU {
    mem: [i8; 4 * 1024], // Memory
    PC: u32, // program counter
    display: [[bool; 64]; 32], // digital display
    I: u16, // I points to something in memory
    stack: Vec<i16>, // stack for function / subroutine calls
    delay_timer: u8,
    sound_timer: u8,
    register: [i8; 16],

    
}

impl CPU {
    // fn load(Vec<string>) {

    // }
}