mod cpu;

extern crate piston_window;
extern crate piston;

use piston::input::generic_event::GenericEvent;
use piston_window::*;
use cpu::Cpu;

struct Machine {
    cpu: Cpu
}

impl Machine {

    fn new() -> Machine {
        Machine { cpu : Cpu::new() }
    }

    fn load_rom() {
        
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

    // loop {
    //     cpu.emulate_cycle();
    //     display.render();
    // }
}