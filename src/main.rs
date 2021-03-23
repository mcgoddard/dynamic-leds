extern crate btleplug;
extern crate scrap;

use std::io::ErrorKind::WouldBlock;
use std::thread;
use std::time::Duration;
use std::panic;

use scrap::{Capturer, Display};
use btleplug::api::{Central, Peripheral, WriteType, Characteristic};
#[cfg(target_os = "linux")]
use btleplug::bluez::manager::Manager;
#[cfg(target_os = "macos")]
use btleplug::corebluetooth::manager::Manager;
#[cfg(target_os = "windows")]
use btleplug::winrtble::manager::Manager;
use uuid::Uuid;

const SLEEP_TO_FIND_DEVICE_S: u64 = 1;
const SLEEP_BETWEEN_UPDATES_MS: u64 = 30;
const SLEEP_WAIT_FOR_FRAMES_MS: u64 = 10;
const COLOUR_SEND_THRESHOLD: u8 = 2;

fn find_light() -> (btleplug::winrtble::peripheral::Peripheral, Characteristic) {
    let light_characteristic_uuid: Uuid = Uuid::parse_str("00010203-0405-0607-0809-0A0B0C0D2B11").unwrap();
    let peripheral_name: String = "ihoment_H6181_9F87".to_string();
    let manager = Manager::new().unwrap();

    // get the first bluetooth adapter
    let central = manager
        .adapters()
        .expect("Unable to fetch adapter list.")
        .into_iter()
        .next()
        .expect("Unable to find adapters.");

    // start scanning for devices
    central.start_scan().unwrap();
    // instead of waiting, you can use central.event_receiver() to get a channel
    // to listen for notifications on.
    thread::sleep(Duration::from_secs(SLEEP_TO_FIND_DEVICE_S));

    // find the device we're interested in
    let light = central
        .peripherals()
        .into_iter()
        .find(|p| {
            p.properties()
                .local_name
                .iter()
                .any(|name| name.contains(&peripheral_name))
        })
        .expect("No lights found");

    // connect to the device
    light.connect().unwrap();

    // discover characteristics
    let chars = light.discover_characteristics().unwrap();

    // find the characteristic we want
    let command_characteristic = chars
        .iter()
        .find(|c| c.uuid == light_characteristic_uuid)
        .expect("Unable to find characterics");

    (light, command_characteristic.clone())
}

fn send_color(light: &btleplug::winrtble::peripheral::Peripheral, command_characteristic: &Characteristic, red: u8, green: u8, blue: u8) {
    let mut color_cmd = vec![
        // command delimiter
        0x33,
        // change colour
        0x05,
        // manual mode
        0x02,
        // red
        red,
        // green
        green,
        //blue
        blue,
        // padding
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        // crc (will be calculated and overwritten)
        0x00
    ];
    let check = color_cmd.iter().fold(0_u8, |acc, x| acc ^ x);
    color_cmd[19] = check;
    light.write(&command_characteristic, &color_cmd, WriteType::WithoutResponse).unwrap();
}

fn colour_difference(colour_1: u8, colour_2: u8) -> bool {
    (colour_1 >= colour_2 && colour_1 > colour_2 + COLOUR_SEND_THRESHOLD) ||
        (colour_1 < colour_2 && colour_1 < colour_2 - COLOUR_SEND_THRESHOLD)
}

fn update_lights() {
    let (light, command_characteristic) = find_light();
    println!("Connected");

    let display = Display::primary().expect("Couldn't find primary display.");
    let mut capturer = Capturer::new(display).expect("Couldn't begin capture.");
    let (w, h) = (capturer.width(), capturer.height());
    println!("Display size: {:?}x{:?}", w, h);

    let (mut last_red, mut last_green, mut last_blue) = (0_u8, 0_u8, 0_u8);

    loop {
        // Capture screen
        let buffer = match capturer.frame() {
            Ok(buffer) => buffer,
            Err(error) => {
                if error.kind() == WouldBlock {
                    // No frame ready, keep spinning
                    thread::sleep(Duration::from_millis(SLEEP_WAIT_FOR_FRAMES_MS));
                    continue;
                } else {
                    panic!("Error: {}", error);
                }
            }
        };

        // Get dominant colour
        let colours = dominant_color::get_colors(&buffer, false);

        // Write to BLE
        if colour_difference(last_red, colours[2]) ||
            colour_difference(last_green, colours[1]) ||
            colour_difference(last_blue, colours[0]) {
            send_color(&light, &command_characteristic, colours[2], colours[1], colours[0]);
            last_red = colours[2];
            last_green = colours[1];
            last_blue = colours[0];
        }

        // Sleep before next frame
        thread::sleep(Duration::from_millis(SLEEP_BETWEEN_UPDATES_MS));
    }
}

pub fn main() {
    loop {
        match panic::catch_unwind(|| {
            update_lights();
        }) {
            Ok(_) => {
                println!("Exiting");
                break;
            },
            Err(_) => {
                println!("Caught panic, restarting");
            }
        };
    }
}
