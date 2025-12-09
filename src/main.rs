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
    let (width, height) = (3840f64, 2160f64);
    let mut scale = 150f64;
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
        img.export_png("image.png");
        


        next_frame().await //draw the frame I think
    
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