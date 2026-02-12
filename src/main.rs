use macroquad::prelude::*;
use std::default::Default;

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
// SCENE SHADER — fully precision-safe, loops forever
//
// Key insight: the original pattern uses log(|coord * 2^a|), which blows up.
// But log(|x * k|) = log|x| + log(k), so scaling is just a phase shift in
// log-space. We pass TWO wrapping phases:
//
//   zoom_phase: shifts both log terms equally (looks like zooming in/out)
//   a_phase:    shifts y and x in opposite directions (the original animation)
//
// y term: sin(log|y| * 10 - zoom_phase - a_phase)
// x term: cos(log|x| * 10 - zoom_phase + a_phase)
//
// Coordinates stay in a FIXED window — no precision loss, ever.
// Both phases wrap mod 2π — loops seamlessly forever.
// ─────────────────────────────────────────────────────────────────────────────

const SCENE_FRAG: &str = r#"#version 100
precision highp float;
varying vec2 uv;

uniform vec2 center;       // pan offset (world space)
uniform vec2 half_extent;  // half-size of coordinate window (constant)
uniform float a_phase;     // animation phase, wraps [0, 2π)
uniform float zoom_phase;  // zoom phase, wraps [0, 2π)
uniform float hue_shift;

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
    // Map UV → world coordinates (FIXED window, never shrinks)
    float x_coord = center.x + (uv.x - 0.5) * half_extent.x * 2.0;
    float y_coord = center.y + (0.5 - uv.y) * half_extent.y * 2.0;

    // log|coord| — the core of the pattern, stays in a stable range
    float log_y = log(max(abs(y_coord), 1e-20)) * 10.0;
    float log_x = log(max(abs(x_coord), 1e-20)) * 10.0;

    // Apply both phase shifts — this IS the zoom + animation, no exp2 needed
    float val = (sin(log_y - zoom_phase - a_phase)
               - cos(log_x - zoom_phase + a_phase)) * 0.5;

    float intensity = exp2(-abs(val));

    float hue = val < 0.0
        ? mod(hue_shift, 1.0)
        : mod(hue_shift + 0.5, 1.0);

    gl_FragColor = vec4(hsl2rgb(hue, 1.0, intensity * 0.5), 1.0);
}
"#;

// ─────────────────────────────────────────────────────────────────────────────
// BLOOM SHADERS (unchanged)
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

const COMBINE_FRAG: &str = r#"#version 100
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

// ─────────────────────────────────────────────────────────────────────────────
// HELPERS
// ─────────────────────────────────────────────────────────────────────────────

fn window_conf() -> Conf {
    Conf {
        window_title: "Main Window".to_string(),
        high_dpi: false,
        fullscreen: true,
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

// Phase rates (rad/s) — chosen so the overall loop tiles perfectly.
//
// a_phase   advances at A_PHASE_RATE  = 5·ln2 rad/s
// zoom_phase advances at ZOOM_PHASE_RATE = 10·ln2 rad/s  (2× a_phase)
//
// Loop period = 2π / (5·ln2) ≈ 1.81 seconds.
// At that point:  a_phase has done 1 full 2π cycle,
//                 zoom_phase has done 2 full 2π cycles.
// → both sin and cos return to their starting values → seamless.

const A_PHASE_RATE: f64 = 5.0 * std::f64::consts::LN_2;     // ≈ 3.466 rad/s
const ZOOM_PHASE_RATE: f64 = 10.0 * std::f64::consts::LN_2;  // ≈ 6.931 rad/s
const LOOP_PERIOD: f64 = std::f64::consts::TAU / A_PHASE_RATE; // ≈ 1.813 s

#[macroquad::main(window_conf)]
async fn main() {
    let w = screen_width();
    let h = screen_height();
    let width = w as f64;
    let height = h as f64;

    // Fixed coordinate window — NEVER changes from zoom.
    // This is the entire source of the precision fix.
    let initial_scale = 20.0f64;
    let half_extent_x = (width  / initial_scale / 2.0) as f32;
    let half_extent_y = (height / initial_scale / 2.0) as f32;

    // Panning (can still be changed by mouse)
    let mut center_x: f64 = 0.0;
    let mut center_y: f64 = 0.0;

    // Single time accumulator — everything derives from this
    let mut time: f64 = 0.0;
    let mut hue_shift: f32 = 0.0;

    // ── Render targets ──────────────────────────────────────────────────
    let scene_target = render_target(w as u32, h as u32);
    scene_target.texture.set_filter(FilterMode::Linear);

    let blur_w = (w as u32) / 4;
    let blur_h = (h as u32) / 4;
    let bright_target = render_target(blur_w, blur_h);
    let blur_ping     = render_target(blur_w, blur_h);
    let blur_pong     = render_target(blur_w, blur_h);
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

    let combine_mat = load_material(
        ShaderSource::Glsl { vertex: VERTEX, fragment: COMBINE_FRAG },
        MaterialParams {
            uniforms: vec![UniformDesc::new("bloom_intensity", UniformType::Float1)],
            textures: vec!["_bloom_tex".to_string()],
            ..Default::default()
        },
    ).unwrap();

    // ── Tuning ──────────────────────────────────────────────────────────
    bright_mat.set_uniform("threshold", 0.1f32);
    combine_mat.set_uniform("bloom_intensity", 0.5f32);
    scene_mat.set_uniform("half_extent", vec2(half_extent_x, half_extent_y));
    let blur_passes: u32 = 4;

    let dir_h = vec2(1.0 / blur_w as f32, 0.0);
    let dir_v = vec2(0.0, 1.0 / blur_h as f32);
    let bw = blur_w as f32;
    let bh = blur_h as f32;

    let dummy = Texture2D::from_image(&Image::gen_image_color(1, 1, WHITE));

    loop {
        let dt = get_frame_time() as f64;

        // ── Advance time and wrap at loop period ────────────────────────
        time += dt;
        time %= LOOP_PERIOD; // wraps every ≈1.81s — prevents drift forever

        hue_shift += 0.05 * dt as f32;
        if hue_shift >= 1.0 { hue_shift -= 1.0; }

        // ── Input (panning) ─────────────────────────────────────────────
        if is_mouse_button_down(MouseButton::Left) {
            let md = mouse_delta_position();
            center_x += md.x as f64 / initial_scale * width  * 0.5;
            center_y -= md.y as f64 / initial_scale * height * 0.5;
        }

        // ── Compute phases from time (all f64, wrap to [0,2π), then f32) ─
        let a_phase    = (time * A_PHASE_RATE    % std::f64::consts::TAU) as f32;
        let zoom_phase = (time * ZOOM_PHASE_RATE % std::f64::consts::TAU) as f32;

        // ── Pass 1: Scene (GPU) ─────────────────────────────────────────
        scene_mat.set_uniform("center", vec2(center_x as f32, center_y as f32));
        scene_mat.set_uniform("a_phase", a_phase);
        scene_mat.set_uniform("zoom_phase", zoom_phase);
        scene_mat.set_uniform("hue_shift", hue_shift);

        set_camera(&cam_for_target(Some(scene_target.clone()), w, h));
        clear_background(BLACK);
        gl_use_material(&scene_mat);
        draw_fullscreen(&dummy, w, h);
        gl_use_default_material();

        // ── Pass 2: Brightness extraction ───────────────────────────────
        set_camera(&cam_for_target(Some(bright_target.clone()), bw, bh));
        clear_background(BLACK);
        gl_use_material(&bright_mat);
        draw_fullscreen(&scene_target.texture, bw, bh);
        gl_use_default_material();

        // ── Pass 3–4: Ping-pong blur ────────────────────────────────────
        let mut src = &bright_target.texture;
        for _ in 0..blur_passes {
            set_camera(&cam_for_target(Some(blur_ping.clone()), bw, bh));
            clear_background(BLACK);
            blur_mat.set_uniform("direction", dir_h);
            gl_use_material(&blur_mat);
            draw_fullscreen(src, bw, bh);
            gl_use_default_material();

            set_camera(&cam_for_target(Some(blur_pong.clone()), bw, bh));
            clear_background(BLACK);
            blur_mat.set_uniform("direction", dir_v);
            gl_use_material(&blur_mat);
            draw_fullscreen(&blur_ping.texture, bw, bh);
            gl_use_default_material();

            src = &blur_pong.texture;
        }

        // ── Pass 5: Combine → screen ────────────────────────────────────
        set_default_camera();
        clear_background(BLACK);
        combine_mat.set_texture("_bloom_tex", blur_pong.texture.clone());
        gl_use_material(&combine_mat);
        draw_fullscreen(&scene_target.texture, w, h);
        gl_use_default_material();

        draw_fps();
        next_frame().await;
    }
}
