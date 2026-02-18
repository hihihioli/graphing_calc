use macroquad::prelude::*;
use std::default::Default;
use std::sync::{Arc, Mutex};
use std::thread;

mod ui_window;

// ─────────────────────────────────────────────────────────────────────────────
// SHARED VERTEX SHADER
// ─────────────────────────────────────────────────────────────────────────────

const VERTEX: &str = r#"#version 100
precision highp float;
attribute vec3 position;
attribute vec2 texcoord;
varying vec2 uv;
uniform mat4 Model;
uniform mat4 Projection;
void main() {
    gl_Position = Projection * Model * vec4(position, 1.0);
    uv = texcoord;
}
"#;

// ─────────────────────────────────────────────────────────────────────────────
// SCENE SHADER
// ─────────────────────────────────────────────────────────────────────────────

const SCENE_FRAG: &str = r#"#version 100
precision mediump float;
varying vec2 uv;

uniform vec2 center;
uniform vec2 half_extent;
uniform float a_phase;
uniform float zoom_phase;
uniform float hue_shift;
uniform float color_static;
uniform float static_hue_neg;
uniform float static_hue_pos;
uniform float hue_shift_2;

vec3 hsl2rgb(float h, float s, float l) {
    float c = (1.0 - abs(2.0 * l - 1.0)) * s;
    float m = l - c * 0.5;
    float h6 = h * 6.0;
    float x = c * (1.0 - abs(mod(h6, 2.0) - 1.0));
    vec3 rgb;
    if      (h6 < 1.0) rgb = vec3(c, x, 0.0);
    else if (h6 < 2.0) rgb = vec3(x, c, 0.0);
    else if (h6 < 3.0) rgb = vec3(0.0, c, x);
    else if (h6 < 4.0) rgb = vec3(0.0, x, c);
    else if (h6 < 5.0) rgb = vec3(x, 0.0, c);
    else                rgb = vec3(c, 0.0, x);
    return rgb + m;
}

void main() {
    float x_coord = center.x + (uv.x - 0.5) * half_extent.x * 2.0;
    float y_coord = center.y + (0.5 - uv.y) * half_extent.y * 2.0;

    float log_y = log(max(abs(y_coord), 1e-20)) * 10.0;
    float log_x = log(max(abs(x_coord), 1e-20)) * 10.0;

    float val = (sin(log_y - zoom_phase - a_phase)
               - cos(log_x - zoom_phase + a_phase)) * 0.5;

    float intensity = exp2(-abs(val));

    float hue = val < 0.0
        ? mod(hue_shift, 1.0)
        : mod(hue_shift_2, 1.0);
    if (color_static > 0.5) {
        hue = val < 0.0 ? static_hue_neg : static_hue_pos;
    }

    gl_FragColor = vec4(hsl2rgb(hue, 1.0, intensity * 0.5), 1.0);
}
"#;

// ─────────────────────────────────────────────────────────────────────────────
// POST-PROCESSING SHADERS
// ─────────────────────────────────────────────────────────────────────────────

const BRIGHT_FRAG: &str = r#"#version 100
precision lowp float;
varying vec2 uv;
uniform sampler2D Texture;
uniform float threshold;
void main() {
    vec4 color = texture2D(Texture, uv);
    float brightness = dot(color.rgb, vec3(0.2126, 0.7152, 0.0722));
    gl_FragColor = brightness > threshold ? color : vec4(0.0, 0.0, 0.0, 1.0);
}
"#;

const BLUR_FRAG: &str = r#"#version 100
precision lowp float;
varying vec2 uv;
uniform sampler2D Texture;
uniform vec2 direction;
void main() {
    vec3 result = texture2D(Texture, uv).rgb * 0.227027;
    result += (texture2D(Texture, uv + direction      ).rgb +
               texture2D(Texture, uv - direction      ).rgb) * 0.1945946;
    result += (texture2D(Texture, uv + direction * 2.0).rgb +
               texture2D(Texture, uv - direction * 2.0).rgb) * 0.1216216;
    result += (texture2D(Texture, uv + direction * 3.0).rgb +
               texture2D(Texture, uv - direction * 3.0).rgb) * 0.054054;
    result += (texture2D(Texture, uv + direction * 4.0).rgb +
               texture2D(Texture, uv - direction * 4.0).rgb) * 0.016216;
    gl_FragColor = vec4(result, 1.0);
}
"#;

// Combined bloom + TAA in one pass (eliminates an extra full-res render target)
const COMBINE_TAA_FRAG: &str = r#"#version 100
precision lowp float;
varying vec2 uv;
uniform sampler2D Texture;
uniform sampler2D _bloom_tex;
uniform sampler2D _history_tex;
uniform float bloom_intensity;
uniform float taa_alpha;
void main() {
    vec3 scene = texture2D(Texture, uv).rgb;
    vec3 bloom = texture2D(_bloom_tex, uv).rgb;
    vec3 current = scene + bloom * bloom_intensity;
    vec3 history = texture2D(_history_tex, uv).rgb;
    gl_FragColor = vec4(mix(history, current, taa_alpha), 1.0);
}
"#;

const COMBINE_BLOOM_FRAG: &str = r#"#version 100
precision lowp float;
varying vec2 uv;
uniform sampler2D Texture;
uniform sampler2D _bloom_tex;
uniform float bloom_intensity;
void main() {
    vec3 scene = texture2D(Texture, uv).rgb;
    vec3 bloom = texture2D(_bloom_tex, uv).rgb;
    gl_FragColor = vec4(scene + bloom * bloom_intensity, 1.0);
}
"#;

const FXAA_FRAG: &str = r#"#version 100
precision highp float;
varying vec2 uv;
uniform sampler2D Texture;
uniform sampler2D _bloom_tex;
uniform vec2 resolution;
uniform float bloom_intensity;

#define FXAA_REDUCE_MIN   (1.0/128.0)
#define FXAA_REDUCE_MUL   (1.0/8.0)
#define FXAA_SPAN_MAX     8.0

void main() {
    vec2 inverseVP = 1.0 / resolution;
    
    vec3 rgbNW = texture2D(Texture, uv + vec2(-1.0, -1.0) * inverseVP).rgb;
    vec3 rgbNE = texture2D(Texture, uv + vec2(1.0, -1.0) * inverseVP).rgb;
    vec3 rgbSW = texture2D(Texture, uv + vec2(-1.0, 1.0) * inverseVP).rgb;
    vec3 rgbSE = texture2D(Texture, uv + vec2(1.0, 1.0) * inverseVP).rgb;
    vec3 rgbM  = texture2D(Texture, uv).rgb;
    
    const vec3 luma = vec3(0.299, 0.587, 0.114);
    float lumaNW = dot(rgbNW, luma);
    float lumaNE = dot(rgbNE, luma);
    float lumaSW = dot(rgbSW, luma);
    float lumaSE = dot(rgbSE, luma);
    float lumaM  = dot(rgbM, luma);
    
    float lumaMin = min(lumaM, min(min(lumaNW, lumaNE), min(lumaSW, lumaSE)));
    float lumaMax = max(lumaM, max(max(lumaNW, lumaNE), max(lumaSW, lumaSE)));
    
    vec2 dir;
    dir.x = -((lumaNW + lumaNE) - (lumaSW + lumaSE));
    dir.y =  ((lumaNW + lumaSW) - (lumaNE + lumaSE));
    
    float dirReduce = max((lumaNW + lumaNE + lumaSW + lumaSE) * (0.25 * FXAA_REDUCE_MUL), FXAA_REDUCE_MIN);
    float rcpDirMin = 1.0 / (min(abs(dir.x), abs(dir.y)) + dirReduce);
    
    dir = min(vec2(FXAA_SPAN_MAX, FXAA_SPAN_MAX),
          max(vec2(-FXAA_SPAN_MAX, -FXAA_SPAN_MAX),
          dir * rcpDirMin)) * inverseVP;
    
    vec3 rgbA = 0.5 * (
        texture2D(Texture, uv + dir * (1.0/3.0 - 0.5)).rgb +
        texture2D(Texture, uv + dir * (2.0/3.0 - 0.5)).rgb);
    
    vec3 rgbB = rgbA * 0.5 + 0.25 * (
        texture2D(Texture, uv + dir * -0.5).rgb +
        texture2D(Texture, uv + dir * 0.5).rgb);
    
    float lumaB = dot(rgbB, luma);
    
    vec3 result;
    if (lumaB < lumaMin || lumaB > lumaMax) {
        result = rgbA;
    } else {
        result = rgbB;
    }
    
    vec3 bloom = texture2D(_bloom_tex, uv).rgb;
    gl_FragColor = vec4(result + bloom * bloom_intensity, 1.0);
}
"#;

// ─────────────────────────────────────────────────────────────────────────────
// HELPERS
// ─────────────────────────────────────────────────────────────────────────────

fn window_conf() -> Conf {
    Conf {
        window_title: "Graphing Calculator".to_string(),
        window_width: 1280,
        window_height: 720,
        high_dpi: true,
        fullscreen: false,
        ..Default::default()
    }
}

#[inline]
fn draw_fullscreen(texture: &Texture2D, w: f32, h: f32) {
    draw_texture_ex(texture, 0.0, 0.0, WHITE, DrawTextureParams {
        dest_size: Some(vec2(w, h)),
        ..Default::default()
    });
}

#[inline]
fn cam_for_target(target: Option<RenderTarget>, w: f32, h: f32) -> Camera2D {
    Camera2D {
        render_target: target,
        ..Camera2D::from_display_rect(Rect::new(0.0, 0.0, w, h))
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// MAIN
// ─────────────────────────────────────────────────────────────────────────────

const A_PHASE_RATE: f64 = 5.0 * std::f64::consts::LN_2;
const ZOOM_PHASE_RATE: f64 = 10.0 * std::f64::consts::LN_2;

#[macroquad::main(window_conf)]
async fn main() {
    // Wait a couple frames for window to be properly initialized at configured size
    next_frame().await;
    next_frame().await;
    
    let dpi_scale = screen_dpi_scale();
    let w = screen_width();
    let h = screen_height();
    let width = w as f64;
    let height = h as f64;
    
    // Actual pixel dimensions for render targets (accounting for high DPI)
    let pixel_w = (w * dpi_scale) as u32;
    let pixel_h = (h * dpi_scale) as u32;

    // Create shared parameters for UI window
    let graph_params = Arc::new(Mutex::new(ui_window::GraphParams::default()));
    
    // Spawn UI window in separate thread (not supported on macOS)
    #[cfg(not(target_os = "macos"))]
    {
        let graph_params_clone = graph_params.clone();
        thread::spawn(move || {
            ui_window::run_ui_window(graph_params_clone);
        });
    }

    let mut initial_scale = 20.0f64;
    let mut half_extent = vec2(
        (width / initial_scale / 2.0) as f32,
        (height / initial_scale / 2.0) as f32,
    );

    let mut center_x: f64 = 0.0;
    let mut center_y: f64 = 0.0;
    let mut time: f64 = 0.0;
    let mut hue_shift: f32 = 0.0;
    let mut hue_shift_2: f32 = 0.5;
    let mut dragging = false;
    let mut last_mouse = vec2(0.0, 0.0);

    // ── Render targets ──────────────────────────────────────────────────
    let mut scene_target = render_target(pixel_w, pixel_h);
    scene_target.texture.set_filter(FilterMode::Linear);

    let mut taa_ping = render_target(pixel_w, pixel_h);
    let mut taa_pong = render_target(pixel_w, pixel_h);
    taa_ping.texture.set_filter(FilterMode::Linear);
    taa_pong.texture.set_filter(FilterMode::Linear);

    let mut blur_w = pixel_w / 4;
    let mut blur_h = pixel_h / 4;
    let mut bright_target = render_target(blur_w, blur_h);
    let mut blur_ping = render_target(blur_w, blur_h);
    let mut blur_pong = render_target(blur_w, blur_h);
    bright_target.texture.set_filter(FilterMode::Linear);
    blur_ping.texture.set_filter(FilterMode::Linear);
    blur_pong.texture.set_filter(FilterMode::Linear);

    // ── Materials ───────────────────────────────────────────────────────
    let scene_mat = load_material(
        ShaderSource::Glsl { vertex: VERTEX, fragment: SCENE_FRAG },
        MaterialParams {
            uniforms: vec![
                UniformDesc::new("center",      UniformType::Float2),
                UniformDesc::new("half_extent", UniformType::Float2),
                UniformDesc::new("a_phase",     UniformType::Float1),
                UniformDesc::new("zoom_phase",  UniformType::Float1),
                UniformDesc::new("hue_shift",   UniformType::Float1),
                UniformDesc::new("color_static", UniformType::Float1),
                UniformDesc::new("static_hue_neg", UniformType::Float1),
                UniformDesc::new("static_hue_pos", UniformType::Float1),
                UniformDesc::new("hue_shift_2", UniformType::Float1),
            ],
            ..Default::default()
        },
    ).unwrap();

    let bright_mat = load_material(
        ShaderSource::Glsl { vertex: VERTEX, fragment: BRIGHT_FRAG },
        MaterialParams {
            uniforms: vec![UniformDesc::new("threshold", UniformType::Float1)],
            ..Default::default()
        },
    ).unwrap();

    let blur_mat = load_material(
        ShaderSource::Glsl { vertex: VERTEX, fragment: BLUR_FRAG },
        MaterialParams {
            uniforms: vec![UniformDesc::new("direction", UniformType::Float2)],
            ..Default::default()
        },
    ).unwrap();

    let combine_taa_mat = load_material(
        ShaderSource::Glsl { vertex: VERTEX, fragment: COMBINE_TAA_FRAG },
        MaterialParams {
            uniforms: vec![
                UniformDesc::new("bloom_intensity", UniformType::Float1),
                UniformDesc::new("taa_alpha", UniformType::Float1),
            ],
            textures: vec!["_bloom_tex".to_string(), "_history_tex".to_string()],
            ..Default::default()
        },
    ).unwrap();

    let combine_bloom_mat = load_material(
        ShaderSource::Glsl { vertex: VERTEX, fragment: COMBINE_BLOOM_FRAG },
        MaterialParams {
            uniforms: vec![UniformDesc::new("bloom_intensity", UniformType::Float1)],
            textures: vec!["_bloom_tex".to_string()],
            ..Default::default()
        },
    ).unwrap();

    let fxaa_mat = load_material(
        ShaderSource::Glsl { vertex: VERTEX, fragment: FXAA_FRAG },
        MaterialParams {
            uniforms: vec![
                UniformDesc::new("bloom_intensity", UniformType::Float1),
                UniformDesc::new("resolution", UniformType::Float2),
            ],
            textures: vec!["_bloom_tex".to_string()],
            ..Default::default()
        },
    ).unwrap();

    // ── Tuning ──────────────────────────────────────────────────────────
    bright_mat.set_uniform("threshold", 0.1f32);
    combine_taa_mat.set_uniform("bloom_intensity", 0.5f32);
    combine_bloom_mat.set_uniform("bloom_intensity", 0.5f32);
    fxaa_mat.set_uniform("bloom_intensity", 0.5f32);
    fxaa_mat.set_uniform("resolution", vec2(pixel_w as f32, pixel_h as f32));
    scene_mat.set_uniform("static_hue_neg", 0.0f32);
    scene_mat.set_uniform("static_hue_pos", 2.0f32 / 3.0f32);
    scene_mat.set_uniform("half_extent", half_extent);

    let mut dir_h = vec2(1.0 / blur_w as f32, 0.0);
    let mut dir_v = vec2(0.0, 1.0 / blur_h as f32);
    let mut bw = blur_w as f32;
    let mut bh = blur_h as f32;

    let dummy = Texture2D::from_image(&Image::gen_image_color(1, 1, WHITE));
    let mut first_frame = true;

    let mut current_w = w;
    let mut current_h = h;
    let mut current_pixel_w = pixel_w;
    let mut current_pixel_h = pixel_h;
    let mut half_w = w / 2.0;
    let mut half_h = h / 2.0;
    let axis_color = Color::new(1.0, 1.0, 1.0, 0.3);
    let origin_color = Color::new(1.0, 1.0, 1.0, 0.5);

    loop {
        // Check for window resize
        let new_w = screen_width();
        let new_h = screen_height();
        if (new_w - current_w).abs() > 0.1 || (new_h - current_h).abs() > 0.1 {
            current_w = new_w;
            current_h = new_h;
            half_w = new_w / 2.0;
            half_h = new_h / 2.0;
            
            // Recalculate pixel dimensions for high DPI
            let dpi_scale = screen_dpi_scale();
            current_pixel_w = (current_w * dpi_scale) as u32;
            current_pixel_h = (current_h * dpi_scale) as u32;
            
            // Recreate render targets at new pixel size
            scene_target = render_target(current_pixel_w, current_pixel_h);
            scene_target.texture.set_filter(FilterMode::Linear);
            
            taa_ping = render_target(current_pixel_w, current_pixel_h);
            taa_pong = render_target(current_pixel_w, current_pixel_h);
            taa_ping.texture.set_filter(FilterMode::Linear);
            taa_pong.texture.set_filter(FilterMode::Linear);
            
            blur_w = current_pixel_w / 4;
            blur_h = current_pixel_h / 4;
            bright_target = render_target(blur_w, blur_h);
            blur_ping = render_target(blur_w, blur_h);
            blur_pong = render_target(blur_w, blur_h);
            bright_target.texture.set_filter(FilterMode::Linear);
            blur_ping.texture.set_filter(FilterMode::Linear);
            blur_pong.texture.set_filter(FilterMode::Linear);
            
            dir_h = vec2(1.0 / blur_w as f32, 0.0);
            dir_v = vec2(0.0, 1.0 / blur_h as f32);
            bw = blur_w as f32;
            bh = blur_h as f32;
            
            // Update FXAA resolution with pixel dimensions
            fxaa_mat.set_uniform("resolution", vec2(current_pixel_w as f32, current_pixel_h as f32));
            
            first_frame = true;
        }
        
        let dt = get_frame_time() as f64;
        time += dt;
        let mut camera_moved = false;

        // Read from UI controls
        let (aa_mode, bloom_enabled, color_static) = if let Ok(params) = graph_params.lock() {
            if (center_x - params.center_x).abs() > 1e-10 
               || (center_y - params.center_y).abs() > 1e-10 
               || (initial_scale - params.zoom).abs() > 1e-10 {
                center_x = params.center_x;
                center_y = params.center_y;
                initial_scale = params.zoom;
                half_extent = vec2(
                    (current_w as f64 / initial_scale / 2.0) as f32,
                    (current_h as f64 / initial_scale / 2.0) as f32,
                );
                scene_mat.set_uniform("half_extent", half_extent);
                camera_moved = true;
            }
            (params.aa_mode, params.bloom_enabled, params.color_static)
        } else {
            (ui_window::AAMode::TAA, true, false)
        };

        // Ensure zoom is always synced back to params
        if let Ok(mut params) = graph_params.lock() {
            if (params.zoom - initial_scale).abs() > 1e-10 {
                params.zoom = initial_scale;
            }
        }

        let scale_f32 = initial_scale as f32;

        hue_shift += 0.05 * dt as f32;
        if hue_shift >= 1.0 { hue_shift -= 1.0; }

        hue_shift_2 += 0.07 * dt as f32;
        if hue_shift_2 >= 1.0 { hue_shift_2 -= 1.0; }

        // ── Input ───────────────────────────────────────────────────────
        if is_mouse_button_down(MouseButton::Left) {
            let (mx, my) = mouse_position();
            let current = vec2(mx, my);
            if dragging {
                let md = current - last_mouse;
                if md.length() > 0.0 {
                    center_x -= md.x as f64 / initial_scale;
                    center_y -= md.y as f64 / initial_scale;
                    // Update shared params
                    if let Ok(mut params) = graph_params.lock() {
                        params.center_x = center_x;
                        params.center_y = center_y;
                    }
                    camera_moved = true;
                }
            }
            last_mouse = current;
            dragging = true;
        } else {
            dragging = false;
        }

        let a_phase    = (time * A_PHASE_RATE    % std::f64::consts::TAU) as f32;
        let zoom_phase = (time * ZOOM_PHASE_RATE % std::f64::consts::TAU) as f32;

        // ── Pass 1: Scene ───────────────────────────────────────────────
        scene_mat.set_uniform("center", vec2(center_x as f32, center_y as f32));
        scene_mat.set_uniform("a_phase", a_phase);
        scene_mat.set_uniform("zoom_phase", zoom_phase);
        scene_mat.set_uniform("hue_shift", hue_shift);
        scene_mat.set_uniform("color_static", if color_static { 1.0f32 } else { 0.0f32 });
        scene_mat.set_uniform("hue_shift_2", hue_shift_2);

        set_camera(&cam_for_target(Some(scene_target.clone()), current_w, current_h));
        gl_use_material(&scene_mat);
        draw_fullscreen(&dummy, current_w, current_h);
        gl_use_default_material();

        // ── Pass 2: Brightness extraction ───────────────────────────────
        set_camera(&cam_for_target(Some(bright_target.clone()), bw, bh));
        gl_use_material(&bright_mat);
        draw_fullscreen(&scene_target.texture, bw, bh);
        gl_use_default_material();

        // ── Pass 3: Ping-pong blur (2 passes) ───────────────────────────
        let mut src = &bright_target.texture;
        for _ in 0..2 {
            set_camera(&cam_for_target(Some(blur_ping.clone()), bw, bh));
            blur_mat.set_uniform("direction", dir_h);
            gl_use_material(&blur_mat);
            draw_fullscreen(src, bw, bh);
            gl_use_default_material();

            set_camera(&cam_for_target(Some(blur_pong.clone()), bw, bh));
            blur_mat.set_uniform("direction", dir_v);
            gl_use_material(&blur_mat);
            draw_fullscreen(&blur_ping.texture, bw, bh);
            gl_use_default_material();

            src = &blur_pong.texture;
        }

        // ── Pass 4: Combine bloom + AA ──────────────────────────────────
        let bloom_intensity = if bloom_enabled { 0.5f32 } else { 0.0f32 };
        set_camera(&cam_for_target(Some(taa_pong.clone()), current_w, current_h));
        
        match aa_mode {
            ui_window::AAMode::TAA => {
                let taa_alpha = if first_frame || camera_moved { 1.0f32 } else { 0.1f32 };
                combine_taa_mat.set_uniform("taa_alpha", taa_alpha);
                combine_taa_mat.set_uniform("bloom_intensity", bloom_intensity);
                combine_taa_mat.set_texture("_bloom_tex", blur_pong.texture.clone());
                combine_taa_mat.set_texture("_history_tex", taa_ping.texture.clone());
                gl_use_material(&combine_taa_mat);
            }
            ui_window::AAMode::FXAA => {
                fxaa_mat.set_uniform("bloom_intensity", bloom_intensity);
                fxaa_mat.set_uniform("resolution", vec2(current_pixel_w as f32, current_pixel_h as f32));
                fxaa_mat.set_texture("_bloom_tex", blur_pong.texture.clone());
                gl_use_material(&fxaa_mat);
            }
            ui_window::AAMode::None => {
                combine_bloom_mat.set_uniform("bloom_intensity", bloom_intensity);
                combine_bloom_mat.set_texture("_bloom_tex", blur_pong.texture.clone());
                gl_use_material(&combine_bloom_mat);
            }
        }
        
        draw_fullscreen(&scene_target.texture, current_w, current_h);
        gl_use_default_material();

        // ── Pass 5: Present ─────────────────────────────────────────────
        set_default_camera();
        draw_fullscreen(&taa_pong.texture, current_w, current_h);

        std::mem::swap(&mut taa_ping, &mut taa_pong);
        first_frame = false;

        // ── Axes overlay ────────────────────────────────────────────────
        let ox = half_w - center_x as f32 * scale_f32;
        let oy = half_h + center_y as f32 * scale_f32;

        if oy >= 0.0 && oy <= current_h {
            draw_line(0.0, oy, current_w, oy, 2.0, axis_color);
        }
        if ox >= 0.0 && ox <= current_w {
            draw_line(ox, 0.0, ox, current_h, 2.0, axis_color);
        }
        if ox >= 0.0 && ox <= current_w && oy >= 0.0 && oy <= current_h {
            draw_circle(ox, oy, 4.0, origin_color);
        }

        draw_fps();
        next_frame().await;
    }
}
