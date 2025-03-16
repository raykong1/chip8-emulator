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
    emu.load("test_roms/ibm_logo.ch8".to_string()).unwrap();
    let mut window = Window::new(
        "Minifb Test Window",
        width,
        height,
        WindowOptions {
            scale: minifb::Scale::X16, // Scale up 10x automatically
            ..WindowOptions::default()
        },
    ).expect("Failed to create window");
    window.limit_update_rate(Some(std::time::Duration::from_millis(16))); // ~60 FPS
    let mut buffer = [0u32; 32 * 64];
    let frame_duration = Duration::from_millis(16); // approx 60 fps

    while window.is_open(){
        let frame_start = Instant::now();
        // execute 10 instr per frame
        for _ in 0..1 {
            emu.execute()?;
        }

        if emu.display_flag {
            emu.update_display_buffer(&mut buffer);
            window.update_with_buffer(&buffer, 64, 32).unwrap();
        }
        
        let elapsed = frame_start.elapsed();
        if elapsed < frame_duration {
            std::thread::sleep(frame_duration - elapsed);
        }
    }

    Ok(())
        // window.
}
