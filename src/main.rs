use macroquad::prelude::*;

#[macroquad::main("Main Window")]
async fn main() {
    loop {
        clear_background(GREEN);
        next_frame().await
    }
}
