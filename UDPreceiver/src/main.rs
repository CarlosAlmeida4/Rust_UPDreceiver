use serialport;
use std::time::Duration;
use std::io::{self, Write};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Specify your Windows COM port (e.g., COM4) and baud rate
    let port_name = "COM8";  
    let baud_rate = 115_200;

    // Open the serial port
    let mut port = serialport::new(port_name, baud_rate)
        .timeout(Duration::from_secs(1))
        .flow_control(serialport::FlowControl::None) // Ensure flow control is disabled
        .open()?;
    println!("Connected to {}", port_name);

    // Create a loop to send data
    let stdin = io::stdin();
    loop {
        let mut input = String::new();
        println!("Enter command to send (or type 'exit' to quit):");
        input.clear();
        stdin.read_line(&mut input)?;

        let trimmed_input = input.trim();
        if trimmed_input.eq_ignore_ascii_case("exit") {
            println!("Exiting...");
            break;
        }

        // Append \r\n to the input to ensure Arduino receives the command properly
        let command = format!("{}\r\n", trimmed_input);
        port.write_all(command.as_bytes())?;
        port.flush()?;
        println!("Sent: {}", trimmed_input);
    }

    Ok(())
}