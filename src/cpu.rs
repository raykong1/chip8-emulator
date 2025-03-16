use core::error;
use std::{fs::File, vec, io};
use std::io::{prelude::*, BufReader};
use std::time::{Duration, Instant};
use minifb::{Key, Window, WindowOptions};
use rand::prelude::*;


const WIDTH: usize = 64;
const HEIGHT: usize = 32;
const FONT_START: usize = 0x50;

#[derive(Debug)]
pub struct CPU {
    mem: [u8; 4 * 1024], // Memory
    PC: usize, // program counter by bytes
    display: [[bool; WIDTH as usize]; HEIGHT as usize], // digital display
    I: u16, // I points to something in memory
    stack: Vec<usize>, // stack for function / subroutine calls
    delay_timer: u8,
    sound_timer: u8,
    register: [u8; 16],
    pub display_flag: bool,
    window: Window
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

fn get_chip8_key(key: Key) -> Option<u8> {
    match key {
        Key::Key1 => Some(0x1),
        Key::Key2 => Some(0x2),
        Key::Key3 => Some(0x3),
        Key::Key4 => Some(0xC),
        Key::Q    => Some(0x4),
        Key::W    => Some(0x5),
        Key::E    => Some(0x6),
        Key::R    => Some(0xD),
        Key::A    => Some(0x7),
        Key::S    => Some(0x8),
        Key::D    => Some(0x9),
        Key::F    => Some(0xE),
        Key::Z    => Some(0xA),
        Key::X    => Some(0x0),
        Key::C    => Some(0xB),
        Key::V    => Some(0xF),
        _         => None,
    }
}

fn get_qwerty_key(chip8_key: u8) -> Option<Key> {
    let key = match chip8_key {
        0x1 => Some(Key::Key1),
        0x2 => Some(Key::Key2),
        0x3 => Some(Key::Key3),
        0xC => Some(Key::Key4),
        0x4 => Some(Key::Q),
        0x5 => Some(Key::W),
        0x6 => Some(Key::E),
        0xD => Some(Key::R),
        0x7 => Some(Key::A),
        0x8 => Some(Key::S),
        0x9 => Some(Key::D),
        0xE => Some(Key::F),
        0xA => Some(Key::Z),
        0x0 => Some(Key::X),
        0xB => Some(Key::C),
        0xF => Some(Key::V),
        _ => None, // Invalid keycode
    };
    return key;
}

impl CPU {
    pub fn new() -> Self {
        let mut ret = CPU {
            mem: [0; 4096],
            PC: 0x200, // typical starting address
            display: [[false; WIDTH]; HEIGHT],
            I: 0,
            stack: Vec::new(),
            delay_timer: 0,
            sound_timer: 0,
            register: [0; 16],
            display_flag: false,
            window: Window::new(
                "Minifb Test Window",
                WIDTH,
                HEIGHT,
                WindowOptions {
                    scale: minifb::Scale::X16, // Scale up 10x automatically
                    ..WindowOptions::default()
                },
            ).expect("Failed to create window"),
        };
        ret.window.limit_update_rate(Some(std::time::Duration::from_millis(16))); // ~60 FPS
        return ret;
    }

    fn start(&mut self) -> Result<(), String> {
        let mut buffer = [0u32; WIDTH * HEIGHT];
        // execute loop
        let frame_duration = Duration::from_millis(16); // approx 60 fps

        while self.window.is_open(){
            let frame_start = Instant::now();
            // execute 10 instr per frame
            for _ in 0..1 {
                self.execute()?;
            }

            if self.display_flag {
                self.update_display_buffer(&mut buffer);
                self.window.update_with_buffer(&buffer, 64, 32).unwrap();
            }

            // decrement timers
            self.delay_timer = if self.delay_timer == 0 {60} else {self.delay_timer - 1};
            self.sound_timer = if self.sound_timer == 0 {60} else {self.sound_timer - 1};
            
            let elapsed = frame_start.elapsed();
            if elapsed < frame_duration {
                std::thread::sleep(frame_duration - elapsed);
            }
        }
        Ok(())
    }

    pub fn load_and_execute(&mut self, filename: String) -> Result<(), String> {
        let f = BufReader::new(File::open(filename).unwrap());

        let mut prog_start = 0x200;
        for byte in f.bytes() {
            self.mem[prog_start] = byte.unwrap();
            prog_start += 1
        }
        self.mem[FONT_START..FONT_START + FONT_SET.len()].copy_from_slice(&FONT_SET);

        self.start()
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
            0 => match instr & 0x00FF { 
                0x00E0 => { // clear screen
                    self.display = [[false; 64]; 32];
                    self.display_flag = true;
                    println!(" clr");
                }
                0x00EE => { // pop subroutine
                    let link = self.stack.pop().unwrap();
                    self.PC = link.into();
                    println!(" retn");
                }
                _ => {
                    return Err(format!("Instruction cannot be matched: 0x{:04X}", instr));
                }
            }
            1 => { // 1NNN jump
                self.PC = (instr & 0x0FFF) as usize; // set the PC to the last 3 bits
                println!(" jmp 0x{:04X}", self.PC);
            }
            2 => { // 2NNN JALR
                self.stack.push(self.PC);
                self.PC = (instr & 0x0FFF) as usize; // set the PC to the last 3 bits
                println!(" jalr 0x{:04X}", self.PC);
            }
            3 => { // 3XNN beq
                let vx = ((instr & 0x0F00) >> 8) as usize;
                let a = self.register[vx];
                let b = instr & 0x00FF;
                if a == b as u8 {
                    self.PC += 2;
                }
                println!(" beq 0x{:01X}, 0x{:02X}", vx, b);
            }
            4 => { // 4XNN bne
                let vx = ((instr & 0x0F00) >> 8) as usize;
                let a = self.register[vx];
                let b = instr & 0x00FF;
                if a != b as u8 {
                    self.PC += 2;
                }
                println!(" bne 0x{:01X}, 0x{:02X}", vx, b);
            }
            5 => { // 5XY0
                let vx = ((instr & 0x0F00) >> 8) as usize;
                let vy = ((instr & 0x00F0) >> 4) as usize;
                let a = self.register[vx];
                let b = self.register[vy];
                if a == b {
                    self.PC += 2;
                }
                println!(" ber 0x{:01X}, 0x{:01X}", vx, vy);
            }
            6 => { // 6XNN set register VX
                self.register[((instr & 0x0F00) >> 8) as usize] = (instr & 0x00FF) as u8;
                println!(" set 0x{:01X}, 0x{:02X}", ((instr & 0x0F00) >> 8), (instr & 0x00FF)); 
            }
            7 => { // 7XNN add value to register
                let vx = ((instr & 0x0F00) >> 8) as usize;
                self.register[vx] = self.register[vx].wrapping_add((instr & 0x00FF) as u8);
                println!(" addi 0x{:01X}, 0x{:02X}", ((instr & 0x0F00) >> 8), (instr & 0x00FF)); 
            }
            8 => {
                let vx = ((instr & 0x0F00) >> 8) as usize;
                let vy = ((instr & 0x00F0) >> 4) as usize;
                match instr & 0x000F { // 8XYn -- arithmetic / logical operations
                    0 => { // set
                        self.register[vx] = self.register[vy];
                        println!(" set 0x{:01X}, 0x{:01X}", vx, vy); 
                    }
                    1 => { //OR
                        self.register[vx] |= self.register[vy];
                        println!(" or 0x{:01X}, 0x{:01X}", vx, vy); 
                    }
                    2 => { // and
                        self.register[vx] &= self.register[vy];
                        println!(" and 0x{:01X}, 0x{:01X}", vx, vy); 
                    }
                    3 => {
                        self.register[vx] ^= self.register[vy];
                        println!(" xor 0x{:01X}, 0x{:01X}", vx, vy); 
                    }
                    4 => {
                        let val= self.register[vx] as u16 + self.register[vy] as u16;
                        if val > 0xFF {
                            self.register[0xF] = 1;
                        } else{
                            self.register[0xF] = 0;
                        }
                        self.register[vx] = (val & 0xFF) as u8;
                        println!(" add 0x{:01X}, 0x{:01X}", vx, vy); 
                    }
                    5 => {
                        let (result, did_borrow) = self.register[vx].overflowing_sub(self.register[vy]);
                        self.register[vx] = result;
                        self.register[0xF] = if did_borrow { 0 } else { 1 };
                        println!(" sub 0x{:01X}, 0x{:01X}", vx, vy); 
                    }
                    7 => {
                        let (result, did_borrow) = self.register[vy].overflowing_sub(self.register[vx]);
                        self.register[vx] = result;
                        self.register[0xF] = if did_borrow { 0 } else { 1 };
                        println!(" rsub 0x{:01X}, 0x{:01X}", vx, vy); 
                    }
                    6 => {
                        self.register[vx] = self.register[vy] >> 1; 
                        println!(" str 0x{:01X}, 0x{:01X}", vx, vy); 
                    }
                    0xE => {
                        self.register[vx] = self.register[vy] << 1; 
                        println!(" stl 0x{:01X}, 0x{:01X}", vx, vy); 
                    }
                    _ => {
                        return Err(format!("Instruction cannot be matched: 0x{:04X}", instr));
                    }
                }
            }
            9 => { // 5XY0
                let vx = ((instr & 0x0F00) >> 8) as usize;
                let vy = ((instr & 0x00F0) >> 4) as usize;
                let a = self.register[vx];
                let b = self.register[vy];
                if a != b as u8 {
                    self.PC += 2;
                }
                println!(" bnr 0x{:01X}, 0x{:01X}", vx, vy);
            }
            0xA => { // ANNN set index register I
                self.I = (instr & 0x0FFF) as u16;
                println!(" seti 0x{:03X}", (instr & 0x0FFF)); 
            }
            0xB => { // BNNN Jump with offset in V0
                self.PC = (instr & 0x0FFF) as usize + self.register[0] as usize;
                println!(" jwo 0x{:03X}", (instr & 0x0FFF)); 
            }
            0xC => { // CXNN rnd & NN
                let vx = ((instr & 0x0F00) >> 8) as usize;
                let rnd = rand::random::<u16>() & instr & 0x00FF;
                self.register[vx] = rnd as u8;
                println!(" rnd 0x{:X}, 0x{:02X}", vx, (instr & 0x00FF));
            }
            0xD => { // DXYN display / draw
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
            0xE => { // 
                let vx = ((instr & 0x0F00) >> 12) as usize;
                let keycode = self.register[vx];
                match instr & 0x00FF {
                    0x9E => { // skip if pressed
                        if self.window.is_key_down(get_qwerty_key(keycode).unwrap()) {
                            self.PC += 2;
                        }
                        println!(" skp 0x{:X}", vx);
                    }
                    0xA1 => {
                        if !self.window.is_key_down(get_qwerty_key(keycode).unwrap()) {
                            self.PC += 2;
                        }
                        println!(" snp 0x{:X}", vx);
                    }
                    _ => {
                        return Err(format!("Instruction cannot be matched: 0x{:04X}", instr));
                    }
                }
            }
            0xF => {
                let vx = ((instr & 0x0F00) >> 8) as usize;
                match instr & 0x00FF {
                    0x07 => {
                        self.register[vx] = self.delay_timer;
                    }
                    0x15 => {
                        self.delay_timer = self.register[vx];
                    }
                    0x18 => {
                        self.sound_timer = self.register[vx];
                    }
                    0x1E => {
                        self.I += self.register[vx] as u16;
                    }
                    0x0A => { // get key
                        let keys = self.window.get_keys_pressed(minifb::KeyRepeat::No);
                        if let Some(&key) = keys.first() {
                            if let Some(chip8_key) = get_chip8_key(key) {
                                self.register[vx] = chip8_key;
                            } else {
                                self.PC -= 2;
                            }
                        } else {
                            self.PC -= 2;
                        }
                    }
                    0x29 => { // font character
                        self.I = (FONT_START + self.register[vx] as usize * 5) as u16;
                    }
                    0x33 => { // decimal division -- stores from I, I + 1, I + 2, in little endian
                        let val = self.register[vx];
                        self.mem[self.I as usize] = val / 100;
                        self.mem[self.I as usize + 1] = (val % 100) / 10;
                        self.mem[self.I as usize + 2] = val % 10;
                    }
                    0x55 => { // store memory
                        for i in 0..=vx {
                            self.mem[self.I as usize + i] = self.register[i]
                        }
                    }
                    0x65 => { // load memory
                        for i in 0..=vx {
                            self.register[i] = self.mem[self.I as usize + i]
                        }
                    }
                    _ => {
                        return Err(format!("Instruction cannot be matched: 0x{:04X}", instr));
                    }
                }
            }
            _ => {
                return Err(format!("Instruction cannot be matched: 0x{:04X}", instr));
            }
        } 
        Ok(())
    }
}