use macroquad::miniquad::window::dpi_scale;
use macroquad::prelude::*;
use std::default::Default;

fn window_conf() -> Conf {
    //the config for the main window
    Conf {
        window_title: "Main Window".to_string(),
        high_dpi: true,
        fullscreen: true,
        ..Default::default()
    }
}
fn F(x: f64, y: f64) -> f64 {
    // y-(x+4.0)*(x-3.0)*(x-3.0)*(x+1.0)*(x+1.0)*(x+1.0)
    // let sx = x.sin();
    // let cy = x.cos();
    // sx * sx + cy * cy - 1_f64
    (x * x).sin() + (y * y).sin() //- 1f64
    // x * x - y
    // x*x+y*y-1f32
    // y*x.cos()-x*y.sin()
    // x.sin()-y
    // x*x*x*x*x+7f64*x*x*x+148f64*x*x+y*x+1f64
    // 2.8 * x
    //     * x
    //     * (x * x * (2.5 * x * x + y * y - 2f64)
    //         + 1.2 * y * y * (y * (3f64 * y - 0.75) - 6.0311)
    //         + 3.09)
    //     + 0.98 * y * y * ((y * y - 3.01) * y * y + 3f64)
    //     - 1.005 //dont show in class

}
#[macroquad::main(window_conf)] //pass with config in
async fn main() { //todo: fix coordinate system
    let (width, height) = (screen_width() as f64, screen_height() as f64);
    let mut scale = 20f64;
    let mut corner1: (f64, f64) = (-1f64 * width / (2f64 * scale), height / (2f64 * scale));
    let mut corner2: (f64, f64) = ( width / (2f64 * scale), -1f64 * height / (2f64 * scale));
    let dpi_scale = screen_dpi_scale() as f64;
    let mut x_shift = 0f64;
    let mut y_shift = 0f64;
    let mut img = Image::gen_image_color(
        width as u16 * dpi_scale as u16,
        height as u16 * dpi_scale as u16,
        BLACK,
    ); //an image to manipulate
    let mut tex = Texture2D::from_image(&img);

    loop {
        //set_fullscreen(true); //force into fullscreen; not necessary
        //handle zoom
        let delta_time = get_frame_time() as f64;
        scale += scale * mouse_wheel().1 as f64 * delta_time * 0.5;
        if scale < 1f64 {
            scale = 1f64;
        }
        //handle movement
        if is_mouse_button_down(MouseButton::Left) { //mouse movements
            let mouse_delta = mouse_delta_position();
            x_shift+=mouse_delta.x as f64 /scale * width / 2.0;
            y_shift+=mouse_delta.y as f64 / scale * height / 2.0;
        }
        if is_key_down(KeyCode::Right) {
            x_shift += 200f64 * delta_time / scale;
        }
        if is_key_down(KeyCode::Left) {
            x_shift -= 200f64 * delta_time / scale;
        }
        if is_key_down(KeyCode::Down) {
            y_shift += 200f64 * delta_time / scale;
        }
        if is_key_down(KeyCode::Up) {
            y_shift -= 200f64 * delta_time / scale;
        }

        //update corners
        corner1 = (
            -1f64 * width / (2f64 * scale) + x_shift,
            height / (2f64 * scale) - y_shift,
        );
        corner2 = (
            //
            width/ (2f64 * scale) + x_shift,
            -1f64 * height / (2f64 * scale) - y_shift,
        );
        let x_scale = (width / scale) / (corner2.0 - corner1.0);
        let y_scale = (height / scale) / (corner1.1 - corner2.1);

        // let dx = (corner2.0 - corner1.0) / width; // math units per screen pixel in x
        // let dy = (corner2.1 - corner1.1) / height; // math units per screen pixel in y

        //Start doing the graphing fr
        for y_pixel in 0..(height * dpi_scale) as i32 {
            let y_coord: f64 =
                corner1.1 + ((y_pixel as f64 / dpi_scale) / height) * (corner2.1 - corner1.1);
            for x_pixel in 0..(width * dpi_scale) as i32 {
                let x_coord: f64 =
                    ((x_pixel as f64 / dpi_scale) / width) * (corner2.0 - corner1.0) + corner1.0;
                let val = F(x_coord, y_coord);
                if sign(val) < 0 {
                    let color = Color {
                        r: 0f32,
                        g: 0f32,
                        b: 2f64.powf(val.abs() * -1f64) as f32,
                        a: 1f32,
                    };
                    img.set_pixel(x_pixel as u32, y_pixel as u32, color)
                } else {
                    let color = Color {
                        b: 0f32,
                        g: 0f32,
                        r: 2f64.powf(val.abs() * -1f64) as f32,
                        a: 1f32,
                    };
                    img.set_pixel(x_pixel as u32, y_pixel as u32, color)
                }

                // let val = F(x_coord, y_coord);
                // let up = F(x_coord, y_coord + dy);
                // let down = F(x_coord, y_coord - dy);
                // let left = F(x_coord - dx, y_coord);
                // let right = F(x_coord + dx, y_coord);

                // // let valsign = sign(val);
                // //
                // // if (valsign != sign(up))
                // //     || (valsign != sign(down))
                // //     || (valsign != sign(left))
                // //     || (valsign != sign(right))
                // // {
                //     draw_rectangle(
                //         x_pixel as f32 / dpi_scale,
                //         y_pixel as f32 / dpi_scale,
                //         1f32 / dpi_scale,
                //         1f32 / dpi_scale,
                //         BLUE,
                //     );
                //}
            }
        }
        tex.update(&img);
        draw_texture_ex(
            &tex,
            0f32,
            0f32,
            WHITE,
            DrawTextureParams {
                dest_size: Some(vec2(width as f32, height as f32)),
                ..Default::default()
            },
        );

        //Draw the axes
        let x_middle = (-1f64 * corner1.0) * scale * x_scale;
        let y_middle = (corner1.1) * scale * y_scale;

        //y-axis
        draw_line(
            x_middle as f32,
            0f32,
            x_middle as f32,
            height as f32,
            1f32,
            GREEN,
        );
        //draw the tick marks
        for i in 0..((height - y_middle) / (scale * y_scale) + 1f64) as i32 {
            //down
            draw_line(
                (x_middle + 3f64) as f32,
                (i as f64 * scale * y_scale + y_middle) as f32,
                x_middle as f32 - 3f32,
                (i as f64 * scale * y_scale + y_middle) as f32,
                1f32,
                GREEN,
            );
        }
        for i in 0..((y_middle + 1f64) / (scale * y_scale) + 1f64) as i32 {
            //up
            draw_line(
                (x_middle + 3f64) as f32,
                (-1f64 * i as f64 * y_scale * scale + y_middle) as f32,
                (x_middle - 3f64) as f32,
                (-1f64 * i as f64 * y_scale * scale + y_middle) as f32,
                1f32,
                GREEN,
            );
        }

        //x-axis
        draw_line(
            0f32,
            y_middle as f32,
            width as f32,
            y_middle as f32,
            1f32,
            GREEN,
        ); //x-axis
        //draw the tick marks
        for i in 0..((width - x_middle) / (scale * x_scale) + 1f64) as i32 {
            //right
            draw_line(
                (i as f64 * scale * x_scale + x_middle) as f32,
                (y_middle + 3f64) as f32,
                (i as f64 * scale * x_scale + x_middle) as f32,
                (y_middle - 3f64) as f32,
                1f32,
                GREEN,
            );
        }
        for i in 0..((width + x_middle) / (scale * x_scale) + 1f64) as i32 {
            //left
            draw_line(
                (-1f64 * i as f64 * scale * x_scale + x_middle) as f32,
                 (y_middle + 3f64) as f32,
                  (-1f64 * i as f64 * scale * x_scale + x_middle) as f32,
                   (y_middle - 3f64) as f32,
                1f32,
                GREEN,
            );
        }

        draw_text_in_corner(&corner1, &corner2, &scale); //draw text

        next_frame().await //draw the frame I think
    }
}


fn sign(v: f64) -> i8 {
    if v > 0.0 {
        1
    } else if v < 0.0 {
        -1
    } else {
        0
    } // treat exact (or near) zero separately if you like
}

fn draw_text_in_corner(corner1: &(f64,f64), corner2: &(f64,f64), scale: &f64) {
    draw_fps(); //todo: draw above graph but semi-transparent
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
}