use ev3dev_lang_rust::motors::{LargeMotor, MotorPort};
use ev3dev_lang_rust::sensors::ColorSensor;
use ev3dev_lang_rust::Ev3Result;

use std::time::Instant;

mod color;

const KP: f32 = 0.9;
const KI: f32 = 0.0;
const KD: f32 = 0.0;

const SPEED: f32 = 10.0;

fn main() -> Ev3Result<()> {
    // Get large motor on port outA.
    let right_motor = LargeMotor::get(MotorPort::OutA)?;
    let left_motor = LargeMotor::get(MotorPort::OutD)?;

    right_motor.set_stop_action(LargeMotor::STOP_ACTION_BRAKE)?;
    left_motor.set_stop_action(LargeMotor::STOP_ACTION_BRAKE)?;

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

    color_sensor.set_mode_rgb_raw()?;

    let mut last_error = 0.0;
    let mut i = 0.0;
    let count = 500;

    // let right_attr = right_motor.get_attribute("duty_cycle_sp");
    // let left_attr = right_motor.get_attribute("duty_cycle_sp");

    /*
    let color_attr_r = color_sensor.get_attribute("value0");
    let color_attr_g = color_sensor.get_attribute("value1");
    let color_attr_b = color_sensor.get_attribute("value2");
    */

    let now = Instant::now();

    for _ in 0..count {
        let color = color_sensor.get_rgb()?;
        /*
        let color = (
            color_attr_r.get()?,
            color_attr_g.get()?,
            color_attr_b.get()?,
        );
         */

        let brightness = color::calculate_brightness(color);
        // Error should range from -1.0 to 1.0
        let error = 2.0 * (brightness - 0.5);

        let p = error;
        i += error;
        let d: f32 = error - last_error;

        // println!("brightness {brightness}, error: {error}");
        println!("PID {p} : {i} : {d}");

        let pid = (p * KP) + (i * KI) + (d * KD);

        let right_speed = (SPEED - (pid * SPEED)).round() as i32;
        let right_speed = right_speed.clamp(-100, 100);
        let left_speed = (SPEED + (pid * SPEED)).round() as i32;
        let left_speed = left_speed.clamp(-100, 100);

        // println!("{pid} : {left_speed} : {right_speed}");

        right_motor.set_duty_cycle_sp(right_speed)?;
        left_motor.set_duty_cycle_sp(left_speed)?;

        // right_attr.set(right_speed)?;
        // left_attr.set(left_speed)?;

        last_error = error;
    }
    let elapsed = now.elapsed() / count;
    println!("Loop time {:?}", elapsed);

    Ok(())
}
