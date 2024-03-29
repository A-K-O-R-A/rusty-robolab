use ev3dev_lang_rust::motors::{LargeMotor, MotorPort};
use ev3dev_lang_rust::sensors::ColorSensor;
use ev3dev_lang_rust::Ev3Result;

use std::error;
use std::time::Instant;

const WHITE: (u32, u32, u32) = (200, 200, 200);
const BLACK: (u32, u32, u32) = (20, 20, 20);

const KP: f32 = 0.9;
const KI: f32 = 0.0;
const KD: f32 = 0.0;

const SPEED: f32 = 20.0;

fn main() -> Ev3Result<()> {
    // Get large motor on port outA.
    let right_motor = LargeMotor::get(MotorPort::OutA)?;
    let left_motor = LargeMotor::get(MotorPort::OutD)?;

    // Find color sensor. Always returns the first recognized one.
    let color_sensor = ColorSensor::find()?;

    match run(&right_motor, &left_motor, &color_sensor) {
        Err(e) => eprintln!("{e}"),
        Ok(_) => {}
    }

    // This is a fail safe that will all motors in case something goes wrong
    right_motor.stop()?;
    left_motor.stop()?;

    Ok(())
}

fn run(
    right_motor: &LargeMotor,
    left_motor: &LargeMotor,
    color_sensor: &ColorSensor,
) -> Ev3Result<()> {
    // Set command "run-direct".
    right_motor.run_direct()?;
    left_motor.run_direct()?;

    // Run motor.
    right_motor.set_duty_cycle_sp(0)?;
    left_motor.set_duty_cycle_sp(0)?;

    color_sensor.set_mode_rgb_raw()?;

    let now = Instant::now();

    let mut last_error = 0.0;
    let mut i = 0.0;
    let count = 500;

    for _ in 0..count {
        // Switch to rgb mode.
        // let brightness = calculate_brightness(color_sensor.get_rgb()?);
        let brightness = calculate_brightness((1, 2, 3));
        // let error = brightness - 0.5;
        let error = last_error + 0.01;
        //let error = -0.2;

        let p = error;
        // i += error;
        // let d = error / last_error;
        let d = 0.0;

        // println!("brightness {brightness}, error: {error}");
        // println!("PID {p} : {i} : {d}");

        let pid = (p * KP) + (i * KI) + (d * KD);

        let right_speed = (SPEED - (pid * SPEED)).round() as i32;
        let left_speed = (SPEED + (pid * SPEED)).round() as i32;

        // println!("{pid} : {left_speed} : {right_speed}");

        right_motor.set_duty_cycle_sp(right_speed)?;
        left_motor.set_duty_cycle_sp(left_speed)?;

        last_error = error;
    }
    let elapsed = now.elapsed() / count;
    println!("Loop time {:?}", elapsed);

    Ok(())
}

fn calculate_brightness(color: (i32, i32, i32)) -> f32 {
    let (r_w, g_w, b_w) = WHITE;
    let (r_b, g_b, b_b) = BLACK;

    let (r, g, b) = color;
    let r = ((r as f32 - r_b as f32) / (r_w as f32 - r_b as f32)).clamp(0.0, 1.0);
    let g = ((g as f32 - g_b as f32) / (g_w as f32 - g_b as f32)).clamp(0.0, 1.0);
    let b = ((b as f32 - b_b as f32) / (b_w as f32 - b_b as f32)).clamp(0.0, 1.0);

    (0.299 * (r as f32)) + (0.587 * (g as f32)) + (0.114 * (b as f32))
}
