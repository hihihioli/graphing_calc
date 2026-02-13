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
// ─────────────────────────────────────────────────────────────────────────────

const SCENE_FRAG: &str = r#"#version 100
precision highp float;
varying vec2 uv;

uniform vec2 center;
uniform vec2 half_extent;
uniform float a_phase;
uniform float zoom_phase;
uniform float hue_shift;
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

const A_PHASE_RATE: f64 = 5.0 * std::f64::consts::LN_2;
const ZOOM_PHASE_RATE: f64 = 10.0 * std::f64::consts::LN_2;

#[macroquad::main(window_conf)]
async fn main() {
    let w = screen_width();
    let h = screen_height();
    let width = w as f64;
    let height = h as f64;

    let initial_scale = 20.0f64;
    let scale_f32 = initial_scale as f32;
    let half_extent = vec2(
        (width / initial_scale / 2.0) as f32,
        (height / initial_scale / 2.0) as f32,
    );

    let mut center_x: f64 = 0.0;
    let mut center_y: f64 = 0.0;
    let mut time: f64 = 0.0;
    let mut hue_shift: f32 = 0.0;
    let mut hue_shift_2: f32 = 0.5;

    // ── Render targets ──────────────────────────────────────────────────
    let scene_target = render_target(w as u32, h as u32);
    scene_target.texture.set_filter(FilterMode::Linear);

    let mut taa_ping = render_target(w as u32, h as u32);
    let mut taa_pong = render_target(w as u32, h as u32);
    taa_ping.texture.set_filter(FilterMode::Linear);
    taa_pong.texture.set_filter(FilterMode::Linear);

    let blur_w = (w as u32) / 4;
    let blur_h = (h as u32) / 4;
    let bright_target = render_target(blur_w, blur_h);
    let blur_ping = render_target(blur_w, blur_h);
    let blur_pong = render_target(blur_w, blur_h);
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

    // ── Tuning ──────────────────────────────────────────────────────────
    bright_mat.set_uniform("threshold", 0.1f32);
    combine_taa_mat.set_uniform("bloom_intensity", 0.5f32);
    scene_mat.set_uniform("half_extent", half_extent);

    let dir_h = vec2(1.0 / blur_w as f32, 0.0);
    let dir_v = vec2(0.0, 1.0 / blur_h as f32);
    let bw = blur_w as f32;
    let bh = blur_h as f32;

    let dummy = Texture2D::from_image(&Image::gen_image_color(1, 1, WHITE));
    let mut first_frame = true;

    let half_w = w / 2.0;
    let half_h = h / 2.0;
    let axis_color = Color::new(1.0, 1.0, 1.0, 0.3);
    let origin_color = Color::new(1.0, 1.0, 1.0, 0.5);

    loop {
        let dt = get_frame_time() as f64;
        time += dt;

        hue_shift += 0.05 * dt as f32;
        if hue_shift >= 1.0 { hue_shift -= 1.0; }

        hue_shift_2 += 0.07 * dt as f32;
        if hue_shift_2 >= 1.0 { hue_shift_2 -= 1.0; }

        // ── Input ───────────────────────────────────────────────────────
        if is_mouse_button_down(MouseButton::Left) {
            let md = mouse_delta_position();
            center_x += md.x as f64 / initial_scale * width * 0.5;
            center_y -= md.y as f64 / initial_scale * height * 0.5;
        }

        let a_phase    = (time * A_PHASE_RATE    % std::f64::consts::TAU) as f32;
        let zoom_phase = (time * ZOOM_PHASE_RATE % std::f64::consts::TAU) as f32;

        // ── Pass 1: Scene ───────────────────────────────────────────────
        scene_mat.set_uniform("center", vec2(center_x as f32, center_y as f32));
        scene_mat.set_uniform("a_phase", a_phase);
        scene_mat.set_uniform("zoom_phase", zoom_phase);
        scene_mat.set_uniform("hue_shift", hue_shift);
        scene_mat.set_uniform("hue_shift_2", hue_shift_2);

        set_camera(&cam_for_target(Some(scene_target.clone()), w, h));
        gl_use_material(&scene_mat);
        draw_fullscreen(&dummy, w, h);
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

        // ── Pass 4: Combine bloom + TAA ─────────────────────────────────
        let taa_alpha = if first_frame { 1.0f32 } else { 0.1f32 };
        combine_taa_mat.set_uniform("taa_alpha", taa_alpha);
        combine_taa_mat.set_texture("_bloom_tex", blur_pong.texture.clone());
        combine_taa_mat.set_texture("_history_tex", taa_ping.texture.clone());

        set_camera(&cam_for_target(Some(taa_pong.clone()), w, h));
        gl_use_material(&combine_taa_mat);
        draw_fullscreen(&scene_target.texture, w, h);
        gl_use_default_material();

        // ── Pass 5: Present ─────────────────────────────────────────────
        set_default_camera();
        draw_fullscreen(&taa_pong.texture, w, h);

        std::mem::swap(&mut taa_ping, &mut taa_pong);
        first_frame = false;

        // ── Axes overlay ────────────────────────────────────────────────
        let ox = half_w - center_x as f32 * scale_f32;
        let oy = half_h + center_y as f32 * scale_f32;

        if oy >= 0.0 && oy <= h {
            draw_line(0.0, oy, w, oy, 2.0, axis_color);
        }
        if ox >= 0.0 && ox <= w {
            draw_line(ox, 0.0, ox, h, 2.0, axis_color);
        }
        if ox >= 0.0 && ox <= w && oy >= 0.0 && oy <= h {
            draw_circle(ox, oy, 4.0, origin_color);
        }

        draw_fps();
        next_frame().await;
    }
}
