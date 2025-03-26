use macroquad::prelude::*;

#[macroquad::main("Xnake")]
async fn main() {
    loop {
        clear_background(DARKPURPLE);
        next_frame().await;
    }
}
