use std::io::{stdout, Read, Write};
use std::net::TcpListener;

pub fn run(bind_host: &String, port: &u16) -> Result<(), String> {
    let addr = format!("{}:{}", bind_host, port);
    let listener = TcpListener::bind(&addr)
        .map_err(|e| format!("Failed to bind to {} address! {}", addr, e))?;

    for stream in listener.incoming() {
        match stream {
            Ok(mut stream) => {
                println!("Connection {:?} accepted", &stream);

                // Creates a buffer to read to from the socket (up to 128 bytes)
                let mut buf = [0u8; 128];
                let mut read_bytes: usize = 0;
                while read_bytes == 0 {
                    read_bytes = stream
                        .read(&mut buf)
                        .map_err(|e| format!("Failed to read from socket: {}", e))?;
                    println!("Received {} bytes from socket.", read_bytes);
                }

                // Print the buffer to the console cutting off zeroes
                stdout().write(&buf[0..read_bytes]).map_err(|e| format!("Error printing out the buffer: {}", e))?;
                stdout().flush().map_err(|e| format!("Error flushing the stdout: {}", e))?;
            }
            Err(e) => println!("Error while accepting connection: {}", e),
        };
    }

    Ok(())
}
