extern crate btleplug;
extern crate rand;
extern crate captrs;
extern crate shuteye;

use captrs::*;
use shuteye::sleep;
use std::time::Duration;

use btleplug::api::{bleuuid::uuid_from_u16, Central, Peripheral, WriteType};
#[cfg(target_os = "linux")]
use btleplug::bluez::manager::Manager;
#[cfg(target_os = "macos")]
use btleplug::corebluetooth::manager::Manager;
#[cfg(target_os = "windows")]
use btleplug::winrtble::manager::Manager;
use rand::{thread_rng, Rng};
use std::thread;
use std::time::Duration;
use std::path;
use uuid::Uuid;

const LIGHT_CHARACTERISTIC_UUID: Uuid = uuid_from_u16(0xFFE9);
const PERIPHERAL_NAME = "DesktopLights";
const DISPLAY_INDEX = 0;

fn find_light() {
    let manager = Manager::new().unwrap();

    // get the first bluetooth adapter
    let central = manager
        .adapters()
        .expect("Unable to fetch adapter list.")
        .into_iter()
        .nth(0)
        .expect("Unable to find adapters.");

    // start scanning for devices
    central.start_scan().unwrap();
    // instead of waiting, you can use central.event_receiver() to get a channel
    // to listen for notifications on.
    thread::sleep(Duration::from_secs(2));

    // find the device we're interested in
    let light = central
        .peripherals()
        .into_iter()
        .find(|p| {
            p.properties()
                .local_name
                .iter()
                .any(|name| name.contains(PERIPHERAL_NAME))
        })
        .expect("No lights found");

    // connect to the device
    light.connect().unwrap();

    // discover characteristics
    light.discover_characteristics().unwrap();

    // find the characteristic we want
    let chars = light.characteristics();
    let cmd_char = chars
        .iter()
        .find(|c| c.uuid == LIGHT_CHARACTERISTIC_UUID)
        .expect("Unable to find characterics");

    return light, command_characteristic;
}

pub fn main() {
    const light, command_characteristic = find_light();

    let mut capturer = Capturer::new(DISPLAY_INDEX).unwrap();

    let (w, h) = capturer.geometry();
    let size = w as u64 * h as u64;
    println!("Display size: {:?}x{:?}", w, h);

    //loop {
    // Capture screen
    let ps = capturer.capture_frame().unwrap();

    let (mut tot_r, mut tot_g, mut tot_b) = (0, 0, 0);

    for Bgr8 { r, g, b, .. } in ps.into_iter() {
        tot_r += r as u64;
        tot_g += g as u64;
        tot_b += b as u64;
    }

    // Get dominant colour
    let image = image::open(&path::Path::new("./docs/Fotolia_45549559_320_480.jpg")).unwrap();
    let has_alpha = match image.color() {
        image::ColorType::Rgba8 => true,
        image::ColorType::Bgra8 => true,
        _ => false,
    };
    let colors = dominant_color::get_colors(&image.to_bytes(), has_alpha);
    println!("has_alpha: {}, colors: {:?}", has_alpha, colors);

    // Write to BLE
    let color_cmd = vec![0x56, rng.gen(), rng.gen(), rng.gen(), 0x00, 0xF0, 0xAA];
    light
        .write(&cmd_char, &color_cmd, WriteType::WithoutResponse)
        .unwrap();
    thread::sleep(Duration::from_millis(200));

    // Sleep before next frame
    //sleep(Duration::from_millis(80));
    //}
}
