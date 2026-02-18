use macroquad::prelude::*;
pub use macroquad::ui::*;
use std::default::Default;
use std::thread::yield_now;
use std::f64::consts::PI;

#[derive(Debug, Clone, Copy, PartialEq)]
enum AntiAliasingMode {
    None,
    EdgeDetection,
    SSAA2x,
    SSAA4x,
}

fn window_conf() -> Conf {
    //the config for the main window
    Conf {
        window_title: "Main Window".to_string(),
        high_dpi: true,
        fullscreen: false,
        ..Default::default()
    }
}
fn F(x: f64, y: f64) -> f64 {
    // y-(x+4.0)*(x-3.0)*(x-3.0)*(x+1.0)*(x+1.0)*(x+1.0)
    // let sx = x.sin();
    // let sy = y.sin();
    // sx*sy
    //sx * sx + cy * cy - 1_f64
    // x*x*x*x - 2.0 * x*x*x - 15.0 * x*x - y
    // x.sqrt() - y
    // x.ln() - y
     (x*x).sin()*(y*y).sin()
    //x.sin()+y.sin()-(x*y).sin() //polka-dot
    // (x/5.0).sin()*(y/5.0).sin()-0.7
    //(PI * x / 2.0).cos().powf(0.86016)+(PI * y / 2.0).cos().powf(0.86016)-1.0
    // x.cos().powf(10.0)+y.cos().powf(10.0)-1.0
    // x.sin()*y.sin()*x*x*y*y/(x/y).cos()*(x*y).sin() //wird
    // x*x+y*y-2.0
    // (x * x).sin() + (y * y).sin() // - 1f64
    // x * x - y
    // x*x+y*y-1f32
    // 3.0 * y.sqrt() - x - 2.0
    // y*x
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
async fn main() {
    //todo: fix coordinate system
    let (width, height) = (screen_width() as f64, screen_height() as f64);
    let mut scale = 20f64;
    let mut max: (f64, f64);
    let mut min: (f64, f64);
    let dpi_scale = screen_dpi_scale() as f64;
    let mut x_center = 0f64;
    let mut y_center = 0f64;
    // Physical pixel dimensions for high-res rendering
    let (img_width, img_height) = ((width * dpi_scale) as u16, (height * dpi_scale) as u16);
    let mut img = Image::gen_image_color(
        img_width,
        img_height,
        BLACK,
    ); //an image to manipulate
    let mut tex = Texture2D::from_image(&img); // reserve memory in the gpu
    let axis_color: Color = GOLD;
    let mut aa_mode = AntiAliasingMode::None;

    loop {
        //handle zoom
        let delta_time = get_frame_time() as f64;
        let mouse_x = mouse_position_local().x as f64 * width / 2.0 / scale + x_center;
        let mouse_y = mouse_position_local().y as f64 * height / 2.0 / scale * -1f64 + y_center;
        if mouse_wheel().1 != 0.0 {
            let delta_scale = scale * mouse_wheel().1 as f64 * delta_time * 0.1;
            if scale >= 1f64 {
                scale += delta_scale;
            } else {
                scale = 1f64;
            }
            x_center = mouse_x - mouse_position_local().x as f64 * width / 2.0 / scale;
            y_center = mouse_y - mouse_position_local().y as f64 * height / 2.0 / scale * -1.0;
        }

        //handle movement
        if is_mouse_button_down(MouseButton::Left) {
            //mouse movements
            let mouse_delta = mouse_delta_position();
            x_center += mouse_delta.x as f64 / scale * width / 2.0;
            // On MacOS, the y-axis is inverted, so we need to add instead of subtract
            #[cfg(target_os = "macos")]
            {
                y_center += mouse_delta.y as f64 / scale * height / 2.0;
            }
            #[cfg(not(target_os = "macos"))]
            {
                y_center -= mouse_delta.y as f64 / scale * height / 2.0;
            }
        }
        max = (
            width / scale / 2.0 + x_center,
            height / scale / 2.0 + y_center,
        );
        min = (
            -1.0 * width / scale / 2.0 + x_center,
            -1.0 * height / scale / 2.0 + y_center,
        );

        //Start doing the graphing fr
        match aa_mode {
            AntiAliasingMode::None => {
                // Original simple rendering
                for y_pixel in 0..img_height as i32 {
                    let yp = y_pixel as f64 / dpi_scale;
                    let t_y = yp / height;
                    let y_coord: f64 = max.1 + t_y * (min.1 - max.1);
                    for x_pixel in 0..img_width as i32 {
                        let xp = x_pixel as f64 / dpi_scale;
                        let t_x = xp / width;
                        let x_coord: f64 = min.0 + t_x * (max.0 - min.0);

                        let val = F(x_coord, y_coord);
                        let color = if sign(val) < 0 {
                            Color {
                                r: 0f32,
                                g: 0f32,
                                b: 2f64.powf(val.abs() * -1f64) as f32,
                                a: 1f32,
                            }
                        } else {
                            Color {
                                b: 0f32,
                                g: 0f32,
                                r: 2f64.powf(val.abs() * -1f64) as f32,
                                a: 1f32,
                            }
                        };
                        img.set_pixel(x_pixel as u32, y_pixel as u32, color);
                    }
                }
            }
            AntiAliasingMode::EdgeDetection => {
                // Edge detection anti-aliasing
                let dx = (max.0 - min.0) / (width * dpi_scale);
                let dy = (max.1 - min.1) / (height * dpi_scale);
                
                for y_pixel in 0..img_height as i32 {
                    let yp = y_pixel as f64 / dpi_scale;
                    let t_y = yp / height;
                    let y_coord: f64 = max.1 + t_y * (min.1 - max.1);
                    for x_pixel in 0..img_width as i32 {
                        let xp = x_pixel as f64 / dpi_scale;
                        let t_x = xp / width;
                        let x_coord: f64 = min.0 + t_x * (max.0 - min.0);

                        let val = F(x_coord, y_coord);
                        let up = F(x_coord, y_coord + dy);
                        let down = F(x_coord, y_coord - dy);
                        let left = F(x_coord - dx, y_coord);
                        let right = F(x_coord + dx, y_coord);

                        let valsign = sign(val);
                        
                        // Check if we're on an edge
                        let is_edge = (valsign != sign(up))
                            || (valsign != sign(down))
                            || (valsign != sign(left))
                            || (valsign != sign(right));

                        let color = if is_edge {
                            // Draw edge in white for better visibility
                            WHITE
                        } else if sign(val) < 0 {
                            Color {
                                r: 0f32,
                                g: 0f32,
                                b: 2f64.powf(val.abs() * -1f64) as f32,
                                a: 1f32,
                            }
                        } else {
                            Color {
                                b: 0f32,
                                g: 0f32,
                                r: 2f64.powf(val.abs() * -1f64) as f32,
                                a: 1f32,
                            }
                        };
                        img.set_pixel(x_pixel as u32, y_pixel as u32, color);
                    }
                }
            }
            AntiAliasingMode::SSAA2x => {
                // 2x2 Supersampling
                for y_pixel in 0..img_height as i32 {
                    let yp = y_pixel as f64 / dpi_scale;
                    let t_y = yp / height;
                    let y_coord: f64 = max.1 + t_y * (min.1 - max.1);
                    for x_pixel in 0..img_width as i32 {
                        let xp = x_pixel as f64 / dpi_scale;
                        let t_x = xp / width;
                        let x_coord: f64 = min.0 + t_x * (max.0 - min.0);

                        let pixel_width = (max.0 - min.0) / (width * dpi_scale);
                        let pixel_height = (max.1 - min.1) / (height * dpi_scale);
                        
                        // Sample 4 points in a 2x2 grid
                        let mut r_sum = 0f32;
                        let g_sum = 0f32;
                        let mut b_sum = 0f32;
                        
                        for sy in 0..2 {
                            for sx in 0..2 {
                                let sample_x = x_coord + (sx as f64 - 0.5) * pixel_width * 0.5;
                                let sample_y = y_coord + (sy as f64 - 0.5) * pixel_height * 0.5;
                                let val = F(sample_x, sample_y);
                                
                                if sign(val) < 0 {
                                    b_sum += 2f64.powf(val.abs() * -1f64) as f32;
                                } else {
                                    r_sum += 2f64.powf(val.abs() * -1f64) as f32;
                                }
                            }
                        }
                        
                        let color = Color {
                            r: r_sum / 4.0,
                            g: g_sum / 4.0,
                            b: b_sum / 4.0,
                            a: 1f32,
                        };
                        img.set_pixel(x_pixel as u32, y_pixel as u32, color);
                    }
                }
            }
            AntiAliasingMode::SSAA4x => {
                // 4x4 Supersampling (higher quality but slower)
                for y_pixel in 0..img_height as i32 {
                    let yp = y_pixel as f64 / dpi_scale;
                    let t_y = yp / height;
                    let y_coord: f64 = max.1 + t_y * (min.1 - max.1);
                    for x_pixel in 0..img_width as i32 {
                        let xp = x_pixel as f64 / dpi_scale;
                        let t_x = xp / width;
                        let x_coord: f64 = min.0 + t_x * (max.0 - min.0);

                        let pixel_width = (max.0 - min.0) / (width * dpi_scale);
                        let pixel_height = (max.1 - min.1) / (height * dpi_scale);
                        
                        // Sample 16 points in a 4x4 grid
                        let mut r_sum = 0f32;
                        let g_sum = 0f32;
                        let mut b_sum = 0f32;
                        
                        for sy in 0..4 {
                            for sx in 0..4 {
                                let sample_x = x_coord + (sx as f64 - 1.5) * pixel_width * 0.25;
                                let sample_y = y_coord + (sy as f64 - 1.5) * pixel_height * 0.25;
                                let val = F(sample_x, sample_y);
                                
                                if sign(val) < 0 {
                                    b_sum += 2f64.powf(val.abs() * -1f64) as f32;
                                } else {
                                    r_sum += 2f64.powf(val.abs() * -1f64) as f32;
                                }
                            }
                        }
                        
                        let color = Color {
                            r: r_sum / 16.0,
                            g: g_sum / 16.0,
                            b: b_sum / 16.0,
                            a: 1f32,
                        };
                        img.set_pixel(x_pixel as u32, y_pixel as u32, color);
                    }
                }
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

        let yc_pix = height / 2.0 - (-1.0 * y_center * scale);
        let xc_pix = width / 2.0 - (x_center * scale);

        //y-axis
        draw_line(
            xc_pix as f32,
            0f32,
            xc_pix as f32,
            height as f32,
            1f32,
            axis_color,
        );
        //draw the tick marks
        for i in 0..((height - yc_pix) / scale + 1f64) as i32 {
            //down
            draw_line(
                (xc_pix + 3f64) as f32,
                (i as f64 * scale + yc_pix) as f32,
                xc_pix as f32 - 3f32,
                (i as f64 * scale + yc_pix) as f32,
                1f32,
                axis_color,
            );
        }
        for i in 0..(yc_pix / scale + 1f64) as i32 {
            //up
            draw_line(
                (xc_pix + 3f64) as f32,
                (-1f64 * i as f64 * scale + yc_pix) as f32,
                (xc_pix - 3f64) as f32,
                (-1f64 * i as f64 * scale + yc_pix) as f32,
                1f32,
                axis_color,
            );
        }

        //x-axis
        draw_line(
            0f32,
            yc_pix as f32,
            width as f32,
            yc_pix as f32,
            1f32,
            axis_color,
        ); //x-axis
        //draw the tick marks
        for i in 0..((width - xc_pix) / scale + 1f64) as i32 {
            //right
            draw_line(
                (i as f64 * scale + xc_pix) as f32,
                (yc_pix + 3f64) as f32,
                (i as f64 * scale + xc_pix) as f32,
                (yc_pix - 3f64) as f32,
                1f32,
                axis_color,
            );
        }
        for i in 0..(xc_pix / scale + 1f64) as i32 {
            //left
            draw_line(
                (-1f64 * i as f64 * scale + xc_pix) as f32,
                (yc_pix + 3f64) as f32,
                (-1f64 * i as f64 * scale + xc_pix) as f32,
                (yc_pix - 3f64) as f32,
                1f32,
                axis_color,
            );
        }

        draw_text_in_corner(&max, &min, &scale,&mouse_x,&mouse_y,&x_center,&y_center); //draw text
        
        // Anti-aliasing UI dropdown
        let ui_width = 200f32;
        let ui_height = 120f32;
        let ui_x = width as f32 - ui_width - 10f32;
        let ui_y = 10f32;
        
        // Draw background
        draw_rectangle(ui_x, ui_y, ui_width, ui_height, Color::new(0.2, 0.2, 0.2, 0.8));
        
        // Title
        draw_text("Anti-Aliasing", ui_x + 10f32, ui_y + 25f32, 20.0, WHITE);
        
        // Mode options
        let modes = [
            (AntiAliasingMode::None, "None"),
            (AntiAliasingMode::EdgeDetection, "Edge Detection"),
            (AntiAliasingMode::SSAA2x, "SSAA 2x"),
            (AntiAliasingMode::SSAA4x, "SSAA 4x"),
        ];
        
        for (i, (mode, name)) in modes.iter().enumerate() {
            let button_y = ui_y + 40f32 + (i as f32 * 20f32);
            let is_selected = aa_mode == *mode;
            
            // Draw selection indicator
            if is_selected {
                draw_text(">", ui_x + 5f32, button_y + 15f32, 16.0, YELLOW);
            }
            
            // Draw button text
            let text_color = if is_selected { YELLOW } else { WHITE };
            draw_text(name, ui_x + 20f32, button_y + 15f32, 16.0, text_color);
            
            // Check for click
            let mouse_pos = mouse_position();
            if is_mouse_button_pressed(MouseButton::Left) {
                if mouse_pos.0 >= ui_x && mouse_pos.0 <= ui_x + ui_width
                    && mouse_pos.1 >= button_y && mouse_pos.1 <= button_y + 20f32 {
                    aa_mode = *mode;
                }
            }
        }

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

fn draw_text_in_corner(corner1: &(f64, f64), corner2: &(f64, f64), scale: &f64,mouse_x:&f64,mouse_y:&f64,x_center:&f64,y_center:&f64) {
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
    draw_text(
        format!(
            "Mouse:{},{}",
            mouse_x,
            mouse_y
        )
            .as_str(),
        10.0,
        95.0,
        25.0,
        WHITE,
    );
    draw_text(
        format!(
            "Center:{},{}",
            x_center,
            y_center
        )
            .as_str(),
        10.0,
        115.0,
        25.0,
        WHITE,
    );
}
