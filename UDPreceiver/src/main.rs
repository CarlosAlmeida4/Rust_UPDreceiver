use core::str;
use std::net::UdpSocket;
use std::io;

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

fn main() -> io::Result<()>{
    let mut addr = String::new();
    println!("Please input the IP and the port");
    io::stdin().read_line(&mut addr)?;
    
    addr.pop(); //pop out \n
    addr.pop(); //pop out \r

    let socket = UdpSocket::bind(addr.clone()).expect("cannot bind to address");
    println!("Listening on socket {}",addr);

    let mut buf = [0u8;1024];

    loop {
        match socket.recv_from(&mut buf) {
            Ok((size,src)) => {
                //println!("Received {} bytes from {}",size,src);
                if let Ok(packet) = parse_packet(&buf[..size]) {
                    println!("Id: {:?}", packet.packet_uid);
                    println!("Gear Index: {:?}", packet.vehicle_gear_index);
                } else {
                    println!("Failed to parse the packet");
                }
            }
            Err(e) => {
                eprintln!("Error receiving data: {}",e);
            }
        }
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