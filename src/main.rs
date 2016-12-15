use cpu::Cpu;
use display::Display;

mod cpu;
mod display;

fn main() {
    let mut cpu = Cpu::new();
    let display = Display::new(cpu.disp_buff);

    // loop {
    //     cpu.emulate_cycle();
    //     display.render();
    // }
}