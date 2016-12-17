pub struct Display {
    buff: [[bool; 64]; 32],
}

impl Display {

    pub fn new(display_buffer: [[bool; 64]; 32], window: PistonWindow) -> Display {
        Display { 
            buff: display_buffer,
        }
    }

    pub fn render_to_raw_terminal(&self) {
        for row in self.buff.iter() {
            for pix in row.iter() {
                if *pix { print!("x"); } else { print!(" "); }
            }
            print!("\n");
        }
    }

}