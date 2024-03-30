use ev3dev_lang_rust::motors::{LargeMotor, MotorPort};
use ev3dev_lang_rust::sensors::ColorSensor;
use ev3dev_lang_rust::{Device, Ev3Result};

use std::fs::{self, OpenOptions};
use std::io::{Read, Seek, SeekFrom};
use std::os::unix::fs::PermissionsExt;
use std::process;
use std::time::Instant;

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

    ctrlc::set_handler(|| {
        let right_motor = LargeMotor::get(MotorPort::OutA).unwrap();
        let left_motor = LargeMotor::get(MotorPort::OutD).unwrap();

        // This is a fail safe that will all motors in case something goes wrong
        right_motor.stop().unwrap();
        left_motor.stop().unwrap();

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

    // let color_attr_r = color_sensor.get_attribute("value0");
    // let color_attr_g = color_sensor.get_attribute("value1");
    // let color_attr_b = color_sensor.get_attribute("value2");

    // let color_format: String = color_sensor.get_attribute("num_values").get()?;
    // println!("{color_format}"); // s16

    let color_attr = color_sensor.get_attribute("bin_data");
    // "/sys/class/lego-sensor/sensor0/bin_data"
    let path = color_attr.get_file_path().clone();
    println!("{path:?}");
    drop(color_attr);

    let stat = fs::metadata(&path)?;

    let mode = stat.permissions().mode();

    // Read permission for group (`ev3dev`)
    let readable = mode & 0o040 == 0o040;
    let writeable = mode & 0o020 == 0o020;

    let mut file = OpenOptions::new()
        .read(readable)
        .write(writeable)
        .open(path)?;

    let now = Instant::now();

    // (s16) 2 * 4 = 8 bytes
    let mut color_vec: Vec<u8> = Vec::with_capacity(8);

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

        file.seek(SeekFrom::Start(0))?;
        color_vec.clear();
        let _count_read = file.read_to_end(&mut color_vec)?;

        let title: Vec<i16> = color_vec[0..8]
            .chunks_exact(2)
            .map(|a| i16::from_ne_bytes([a[0], a[1]]))
            .collect();
        // println!("{title:?}");
        let color = (title[0] as i32, title[1] as i32, title[2] as i32);

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
