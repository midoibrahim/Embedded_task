// Import necessary modules and crates
use embedded_recruitment_task::message::{client_message, ServerMessage, server_message}; // Protobuf message types
use log::error; // Logging macros for error messages
use log::info; // Logging macros for informational messages
use prost::Message; // Protobuf message encoding/decoding
use std::io::Read; // Trait for reading from streams
use std::io::Write; // Trait for writing to streams
use std::{
    io, // Standard I/O library
    net::{SocketAddr, TcpStream, ToSocketAddrs}, // Networking types and traits
    time::Duration, // Time handling
};

// TCP/IP Client
pub struct Client {
    ip: String, // IP address of the server
    port: u32, // Port number of the server
    timeout: Duration, // Connection timeout duration
    stream: Option<TcpStream>, // Optional TCP stream for the connection
}
impl Client {
    pub fn new(ip: &str, port: u32, timeout_ms: u64) -> Self {
        Client {
            ip: ip.to_string(),
            port,
            timeout: Duration::from_millis(timeout_ms),
            stream: None,
        }
    }

    // connect the client to the server
    pub fn connect(&mut self) -> io::Result<()> {
        println!("Connecting to {}:{}", self.ip, self.port);

        // Resolve the address
        let address = format!("{}:{}", self.ip, self.port);
        let socket_addrs: Vec<SocketAddr> = address.to_socket_addrs()?.collect();

        if socket_addrs.is_empty() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "Invalid IP or port",
            ));
        }

        // Connect to the server with a timeout
        let stream = TcpStream::connect_timeout(&socket_addrs[0], self.timeout)?;
        self.stream = Some(stream);

        println!("Connected to the server!");
        Ok(())
    }

    // disconnect the client
    pub fn disconnect(&mut self) -> io::Result<()> {
        if let Some(stream) = self.stream.take() {
            stream.shutdown(std::net::Shutdown::Both)?;
        }

        println!("Disconnected from the server!");
        Ok(())
    }

    // generic message to send message to the server
    pub fn send(&mut self, message: client_message::Message) -> io::Result<()> {
        if let Some(ref mut stream) = self.stream {
            // Encode the message to a buffer
            let mut buffer = Vec::new();
            message.encode(&mut buffer);

            // Send the buffer to the server
            stream.write_all(&buffer)?;
            stream.flush()?;

            Ok(())
        } else {
            Err(io::Error::new(
                io::ErrorKind::NotConnected,
                "No active connection",
            ))
        }
    }
    // Receive a message from the server
    pub fn receive(&mut self) -> io::Result<ServerMessage> {
        if let Some(ref mut stream) = self.stream {
            info!("Receiving message from the server");
            let mut buffer = vec![0u8; 1024]; // Create a buffer to hold the received data
            let bytes_read = stream.read(&mut buffer)?; // Read data from the stream
            if bytes_read == 0 {
                info!("Server disconnected.");
                return Err(io::Error::new(
                    io::ErrorKind::ConnectionAborted,
                    "Server disconnected",
                ));
            }

            info!("Received {} bytes from the server", bytes_read);

            // Decode the received message
            match ServerMessage::decode(&buffer[..bytes_read]) {
                Ok(server_message) => {
                    if let Some(ref message) = server_message.message {
                        match message {
                            server_message::Message::AddResponse(add_response) => {
                                info!("Received AddResponse: result = {}", add_response.result);
                            }
                            server_message::Message::EchoMessage(echo_response) => {
                                info!("Received EchoResponse: content = {}", echo_response.content);
                            }
                        }
                    } else {
                        error!("Received empty server message");
                    }
                    Ok(server_message)
                }
                Err(e) => {
                    error!("Failed to decode ServerMessage: {}", e);
                    Err(io::Error::new(
                        io::ErrorKind::InvalidData,
                        format!("Failed to decode ServerMessage: {}", e),
                    ))
                }
            }
        } else {
            error!("No active connection");
            Err(io::Error::new(
                io::ErrorKind::NotConnected,
                "No stream available",
            ))
        }
    }
}
