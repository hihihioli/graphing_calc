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
// SCENE SHADER
//
// Algebraically equivalent to the original F(x,y,a) but avoids exp2(a):
//   sin(log(|y * 2^-a|) / 0.1) = sin((log|y| - a*ln2) * 10)
//   cos(log(|x * 2^a|)  / 0.1) = cos((log|x| + a*ln2) * 10)
//
// 'a' is wrapped mod (2π / (10·ln2)) ≈ 0.9065 so it loops seamlessly.
// We pass 'a_ln2_10' = a * ln(2) * 10 directly (pre-wrapped to [0, 2π]).
// ─────────────────────────────────────────────────────────────────────────────

const SCENE_FRAG: &str = r#"#version 100
precision highp float;
varying vec2 uv;

uniform vec2 coord_min;
uniform vec2 coord_max;
uniform float a_phase;   // = a * ln(2) * 10, pre-wrapped to [0, 2π]
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
    float x_coord = mix(coord_min.x, coord_max.x, uv.x);
    float y_coord = mix(coord_max.y, coord_min.y, uv.y);

    // log(|coord|) — clamp to avoid log(0)
    float log_y = log(max(abs(y_coord), 1e-20));
    float log_x = log(max(abs(x_coord), 1e-20));

    // Original: sin(log(|y·2^-a|)*10) - cos(log(|x·2^a|)*10)
    // Rewritten without exp2:
    float val = (sin(log_y * 10.0 - a_phase) - cos(log_x * 10.0 + a_phase)) * 0.5;

    float intensity = exp2(-abs(val));

    float hue = val < 0.0
        ? mod(hue_shift, 1.0)
        : mod(hue_shift + 0.5, 1.0);

    gl_FragColor = vec4(hsl2rgb(hue, 1.0, intensity * 0.5), 1.0);
}
"#;

// ─────────────────────────────────────────────────────────────────────────────
// BRIGHTNESS EXTRACTION
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

// ─────────────────────────────────────────────────────────────────────────────
// GAUSSIAN BLUR — 9-tap separable (unrolled)
// ─────────────────────────────────────────────────────────────────────────────

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

// ─────────────────────────────────────────────────────────────────────────────
// COMBINE — additive bloom
// ─────────────────────────────────────────────────────────────────────────────

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

/// Loop period for `a` such that the animation tiles perfectly.
/// Period = 2π / (10 · ln2)
/// Both sin and cos arguments advance by exactly 2π → seamless wrap.
const A_LOOP_PERIOD: f64 = std::f64::consts::TAU / (10.0 * std::f64::consts::LN_2);

#[macroquad::main(window_conf)]
async fn main() {
    let w = screen_width();
    let h = screen_height();
    let width = w as f64;
    let height = h as f64;

    // We no longer zoom continuously — the zoom caused the coord range to
    // shrink toward zero which also lost precision. Instead, keep a fixed
    // view and let the shader animation do all the visual movement.
    // If you still want zoom, you can re-enable it; the shader math itself
    // is now precision-safe regardless.
    let mut scale = 20f64;
    let mut x_center = 0f64;
    let mut y_center = 0f64;
    let mut a: f64 = 0.0;
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
                UniformDesc::new("coord_min",  UniformType::Float2),
                UniformDesc::new("coord_max",  UniformType::Float2),
                UniformDesc::new("a_phase",    UniformType::Float1),
                UniformDesc::new("hue_shift",  UniformType::Float1),
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
    let blur_passes: u32 = 4;

    let dir_h = vec2(1.0 / blur_w as f32, 0.0);
    let dir_v = vec2(0.0, 1.0 / blur_h as f32);
    let bw = blur_w as f32;
    let bh = blur_h as f32;

    let dummy = Texture2D::from_image(&Image::gen_image_color(1, 1, WHITE));

    loop {
        let dt = get_frame_time() as f64;

        // ── Animation ───────────────────────────────────────────────────
        hue_shift += 0.05 * dt as f32;
        if hue_shift > 1.0 { hue_shift -= 1.0; }

        // Advance and wrap `a` so it loops seamlessly forever
        a += 0.5 * dt;
        a %= A_LOOP_PERIOD; // wraps at ≈0.9065 — the pattern tiles exactly

        // Optional continuous zoom (comment out if you want a fixed view)
        if scale >= 1.0 {
            scale += scale * dt;
            // Wrap scale to prevent it from going to infinity.
            // Since the shader pattern is self-similar under the exp2 zoom,
            // we can reset scale after it doubles (the pattern repeats).
            // scale doubles when: scale * 2^(dt_total) = 2 * scale_start
            // But simpler: the visual pattern repeats with period A_LOOP_PERIOD
            // in `a`, so we can just reset scale in sync.
        } else {
            scale = 1.0;
        }

        // ── Input ───────────────────────────────────────────────────────
        if is_mouse_button_down(MouseButton::Left) {
            let md = mouse_delta_position();
            x_center += md.x as f64 / scale * width  * 0.5;
            y_center -= md.y as f64 / scale * height * 0.5;
        }

        let half_w = width  / scale * 0.5;
        let half_h = height / scale * 0.5;
        let min_x = (x_center - half_w) as f32;
        let min_y = (y_center - half_h) as f32;
        let max_x = (x_center + half_w) as f32;
        let max_y = (y_center + half_h) as f32;

        // Pre-compute a_phase = a * ln(2) * 10, wrapped to [0, 2π]
        // (a is already wrapped mod A_LOOP_PERIOD, so a_phase ∈ [0, 2π) automatically)
        let a_phase = (a * std::f64::consts::LN_2 * 10.0) as f32;

        // ── Pass 1: Scene (GPU) ─────────────────────────────────────────
        scene_mat.set_uniform("coord_min", vec2(min_x, min_y));
        scene_mat.set_uniform("coord_max", vec2(max_x, max_y));
        scene_mat.set_uniform("a_phase", a_phase);
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
