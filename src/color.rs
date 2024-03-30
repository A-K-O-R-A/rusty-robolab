use std::{thread, time::Duration};

use ev3dev_lang_rust::{sensors::ColorSensor, Ev3Result};

/*
const WHITE: (u32, u32, u32) = (328, 353, 225);
const BLACK: (u32, u32, u32) = (36, 47, 47);

const BLUE: (u32, u32, u32) = (161, 291, 196);
const RED: (u32, u32, u32) = (305, 94, 57);
*/

const WHITE: (u32, u32, u32) = (265, 285, 197);
const BLACK: (u32, u32, u32) = (23, 32, 19);

const BLUE: (u32, u32, u32) = (30, 86, 93);
const RED: (u32, u32, u32) = (124, 27, 17);

type RawColor = (i32, i32, i32);
type RgbColor = (f32, f32, f32);

pub fn check_blue(color: RawColor) -> bool {
    let threshold = 30;

    let (r, g, b) = color;
    let (ref_r, ref_g, ref_b) = BLUE;

    r.abs_diff(ref_r as i32) < threshold
        && g.abs_diff(ref_g as i32) < threshold
        && b.abs_diff(ref_b as i32) < threshold
}

pub fn check_red(color: RawColor) -> bool {
    let threshold = 20;

    let (r, g, b) = color;
    let (ref_r, ref_g, ref_b) = RED;

    r.abs_diff(ref_r as i32) < threshold
        && g.abs_diff(ref_g as i32) < threshold
        && b.abs_diff(ref_b as i32) < threshold
}

pub fn calculate_brightness(color: RgbColor) -> f32 {
    let (r, g, b) = color;

    (0.299 * r) + (0.587 * g) + (0.114 * b)
}

pub fn adjust_color(color: RawColor) -> RgbColor {
    let (r_w, g_w, b_w) = WHITE;
    let (r_b, g_b, b_b) = BLACK;

    let (r, g, b) = color;
    let r = ((r as f32 - r_b as f32) / (r_w as f32 - r_b as f32)).clamp(0.0, 1.0);
    let g = ((g as f32 - g_b as f32) / (g_w as f32 - g_b as f32)).clamp(0.0, 1.0);
    let b = ((b as f32 - b_b as f32) / (b_w as f32 - b_b as f32)).clamp(0.0, 1.0);

    (r, g, b)
}

#[allow(dead_code)]
pub fn calibrate_colors(color_sensor: &ColorSensor) -> Ev3Result<()> {
    color_sensor.set_mode_rgb_raw()?;

    let color_list = ["white", "black", "blue", "red"];

    for color in color_list {
        println!("Calibrating {color}");
        thread::sleep(Duration::from_secs(5));
        let color = color_sensor.get_rgb()?;
        println!("{color:?}");
    }

    Ok(())
}
