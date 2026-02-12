use macroquad::prelude::*;
pub use macroquad::ui::*;
use rayon::prelude::*;
use std::default::Default;

fn window_conf() -> Conf {
    Conf {
        window_title: "Main Window".to_string(),
        high_dpi: false,
        fullscreen: true,
        ..Default::default()
    }
}

#[inline(always)]
fn F(x: f32, y: f32) -> f32 {
    const SCALE: f32 = 0.1;
    const INV_SCALE: f32 = 10.0;
    SCALE * (y.abs().ln() * INV_SCALE).sin() - SCALE * (x.abs().ln() * INV_SCALE).cos()
}

// Glow/bloom effect - samples surrounding pixels
#[inline(always)]
fn apply_glow(x: usize, y: usize, img_data: &[[u8; 4]], width: usize, height: usize, radius: i32) -> [u8; 4] {
    let mut r = 0.0f32;
    let mut g = 0.0f32;
    let mut b = 0.0f32;
    let mut count = 0.0f32;
    
    for dy in -radius..=radius {
        for dx in -radius..=radius {
            let nx = x as i32 + dx;
            let ny = y as i32 + dy;
            if nx >= 0 && nx < width as i32 && ny >= 0 && ny < height as i32 {
                let idx = ny as usize * width + nx as usize;
                let pixel = img_data[idx];
                let dist = ((dx * dx + dy * dy) as f32).sqrt();
                let weight = (-dist * 0.5).exp();
                r += pixel[0] as f32 * weight;
                g += pixel[1] as f32 * weight;
                b += pixel[2] as f32 * weight;
                count += weight;
            }
        }
    }
    
    [(r / count) as u8, (g / count) as u8, (b / count) as u8, 255]
}

#[macroquad::main(window_conf)]
async fn main() {
    // Effect toggles
    let mut glow_enabled = false;
    let mut chromatic_aberration = false;
    let mut scanlines = false;
    let mut vignette = true;
    let mut scale = 20.0f32;
    
    // Wait until window is properly initialized with valid dimensions
    loop {
        next_frame().await;
        let w = screen_width();
        let h = screen_height();
        if w > 0.0 && h > 0.0 {
            let test_img_w = (w * 1.0) as u16;
            let test_img_h = (h * 1.0) as u16;
            if test_img_w > 0 && test_img_h > 0 {
                break;
            }
        }
    }
    
    let (width, height) = (screen_width() as f32, screen_height() as f32);
    
    // Safety check - ensure dimensions are valid
    if width <= 0.0 || height <= 0.0 {
        panic!("Invalid screen dimensions: {}x{}", width, height);
    }
    
    let dpi_scale = 1.0f32;
    let (img_width, img_height) = ((width * dpi_scale) as u16, (height * dpi_scale) as u16);
    
    // Additional safety check for integer dimensions
    if img_width == 0 || img_height == 0 {
        panic!("Invalid image dimensions after conversion: {}x{}", img_width, img_height);
    }
    
    let inv_dpi = 1.0 / dpi_scale;
    let inv_width = 1.0 / width;
    let inv_height = 1.0 / height;
    
    let mut img = Image::gen_image_color(img_width, img_height, BLACK);
    let tex = Texture2D::from_image(&img);
    
    // Pre-compute vignette lookup table for optimization
    // Cast to usize before multiplying to prevent u16 overflow
    let mut vignette_lut = vec![1.0f32; img_width as usize * img_height as usize];
    for y in 0..img_height as usize {
        let t_y = (y as f32 * inv_dpi) * inv_height;
        let dy = t_y - 0.5;
        for x in 0..img_width as usize {
            let t_x = (x as f32 * inv_dpi) * inv_width;
            let dx = t_x - 0.5;
            let dist = (dx * dx + dy * dy).sqrt();
            let idx = y * img_width as usize + x;
            vignette_lut[idx] = 1.0 - (dist * 1.5).min(1.0);
        }
    }

    loop {
        let delta_time = get_frame_time();
        scale += scale * delta_time;
        
        // Toggle effects with keys
        if is_key_pressed(macroquad::prelude::KeyCode::Key1) { glow_enabled = !glow_enabled; }
        if is_key_pressed(macroquad::prelude::KeyCode::Key2) { chromatic_aberration = !chromatic_aberration; }
        if is_key_pressed(macroquad::prelude::KeyCode::Key3) { scanlines = !scanlines; }
        if is_key_pressed(macroquad::prelude::KeyCode::Key4) { vignette = !vignette; }
        
        let inv_scale = 1.0 / scale;
        let half_width_scaled = width * inv_scale * 0.5;
        let half_height_scaled = height * inv_scale * 0.5;
        let range_x = half_width_scaled * 2.0;
        let range_y = -half_height_scaled * 2.0;
        
        let img_data = img.get_image_data_mut();
        img_data.par_chunks_mut(img_width as usize)
            .enumerate()
            .for_each(|(y_pixel, row)| {
                let yp = y_pixel as f32 * inv_dpi;
                let t_y = yp * inv_height;
                let y_coord = half_height_scaled + t_y * range_y;
                
                for x_pixel in 0..img_width as usize {
                    let xp = x_pixel as f32 * inv_dpi;
                    let t_x = xp * inv_width;
                    let x_coord = -half_width_scaled + t_x * range_x;

                    let val = F(x_coord, y_coord);
                    let abs_val = val.abs();
                    let intensity = (-abs_val).exp2();
                    
                    let color = if val < 0.0 {
                        Color { r: 0.0, g: 0.0, b: intensity, a: 1.0 }
                    } else {
                        Color { r: intensity, g: 0.0, b: 0.0, a: 1.0 }
                    };
                    
                    row[x_pixel] = color.into();
                }
            });
        
        // Apply vignette after parallel processing for better performance
        if vignette {
            let img_data = img.get_image_data_mut();
            for y in 0..img_height as usize {
                for x in 0..img_width as usize {
                    let idx = y * img_width as usize + x;
                    let vignette_strength = vignette_lut[idx];
                    let pixel = &mut img_data[idx];
                    pixel[0] = (pixel[0] as f32 * vignette_strength) as u8;
                    pixel[1] = (pixel[1] as f32 * vignette_strength) as u8;
                    pixel[2] = (pixel[2] as f32 * vignette_strength) as u8;
                }
            }
        }
        
        // Apply glow (slower, best as optional)
        if glow_enabled {
            let img_data_copy: Vec<[u8; 4]> = img.get_image_data().to_vec();
            let img_data = img.get_image_data_mut();
            for y in 0..img_height as usize {
                for x in 0..img_width as usize {
                    let idx = y * img_width as usize + x;
                    img_data[idx] = apply_glow(x, y, &img_data_copy, img_width as usize, img_height as usize, 2);
                }
            }
        }
        
        tex.update(&img);
        
        // Chromatic aberration effect - creates color fringing by drawing shifted copies
        if chromatic_aberration {
            let offset = 0.005 * width;
            
            // Draw main texture at center with full brightness
            draw_texture_ex(&tex, 0.0, 0.0, WHITE, 
                DrawTextureParams { dest_size: Some(vec2(width, height)), ..Default::default() });
            
            // Draw semi-transparent copy shifted right (creates red fringe on right edge)
            draw_texture_ex(&tex, offset, 0.0, Color::new(1.0, 1.0, 1.0, 0.2), 
                DrawTextureParams { dest_size: Some(vec2(width, height)), ..Default::default() });
            
            // Draw semi-transparent copy shifted left (creates blue fringe on left edge)
            draw_texture_ex(&tex, -offset, 0.0, Color::new(1.0, 1.0, 1.0, 0.2), 
                DrawTextureParams { dest_size: Some(vec2(width, height)), ..Default::default() });
        } else {
            // Normal rendering without chromatic aberration
            draw_texture_ex(&tex, 0.0, 0.0, WHITE, 
                DrawTextureParams { dest_size: Some(vec2(width, height)), ..Default::default() });
        }
        
        // Scanlines overlay
        if scanlines {
            for y in (0..height as i32).step_by(4) {
                draw_line(0.0, y as f32, width, y as f32, 1.0, Color::new(0.0, 0.0, 0.0, 0.3));
            }
        }
        
        draw_text("1:Glow 2:ChromaAb 3:Scanlines 4:Vignette", 10.0, 30.0, 20.0, WHITE);
        draw_fps();

        next_frame().await
    }
}
