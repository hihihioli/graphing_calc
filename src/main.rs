use macroquad::miniquad::window::dpi_scale;
use macroquad::prelude::*;
use std::default::Default;

fn window_conf() -> Conf {
    //the config for the main window
    Conf {
        window_title: "Main Window".to_string(),
        high_dpi: false,
        ..Default::default()
    }
}

#[macroquad::main(window_conf)] //pass with config in
async fn main() {
    let mut scale = 20f32;
    let mut corner1: (f32, f32);
    let mut corner2: (f32, f32);
    let dpi_scale = screen_dpi_scale();
    let mut x = 0f32;
    let mut y = 0f32;
    loop {
        let delta_time = get_frame_time();
        //handle zoom

        scale += scale * mouse_wheel().1 * delta_time * 0.5;
        if scale < 1f32 {
            scale = 1f32;
        }
        //handle movement
        if is_key_down(KeyCode::Right) {
            x += 200f32 * delta_time / scale;
        }
        if is_key_down(KeyCode::Left) {
            x -= 200f32 * delta_time / scale;
        }
        if is_key_down(KeyCode::Down) {
            y += 200f32 * delta_time / scale;
        }
        if is_key_down(KeyCode::Up) {
            y -= 200f32 * delta_time / scale;
        }
        //update corners
        corner1 = (
            -1f32 * screen_width() / (2f32 * scale) + x,
            screen_height() / (2f32 * scale) - y,
        );
        corner2 = (
            //
            screen_width() / (2f32 * scale) + x,
            -1f32 * screen_height() / (2f32 * scale) - y,
        );
        let x_scale = (screen_width() / scale) / (corner2.0 - corner1.0);
        let y_scale = (screen_height() / scale) / (corner1.1 - corner2.1);

        draw_fps();
        draw_text(
            format!("{},{}", corner1.0, corner1.1).as_str(),
            10.0,
            35.0,
            25.0,
            WHITE,
        );
        draw_text(
            format!("{},{}", corner2.0, corner2.1).as_str(),
            10.0,
            55.0,
            25.0,
            WHITE,
        );
        draw_text(
            format!("{},{}, scale: {}", screen_width(), screen_height(), scale).as_str(),
            10.0,
            75.0,
            25.0,
            WHITE,
        );

        //Draw the axes
        let x_middle = (-1f32 * corner1.0) * scale * x_scale;
        let y_middle = (corner1.1) * scale * y_scale;

        //y-axis
        draw_line(x_middle, 0f32, x_middle, screen_height(), 1f32, GREEN);
        //draw the tick marks
        for i in 0..((screen_height() - y_middle) / (scale * y_scale) + 1f32) as i32 {
            //down
            draw_line(
                x_middle + 3f32,
                i as f32 * scale * y_scale + y_middle,
                x_middle - 3f32,
                i as f32 * scale * y_scale + y_middle,
                1f32,
                GREEN,
            );
        }
        for i in 0..((y_middle + 1f32) / (scale * y_scale) + 1f32) as i32 {
            //up
            draw_line(
                x_middle + 3f32,
                -1f32 * i as f32 * y_scale * scale + y_middle,
                x_middle - 3f32,
                -1f32 * i as f32 * y_scale * scale + y_middle,
                1f32,
                GREEN,
            );
        }

        //x-axis
        draw_line(0f32, y_middle, screen_width(), y_middle, 1f32, GREEN); //x-axis
        //draw the tick marks
        for i in 0..((screen_width() - x_middle) / (scale * x_scale) + 1f32) as i32 {
            //right
            draw_line(
                i as f32 * scale * x_scale + x_middle,
                y_middle + 3f32,
                i as f32 * scale * x_scale + x_middle,
                y_middle - 3f32,
                1f32,
                GREEN,
            );
        }
        for i in 0..((screen_width() + x_middle) / (scale * x_scale) + 1f32) as i32 {
            //left
            draw_line(
                -1f32 * i as f32 * scale * x_scale + x_middle,
                y_middle + 3f32,
                -1f32 * i as f32 * scale * x_scale + x_middle,
                y_middle - 3f32,
                1f32,
                GREEN,
            );
        }
        let height = screen_height();
        let width = screen_width();
        //Start doing the graphing fr
        for y_pixel in 0..(height * dpi_scale) as i32 {
            let y_coord: f32 =
                corner1.1 + ((y_pixel as f32 / dpi_scale) / height) * (corner2.1 - corner1.1);
            for x_pixel in 0..(width * dpi_scale) as i32 {
                let x_coord: f32 =
                    ((x_pixel as f32 / dpi_scale) / width) * (corner2.0 - corner1.0) + corner1.0;

                // let side1 = y_coord*y_coord+x_coord*x_coord;
                // let side2 = 1f32;

                // let side1 = (x_coord*x_coord).sin()+(y_coord*y_coord).sin();
                // let side2 =1f32;

                // let side1 =
                //     x_coord.powi(7) - 3.0 * x_coord.powi(5) + 2.0 * x_coord.powi(2) - x_coord;
                // let side2 = y_coord;
                let dx = (corner2.0 - corner1.0) / width;   // math units per screen pixel in x
                let dy = (corner2.1 - corner1.1) / height;  // math units per screen pixel in y

                let val   = F(x_coord, y_coord);
                let up    = F(x_coord, y_coord + dy);
                let down  = F(x_coord, y_coord - dy);
                let left  = F(x_coord - dx, y_coord);
                let right = F(x_coord + dx, y_coord);
                
                let valsign = sign(val); 
                
                if (valsign != sign(up))
                    || (valsign != sign(down))
                    || (valsign != sign(left))
                    || (valsign != sign(right))
                {
                    draw_rectangle(
                        x_pixel as f32 / dpi_scale,
                        y_pixel as f32 / dpi_scale,
                        1f32 / dpi_scale,
                        1f32 / dpi_scale,
                        BLUE,
                    );
                }
            }
        }

        next_frame().await //draw the frame I think
    }
}

fn F(x: f32, y: f32) -> f32 {
    (x*x).sin() + (y*y).sin() - 1f32
}

fn sign(v: f32) -> i8 {
    if v > 0.0 { 1 }
    else if v < 0.0 { -1 }
    else { 0 } // treat exact (or near) zero separately if you like
}