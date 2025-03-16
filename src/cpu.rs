use core::error;
use std::{fs::File, vec, io};
use std::io::{prelude::*, BufReader};


#[derive(Debug)]
pub struct CPU {
    mem: [u8; 4 * 1024], // Memory
    PC: usize, // program counter by bytes
    display: [[bool; 64]; 32], // digital display
    I: u16, // I points to something in memory
    stack: Vec<u16>, // stack for function / subroutine calls
    delay_timer: u8,
    sound_timer: u8,
    register: [u8; 16],
    pub display_flag: bool
}

const FONT_SET: [u8; 80] = [
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
    0xF0, 0x80, 0xF0, 0x80, 0x80  // F
];

impl CPU {
    pub fn new() -> Self {
        CPU {
            mem: [0; 4096],
            PC: 0x200, // typical starting address
            display: [[false; 64]; 32],
            I: 0,
            stack: Vec::new(),
            delay_timer: 0,
            sound_timer: 0,
            register: [0; 16],
            display_flag: false,
        }
    }
    pub fn load(&mut self, filename: String) -> io::Result<()> {
        let f = BufReader::new(File::open(filename)?);

        let mut prog_start = 0x200;
        for byte in f.bytes() {
            self.mem[prog_start] = byte?;
            prog_start += 1
        }
        let font_start = 0x50;
        self.mem[font_start..font_start + FONT_SET.len()].copy_from_slice(&FONT_SET);
        // reset the PC to 0
        Ok(())
    }

    pub fn update_display_buffer(&mut self, buffer: &mut [u32; 64 * 32]){
        // let mut buffer = [0u32; 64 * 32];
        for y in 0..32 {
            for x in 0..64 {
                buffer[y * 64 + x] = if self.display[y][x] {0xFFFFFF} else {0x0};
            }
        }
        self.display_flag = false;
        // window.
    }

    // fetch-increment-execute loop
    pub fn execute(&mut self) -> Result<(), String>{
        let instr: u16 = (self.mem[self.PC] as u16) << 8 | self.mem[self.PC + 1] as u16;
        self.PC += 2;

        print!("PC: 0x{:04X} Executing: 0x{:04X}", self.PC, instr);
        match (instr & 0xF000) >> 12 {
            0 => { // for now, just clear screen
               self.display = [[false; 64]; 32];
               self.display_flag = true;
               println!(" clr");
            }
            1 => { // jump
                self.PC = (instr & 0x0FFF) as usize; // set the PC to the last 3 bits
                println!(" jmp 0x{:04X}", self.PC);
            }
            6 => { // 6XNN set register VX
                self.register[((instr & 0x0F00) >> 8) as usize] = (instr & 0x00FF) as u8;
                println!(" set 0x{:01X}, 0x{:02X}", ((instr & 0x0F00) >> 8), (instr & 0x00FF)); 
            }
            7 => { // add value to register
                self.register[((instr & 0x0F00) >> 8) as usize] += (instr & 0x00FF) as u8;
                println!(" add 0x{:01X}, 0x{:02X}", ((instr & 0x0F00) >> 8), (instr & 0x00FF)); 
            }
            0xA => { // set index register I
                self.I = (instr & 0x0FFF) as u16;
                println!(" seti 0x{:03X}", (instr & 0x0FFF)); 
            }
            0xD => { // display / draw
                let x = self.register[((instr & 0x0F00) >> 8) as usize] & 63;
                let y: u16 = (self.register[((instr & 0x00F0) >> 4) as usize] & 31) as u16;
                let n = (instr & 0x000F);
                println!(" draw 0x{:01X}, 0x{:01X}, 0x{:01X}", x, y, n);
                self.register[0xF as usize] = 0;
                for i in 0..n {
                    let byte = self.mem[(self.I + i as u16) as usize];
                    print!("0x{:02X} ", byte);
                    for j in 0..8{
                        if x + j >= 64 {
                            break;
                        }
                        let cur = self.display[(y + i) as usize][(x + j) as usize];
                        // let new = (byte & (1<<j)) != 0;
                        let new = (byte & (0x80 >> j)) != 0; // 0x80 = 0b10000000
                        if cur && new {
                            self.register[0xF as usize] = 1;
                        }
                        self.display[(y + i) as usize][(x + j) as usize] = cur ^ new; 
                    }
                    if y + i >= 32 {
                        break;
                    }
                }
                    println!();
                self.display_flag = true;
            }
            _ => {
                return Err(format!("Instruction cannot be matched: 0x{:04X}", instr));
            }
        } 
        Ok(())
    }
}