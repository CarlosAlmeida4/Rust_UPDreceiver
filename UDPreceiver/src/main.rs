use hidapi::{HidApi, HidDevice};
use core::str;
use std::io::{self, Write};
use tokio::time::{sleep,Duration};
use tokio::{net::UdpSocket as AsyncUdpSocket, task, sync::mpsc};
//use serde::{Serialize,Deserialize};

// Default VID list for TinyUSB, Adafruit, RaspberryPi, and Espressif
const USB_VID: [u16; 4] = [0xcafe, 0x239a, 0x2e8a, 0x303a];

#[warn(dead_code)]
#[derive(Debug)]
struct TelemetryData {
    packet_4cc: u32,
    packet_uid: u64,
    shiftlights_fraction: f32,
    shiftlights_rpm_start: f32,
    shiftlights_rpm_end: f32,
    shiftlights_rpm_valid: bool,
    vehicle_gear_index: u8,
    vehicle_gear_index_neutral: u8,
    vehicle_gear_index_reverse: u8,
    vehicle_gear_maximum: u8,
    vehicle_speed: f32,
    vehicle_transmission_speed: f32,
    vehicle_position_x: f32,
    vehicle_position_y: f32,
    vehicle_position_z: f32,
    vehicle_velocity_x: f32,
    vehicle_velocity_y: f32,
    vehicle_velocity_z: f32,
    vehicle_acceleration_x: f32,
    vehicle_acceleration_y: f32,
    vehicle_acceleration_z: f32,
    vehicle_left_direction_x: f32,
    vehicle_left_direction_y: f32,
    vehicle_left_direction_z: f32,
    vehicle_forward_direction_x: f32,
    vehicle_forward_direction_y: f32,
    vehicle_forward_direction_z: f32,
    vehicle_up_direction_x: f32,
    vehicle_up_direction_y: f32,
    vehicle_up_direction_z: f32,
    vehicle_hub_position_bl: f32,
    vehicle_hub_position_br: f32,
    vehicle_hub_position_fl: f32,
    vehicle_hub_position_fr: f32,
    vehicle_hub_velocity_bl: f32,
    vehicle_hub_velocity_br: f32,
    vehicle_hub_velocity_fl: f32,
    vehicle_hub_velocity_fr: f32,
    vehicle_cp_forward_speed_bl: f32,
    vehicle_cp_forward_speed_br: f32,
    vehicle_cp_forward_speed_fl: f32,
    vehicle_cp_forward_speed_fr: f32,
    vehicle_brake_temperature_bl: f32,
    vehicle_brake_temperature_br: f32,
    vehicle_brake_temperature_fl: f32,
    vehicle_brake_temperature_fr: f32,
    vehicle_engine_rpm_max: f32,
    vehicle_engine_rpm_idle: f32,
    vehicle_engine_rpm_current: f32,
    vehicle_throttle: f32,
    vehicle_brake: f32,
    vehicle_clutch: f32,
    vehicle_steering: f32,
    vehicle_handbrake: bool,
    game_total_time: f32,
    game_delta_time: f32,
    game_frame_count: u32,
    stage_current_time: f32,
    stage_current_distance: f32,
    stage_length: f32,
}

#[tokio::main]
async fn main() -> io::Result<()> {
    // Create a channel to send UDP packets to the HID task
    let (tx, rx) = mpsc::channel::<TelemetryData>(10);

    let udp_task = task::spawn(start_udp_listener(tx));
    let hid_task = task::spawn(start_hid_interaction(rx));

    // Wait for both tasks to complete
    let _ = tokio::join!(udp_task, hid_task);

    Ok(())
}

async fn start_udp_listener(tx: mpsc::Sender<TelemetryData>) -> io::Result<()> {
    //let mut addr = String::new();
    //println!("Please input the IP and the port:");
    //io::stdin().read_line(&mut addr)?;
    let mut addr = String::from("127.0.0.1:20782");

    addr = addr.trim().to_string(); // Remove newline characters
    let socket = AsyncUdpSocket::bind(&addr).await?;
    println!("Listening on socket {}", addr);

    let mut buf = [0u8; 1024];

    loop {
        match socket.recv_from(&mut buf).await {
            Ok((size, _src)) => {
                //println!("Received {} bytes from {}", size, src);
                if let Ok(packet) = parse_packet(&buf[..size]) {
                    // Send the packet to the HID task
                    if tx.send(packet).await.is_err() {
                        eprintln!("Failed to send packet to HID task");
                    }
                }
            }
            Err(e) => {
                eprintln!("Error receiving data: {}", e);
            }
        }
    }
}

async fn start_hid_interaction(mut rx: mpsc::Receiver<TelemetryData>) {
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

            if device_info.usage() == 1 {
                match api.open(device_info.vendor_id(), device_info.product_id()) {
                    Ok(device) => {
                        tokio::spawn(cyclic_hid_interaction(device, rx));
                        return; // Start only one HID task
                    }
                    Err(e) => {
                        eprintln!("Failed to open device: {}", e);
                    }
                }
            }
        }
    }
}

async fn cyclic_hid_interaction(mut device: HidDevice, mut rx: mpsc::Receiver<TelemetryData>) {
    println!("Connected to HID device. Sending UDP packets every 10ms.");

    let mut last_packet = TelemetryData {
        packet_4cc: 0,
        packet_uid: 0,
        shiftlights_fraction: 0.0,
        shiftlights_rpm_start: 0.0,
        shiftlights_rpm_end: 0.0,
        shiftlights_rpm_valid: false,
        vehicle_gear_index: 0,
        vehicle_gear_index_neutral: 0,
        vehicle_gear_index_reverse: 0,
        vehicle_gear_maximum: 0,
        vehicle_speed: 0.0,
        vehicle_transmission_speed: 0.0,
        vehicle_position_x: 0.0,
        vehicle_position_y: 0.0,
        vehicle_position_z: 0.0,
        vehicle_velocity_x: 0.0,
        vehicle_velocity_y: 0.0,
        vehicle_velocity_z: 0.0,
        vehicle_acceleration_x: 0.0,
        vehicle_acceleration_y: 0.0,
        vehicle_acceleration_z: 0.0,
        vehicle_left_direction_x: 0.0,
        vehicle_left_direction_y: 0.0,
        vehicle_left_direction_z: 0.0,
        vehicle_forward_direction_x: 0.0,
        vehicle_forward_direction_y: 0.0,
        vehicle_forward_direction_z: 0.0,
        vehicle_up_direction_x: 0.0,
        vehicle_up_direction_y: 0.0,
        vehicle_up_direction_z: 0.0,
        vehicle_hub_position_bl: 0.0,
        vehicle_hub_position_br: 0.0,
        vehicle_hub_position_fl: 0.0,
        vehicle_hub_position_fr: 0.0,
        vehicle_hub_velocity_bl: 0.0,
        vehicle_hub_velocity_br: 0.0,
        vehicle_hub_velocity_fl: 0.0,
        vehicle_hub_velocity_fr: 0.0,
        vehicle_cp_forward_speed_bl: 0.0,
        vehicle_cp_forward_speed_br: 0.0,
        vehicle_cp_forward_speed_fl: 0.0,
        vehicle_cp_forward_speed_fr: 0.0,
        vehicle_brake_temperature_bl: 0.0,
        vehicle_brake_temperature_br: 0.0,
        vehicle_brake_temperature_fl: 0.0,
        vehicle_brake_temperature_fr: 0.0,
        vehicle_engine_rpm_max: 0.0,
        vehicle_engine_rpm_idle: 0.0,
        vehicle_engine_rpm_current: 0.0,
        vehicle_throttle: 0.0,
        vehicle_brake: 0.0,
        vehicle_clutch: 0.0,
        vehicle_steering: 0.0,
        vehicle_handbrake: false,
        game_total_time: 0.0,
        game_delta_time: 0.0,
        game_frame_count: 0,
        stage_current_time: 0.0,
        stage_current_distance: 0.0,
        stage_length: 0.0,
    };

    loop {
        // Check if there's a new packet from UDP
        if let Ok(packet) = rx.try_recv() {
            last_packet = packet; // Update last received packet
        }
        //TODO! - Reduce the packets sent by only sending when theres a change
        // Prepare the message to send
        let mut output: Vec<u8> = vec![0x00]; // Report ID = 0x00
        output = create_hid_packet(&last_packet,1);
        //output.extend_from_slice(&last_packet);//TODO: ve la o que fazes aqui
        println!("Sent to HID Device: {:?}\n", &output);
        // Send to HID device
        if let Err(e) = device.write(&output) {
            eprintln!("Failed to write to device: {}", e);
        }

        // Read response
        let mut buf = [0u8; 5];
        match device.read(&mut buf) {
            Ok(len) => {
                println!("Received from HID Device: {:?}\n", &buf[..len]);
            }
            Err(e) => {
                eprintln!("Failed to read from device: {}", e);
            }
        }

        // Sleep asynchronously for 10ms
        //sleep(Duration::from_millis(1)).await;
    }
}



fn parse_packet(buffer: &[u8]) -> Result<TelemetryData, &'static str> {
    let mut offset = 0;

    let read_u32 = |buf: &[u8]| ->  Result<u32,&'static str>{
        buf.try_into()
        .map(u32::from_le_bytes)
        .map_err(|_| "Invalid u32")
    };
    let read_u64 = |buf: &[u8]| -> Result<u64, &'static str> {
        buf.try_into()
        .map(u64::from_le_bytes)
        .map_err(|_| "Invalid u64")
    };
    let read_f32 = |buf: &[u8]| -> Result<f32, &'static str> {
        buf.try_into()
        .map(f32::from_le_bytes)
        .map_err(|_| "Invalid f32")
    };
    let read_bool = |buf: &[u8]| -> Result<bool, &'static str> { Ok(buf[0] != 0) };
    let read_u8 = |buf: &[u8]| -> Result<u8, &'static str> { Ok(buf[0]) };

    if buffer.len() < 4 {
        return Err("Buffer too small");
    }
    //TODO:: Fix offsets
    let packet_4cc = read_u32(&buffer[offset..offset + 4])?;                    
    offset += 4;
    let packet_uid= read_u64(&buffer[offset..offset + 8])?;                     
    offset += 8;
    let shiftlights_fraction= read_f32(&buffer[offset..offset + 4])?;           
    offset += 4;
    let shiftlights_rpm_start= read_f32(&buffer[offset..offset + 4])?;
    offset += 4;
    let shiftlights_rpm_end= read_f32(&buffer[offset..offset + 4])?;
    offset += 4;
    let shiftlights_rpm_valid= read_bool(&buffer[offset..offset + 1])?;
    offset += 1;
    let vehicle_gear_index= read_u8(&buffer[offset..offset + 1])?;
    offset += 1;
    let vehicle_gear_index_neutral= read_u8(&buffer[offset..offset + 1])?;
    offset += 1;
    let vehicle_gear_index_reverse= read_u8(&buffer[offset..offset + 1])?;
    offset += 1;
    let vehicle_gear_maximum= read_u8(&buffer[offset..offset + 1])?;
    offset += 1;
    let vehicle_speed= read_f32(&buffer[offset..offset + 4])?;
    offset += 4;
    let vehicle_transmission_speed= read_f32(&buffer[offset..offset + 4])?;
    offset += 4;
    let vehicle_position_x= read_f32(&buffer[offset..offset + 4])?;
    offset += 4;
    let vehicle_position_y= read_f32(&buffer[offset..offset + 4])?;
    offset += 4;
    let vehicle_position_z= read_f32(&buffer[offset..offset + 4])?;
    offset += 4;
    let vehicle_velocity_x= read_f32(&buffer[offset..offset + 4])?;
    offset += 4;
    let vehicle_velocity_y= read_f32(&buffer[offset..offset + 4])?;
    offset += 4;
    let vehicle_velocity_z= read_f32(&buffer[offset..offset + 4])?;
    offset += 4;
    let vehicle_acceleration_x= read_f32(&buffer[offset..offset + 4])?;
    offset += 4;
    let vehicle_acceleration_y= read_f32(&buffer[offset..offset + 4])?;
    offset += 4;
    let vehicle_acceleration_z= read_f32(&buffer[offset..offset + 4])?;
    offset += 4;
    let vehicle_left_direction_x= read_f32(&buffer[offset..offset + 4])?;
    offset += 4;
    let vehicle_left_direction_y= read_f32(&buffer[offset..offset + 4])?;
    offset += 4;
    let vehicle_left_direction_z= read_f32(&buffer[offset..offset + 4])?;
    offset += 4;
    let vehicle_forward_direction_x= read_f32(&buffer[offset..offset + 4])?;
    offset += 4;
    let vehicle_forward_direction_y= read_f32(&buffer[offset..offset + 4])?;
    offset += 4;
    let vehicle_forward_direction_z= read_f32(&buffer[offset..offset + 4])?;
    offset += 4;
    let vehicle_up_direction_x= read_f32(&buffer[offset..offset + 4])?;
    offset += 4;
    let vehicle_up_direction_y= read_f32(&buffer[offset..offset + 4])?;
    offset += 4;
    let vehicle_up_direction_z= read_f32(&buffer[offset..offset + 4])?;
    offset += 4;
    let vehicle_hub_position_bl= read_f32(&buffer[offset..offset + 4])?;
    offset += 4;
    let vehicle_hub_position_br= read_f32(&buffer[offset..offset + 4])?;
    offset += 4;
    let vehicle_hub_position_fl= read_f32(&buffer[offset..offset + 4])?;
    offset += 4;
    let vehicle_hub_position_fr= read_f32(&buffer[offset..offset + 4])?;
    offset += 4;
    let vehicle_hub_velocity_bl= read_f32(&buffer[offset..offset + 4])?;
    offset += 4;
    let vehicle_hub_velocity_br= read_f32(&buffer[offset..offset + 4])?;
    offset += 4;
    let vehicle_hub_velocity_fl= read_f32(&buffer[offset..offset + 4])?;
    offset += 4;
    let vehicle_hub_velocity_fr= read_f32(&buffer[offset..offset + 4])?;
    offset += 4;
    let vehicle_cp_forward_speed_bl= read_f32(&buffer[offset..offset + 4])?;
    offset += 4;
    let vehicle_cp_forward_speed_br= read_f32(&buffer[offset..offset + 4])?;
    offset += 4;
    let vehicle_cp_forward_speed_fl= read_f32(&buffer[offset..offset + 4])?;
    offset += 4;
    let vehicle_cp_forward_speed_fr= read_f32(&buffer[offset..offset + 4])?;
    offset += 4;
    let vehicle_brake_temperature_bl= read_f32(&buffer[offset..offset + 4])?;
    offset += 4;
    let vehicle_brake_temperature_br= read_f32(&buffer[offset..offset + 4])?;
    offset += 4;
    let vehicle_brake_temperature_fl= read_f32(&buffer[offset..offset + 4])?;
    offset += 4;
    let vehicle_brake_temperature_fr= read_f32(&buffer[offset..offset + 4])?;
    offset += 4;
    let vehicle_engine_rpm_max= read_f32(&buffer[offset..offset + 4])?;
    offset += 4;
    let vehicle_engine_rpm_idle= read_f32(&buffer[offset..offset + 4])?;
    offset += 4;
    let vehicle_engine_rpm_current= read_f32(&buffer[offset..offset + 4])?;
    offset += 4;
    let vehicle_throttle= read_f32(&buffer[offset..offset + 4])?;
    offset += 4;
    let vehicle_brake= read_f32(&buffer[offset..offset + 4])?;
    offset += 4;
    let vehicle_clutch= read_f32(&buffer[offset..offset + 4])?;
    offset += 4;
    let vehicle_steering= read_f32(&buffer[offset..offset + 4])?;
    offset += 4;
    let vehicle_handbrake= read_bool(&buffer[offset..offset + 1])?;
    offset += 1;
    let game_total_time= read_f32(&buffer[offset..offset + 4])?;
    offset += 4;
    let game_delta_time= read_f32(&buffer[offset..offset + 4])?;
    offset += 4;
    let game_frame_count= read_u32(&buffer[offset..offset + 4])?;
    offset += 4;
    let stage_current_time= read_f32(&buffer[offset..offset + 4])?;
    offset += 4;
    let stage_current_distance= read_f32(&buffer[offset..offset + 4])?;
    offset += 4;
    let stage_length= read_f32(&buffer[offset..offset + 4])?;


    let packet = TelemetryData {
        packet_4cc,
        packet_uid,
        shiftlights_fraction,
        shiftlights_rpm_start,
        shiftlights_rpm_end,
        shiftlights_rpm_valid,
        vehicle_gear_index,
        vehicle_gear_index_neutral,
        vehicle_gear_index_reverse,
        vehicle_gear_maximum,
        vehicle_speed,
        vehicle_transmission_speed,
        vehicle_position_x,
        vehicle_position_y,
        vehicle_position_z,
        vehicle_velocity_x,
        vehicle_velocity_y,
        vehicle_velocity_z,
        vehicle_acceleration_x,
        vehicle_acceleration_y,
        vehicle_acceleration_z,
        vehicle_left_direction_x,
        vehicle_left_direction_y,
        vehicle_left_direction_z,
        vehicle_forward_direction_x,
        vehicle_forward_direction_y,
        vehicle_forward_direction_z,
        vehicle_up_direction_x,
        vehicle_up_direction_y,
        vehicle_up_direction_z,
        vehicle_hub_position_bl,
        vehicle_hub_position_br,
        vehicle_hub_position_fl,
        vehicle_hub_position_fr,
        vehicle_hub_velocity_bl,
        vehicle_hub_velocity_br,
        vehicle_hub_velocity_fl,
        vehicle_hub_velocity_fr,
        vehicle_cp_forward_speed_bl,
        vehicle_cp_forward_speed_br,
        vehicle_cp_forward_speed_fl,
        vehicle_cp_forward_speed_fr,
        vehicle_brake_temperature_bl,
        vehicle_brake_temperature_br,
        vehicle_brake_temperature_fl,
        vehicle_brake_temperature_fr,
        vehicle_engine_rpm_max,
        vehicle_engine_rpm_idle,
        vehicle_engine_rpm_current,
        vehicle_throttle,
        vehicle_brake,
        vehicle_clutch,
        vehicle_steering,
        vehicle_handbrake,
        game_total_time,
        game_delta_time,
        game_frame_count,
        stage_current_time,
        stage_current_distance,
        stage_length,
    };

    Ok(packet)

}



/**
 * MAX Size for hid packet 64 bytes
 */
fn create_hid_packet(input:&TelemetryData, packetID:u8) -> Vec<u8>{
    // Prepare the message to send
    let mut output: Vec<u8> = vec![0x00]; // Report ID = 0x00
    
    //first byte is the packet ID
    output.push(packetID);
    match packetID {
        1 => { output.push(input.vehicle_gear_index);
                    /*println!("Sending gear Index {}", input.vehicle_gear_index);*/},
        _ => println!("Not considered yet"), //TODO
    }

    output
}
