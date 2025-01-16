use hidapi::{HidApi, HidDevice};
use std::io::{self, Write};

// Default VID list for TinyUSB, Adafruit, RaspberryPi, and Espressif
const USB_VID: [u16; 4] = [0xcafe, 0x239a, 0x2e8a, 0x303a];

fn main() {
    // Initialize the HID API
    let api = match HidApi::new() {
        Ok(api) => api,
        Err(e) => {
            eprintln!("Failed to initialize HID API: {}", e);
            return;
        }
    };

    println!("VID list: {:02x?}", USB_VID);

    for &vid in &USB_VID {
        for device_info in api.device_list().filter(|d| d.vendor_id() == vid) {
            println!("Found device: {:?}", device_info);

            if let usage = device_info.usage() {
                if usage == 1 {
                    match api.open(device_info.vendor_id(), device_info.product_id()) {
                        Ok(mut device) => {
                            interact_with_device(&mut device);
                        }
                        Err(e) => {
                            eprintln!("Failed to open device: {}", e);
                        }
                    }
                }
            }
        }
    }
}

fn interact_with_device(device: &mut HidDevice) {
    println!("Connected to device. Start sending text.");

    loop {
        // Get input from the console
        print!("Send text to HID Device: ");
        io::stdout().flush().unwrap();

        let mut input = String::new();
        if let Err(e) = io::stdin().read_line(&mut input) {
            eprintln!("Failed to read input: {}", e);
            continue;
        }

        // Encode the input as a UTF-8 byte array, preceded by a dummy report ID (0x00)
        let mut output = vec![0x00];
        output.extend_from_slice(input.trim().as_bytes());

        // Send the data to the HID device
        if let Err(e) = device.write(&output) {
            eprintln!("Failed to write to device: {}", e);
            continue;
        }

        // Read the response from the device
        let mut buf = [0u8; 64];
        match device.read(&mut buf) {
            Ok(len) => {
                println!("Received from HID Device: {:?}\n", &buf[..len]);
            }
            Err(e) => {
                eprintln!("Failed to read from device: {}", e);
            }
        }
    }
}