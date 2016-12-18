#![feature(inclusive_range_syntax)]

mod cpu;

extern crate piston_window;
extern crate piston;

use piston::input::generic_event::GenericEvent;
use piston_window::*;
use std::env;
use std::fs::File;
use std::io::Read;
use std::process;
use cpu::Cpu;

struct Machine {
    cpu: Cpu
}

impl Machine {

    fn new() -> Machine {
        Machine { cpu : Cpu::new() }
    }

    fn load_rom(&mut self) {
        let args: Vec<String> = env::args().collect();
        let ref rom;
        if args.len() > 1 {
            rom = &args[1];
        } else {
            println!("Please provide a path to a chip8 rom as a command line argument.");
            process::exit(0);
        }


        let file = File::open(rom);
        let mut rom_data = Vec::new();
        let read_result;
        match file {
            Ok(mut f) => { read_result = f.read_to_end(&mut rom_data) },
            Err(e) => {
                println!("Error reading file: {:?}", e);
                process::exit(0);
            }
        }

        match read_result {
            Ok(_) => Cpu::load_data(&mut self.cpu, rom_data),
            Err(e) => {
                println!("Error reading rom: {:?}", e);
                process::exit(0);
            }
        }
    }

    fn on_update(&mut self) {
        self.cpu.emulate_cycle();
    }

    fn on_draw<E: GenericEvent>(&mut self, w: &mut PistonWindow, e: E) {
        let black: [f32; 4] = [0.0, 0.0, 0.0, 1.0];
        let white: [f32; 4] = [1.0, 1.0, 1.0, 1.0];
        let square = rectangle::square(0.0, 0.0, 10.0);

        w.draw_2d(&e, |c, g| {
            clear(black, g);
            for (i, row) in self.cpu.disp_buff.iter().enumerate() {
                for (ii, &pixel) in row.iter().enumerate() {
                    let pixel_color;
                    if pixel {
                        pixel_color = white;
                    } else {
                        pixel_color = black;
                    }

                    let pix_loc = c.transform.trans((ii * 10) as f64, (i * 10) as f64);

                    rectangle(pixel_color, square, pix_loc, g);
                }
            }
        });
    }
}

fn main() {

    let mut machine = Machine::new();
    machine.load_rom();

    let mut window: PistonWindow = WindowSettings::new(
        "chip8 emulator", [640, 320]
    )
    .exit_on_esc(true)
    .build()
    .unwrap();

    let mut events = window.events();
    while let Some(e) = events.next(&mut window) {
        if let Some(_) = e.render_args() {
            machine.on_draw(&mut window, e);
        } else if let Some(_) = e.update_args() {
            machine.on_update();
        }
    }

}