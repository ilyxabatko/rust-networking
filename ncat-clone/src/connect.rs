use std::net::TcpStream;
use std::io::Write;

pub fn run(host: &String, port: &u16) -> Result<(), String> {
    let addr = format!("{}:{}", host, port);
    let mut client = TcpStream::connect(&addr).map_err(|e| format!("Failed to connect to the address {}! {}", addr, e))?;

    client.write("Hello, TCP!".as_bytes()).map_err(|e| format!("Failed to send: {}", e))?;

    Ok(())
}