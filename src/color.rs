const WHITE: (u32, u32, u32) = (200, 200, 200);
const BLACK: (u32, u32, u32) = (20, 20, 20);

pub fn calculate_brightness(color: (i32, i32, i32)) -> f32 {
    let (r_w, g_w, b_w) = WHITE;
    let (r_b, g_b, b_b) = BLACK;

    let (r, g, b) = color;
    let r = ((r as f32 - r_b as f32) / (r_w as f32 - r_b as f32)).clamp(0.0, 1.0);
    let g = ((g as f32 - g_b as f32) / (g_w as f32 - g_b as f32)).clamp(0.0, 1.0);
    let b = ((b as f32 - b_b as f32) / (b_w as f32 - b_b as f32)).clamp(0.0, 1.0);

    (0.299 * (r as f32)) + (0.587 * (g as f32)) + (0.114 * (b as f32))
}
