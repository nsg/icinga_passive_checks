use std::os::unix::net::{UnixListener, UnixStream};
use std::io::{Read, Write};
use std::path::PathBuf;
use crate::checks::{self, CheckResult};
use crate::config;

pub const SOCKET_PATH: &str = "/tmp/icinga_checks.sock";

pub fn start_control_socket() -> std::io::Result<()> {
    let socket = PathBuf::from(SOCKET_PATH);
    if socket.exists() {
        std::fs::remove_file(&socket)?;
    }

    let listener = UnixListener::bind(SOCKET_PATH)?;
    
    for stream in listener.incoming() {
        match stream {
            Ok(mut stream) => {
                let mut buffer = [0; 1024];
                match stream.read(&mut buffer) {
                    Ok(n) => {
                        let command = String::from_utf8_lossy(&buffer[..n]);
                        handle_command(&mut stream, &command);
                    }
                    Err(e) => eprintln!("Failed to read from socket: {}", e),
                }
            }
            Err(e) => eprintln!("Failed to accept connection: {}", e),
        }
    }
    Ok(())
}

fn handle_command(stream: &mut UnixStream, command: &str) {
    let response = match command.trim().split('|').collect::<Vec<_>>().as_slice() {
        ["report", check_source, check_name, exit_status, plugin_output] => {
            let config = config::load_config();
            let mut check_data = CheckResult::new();
            check_data.insert("exit_status".to_string(), exit_status.to_string());
            check_data.insert("plugin_output".to_string(), plugin_output.to_string());
            
            checks::send_passive_check(
                check_source,
                check_name,
                "127.0.0.1",
                "Passive Command",
                &check_data,
                &config
            );
            "report sent"
        }
        _ => "unknown command (valid: status, ping, report|<source>|<name>|<exit_status>|<output>)",
    };
    
    let _ = stream.write_all(response.as_bytes());
}

pub fn send_command(command: &str) -> std::io::Result<String> {
    let mut stream = UnixStream::connect(SOCKET_PATH)?;
    stream.write_all(command.as_bytes())?;
    
    let mut response = String::new();
    stream.read_to_string(&mut response)?;
    Ok(response)
}
