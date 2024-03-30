use ev3dev_lang_rust::motors::{LargeMotor, MotorPort};
use ev3dev_lang_rust::sensors::ColorSensor;
use ev3dev_lang_rust::{Device, Ev3Result};

use std::process;
use std::time::Instant;

#[allow(unused_imports)]
use crate::color::calibrate_colors;

mod color;

const KP: f32 = 0.5;
const KI: f32 = 0.0001;
const KD: f32 = 0.0;

const SPEED: f32 = 100.0;

fn main() -> Ev3Result<()> {
    // Get large motor on port outA.
    let right_motor = LargeMotor::get(MotorPort::OutA)?;
    let left_motor = LargeMotor::get(MotorPort::OutD)?;

    right_motor.set_stop_action(LargeMotor::STOP_ACTION_COAST)?;
    left_motor.set_stop_action(LargeMotor::STOP_ACTION_COAST)?;

    // Find color sensor. Always returns the first recognized one.
    let color_sensor = ColorSensor::find()?;


    let r_motor = right_motor.clone();
    let l_motor = left_motor.clone();
    ctrlc::set_handler( move || {
        // This is a fail safe that will all motors in case something goes wrong
        r_motor.stop().unwrap();
        l_motor.stop().unwrap();

        process::exit(-1)
    })
    .expect("Error setting Ctrl-C handler");

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
    color_sensor.set_mode_rgb_raw()?;

    // calibrate_colors(color_sensor)?;

    // Set command "run-direct".
    right_motor.run_direct()?;
    left_motor.run_direct()?;

    let mut last_error = 0.0;
    let mut i = 0.0;
    let iter_count = 1000;

    let right_attr = right_motor.get_attribute("duty_cycle_sp");
    let left_attr = left_motor.get_attribute("duty_cycle_sp");

    // let color_format: String = color_sensor.get_attribute("num_values").get()?;
    // println!("{color_format}"); // s16


    let now = Instant::now();

    // (s16) 2 * 4 = 8 bytes

    // right_motor.stop()?;
    // left_motor.stop()?;

    let mut count = 0;
    for _ in 0..iter_count {
        // let color = color_sensor.get_rgb()?;

        /*
        let color = (
            color_attr_r.get()?,
            color_attr_g.get()?,
            color_attr_b.get()?,
        );
        */
        count += 1;


        let color = color_sensor.get_bin_data()?;
        let color = (color.0 as i32, color.1 as i32, color.2 as i32);

        if color::check_blue(color) {
            println!("Found blue");
            break;
        } else if color::check_red(color) {
            println!("Found red");
            break;
        }

        let adjusted_color = color::adjust_color(color);
        let brightness = color::calculate_brightness(adjusted_color);
        // Error should range from -1.0 to 1.0
        let error = 2.0 * (brightness - 0.5);

        let p = error;
        i += error;
        let d: f32 = error - last_error;

        // println!("brightness {brightness}, error: {error}");
        // println!("PID {p} : {i} : {d}");

        let pid = (p * KP) + (i * KI) + (d * KD);

        let right_speed = (SPEED - (pid * SPEED)).round() as i32;
        let right_speed = right_speed.clamp(-100, 100);
        let left_speed = (SPEED + (pid * SPEED)).round() as i32;
        let left_speed = left_speed.clamp(-100, 100);

        // println!("{pid} : {left_speed} : {right_speed}");

        // right_motor.set_duty_cycle_sp(right_speed)?;
        // left_motor.set_duty_cycle_sp(left_speed)?;

        right_attr.set(right_speed)?;
        left_attr.set(left_speed)?;

        last_error = error;
    }
    let elapsed = now.elapsed() / count;
    println!("Loop time {:?}", elapsed);

    right_motor.stop()?;
    left_motor.stop()?;

    Ok(())
}
