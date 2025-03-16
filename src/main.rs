use crate::cpu::CPU;
use std::time::{Duration, Instant};
use minifb::{Window, WindowOptions};
pub mod cpu;

fn main() -> Result<(), String>{
    let mut emu = CPU::new();
    let width = 64;
    let height = 32;
    let scale = 10;
    // println!("Current dir: {:?}", std::env::current_dir().unwrap());
    emu.load_and_execute("test_roms/test_opcode.ch8".to_string()).unwrap();
    Ok(())
        // window.
}
