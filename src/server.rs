// Import necessary modules and crates
use crate::message::{ServerMessage, AddResponse, ClientMessage, client_message, server_message};
use log::{error, info, warn}; // Logging macros
use prost::Message; // Protobuf message encoding/decoding
use std::collections::HashMap; // HashMap for storing server instances
use std::{
    io::{self, ErrorKind, Read, Write}, // I/O operations
    net::{TcpListener, TcpStream}, // Networking
    sync::{
        atomic::{AtomicBool, Ordering}, // Atomic operations for thread safety
        {Arc, Mutex}, // Arc for reference counting, Mutex for mutual exclusion
    },
    thread, // Threading
    time::Duration, // Time handling
};
use lazy_static::lazy_static; // Import the lazy_static crate for static initialization

// Define the Client struct
#[derive(Debug)]
pub struct Client {
    stream: TcpStream, // TCP stream for client connection
}

// Implement methods for the Client struct
impl Client {
    // Create a new Client instance
    pub fn new(stream: TcpStream) -> Self {
        Client { stream }
    }

    // Handle client messages
    pub fn handle(&mut self) -> io::Result<()> {
        let mut buffer = [0; 512]; // Buffer for reading data
        // Read data from the client
        let bytes_read = self.stream.read(&mut buffer)?;
        if bytes_read == 0 {
            if bytes_read == 0 {
                return Err(io::Error::new(ErrorKind::ConnectionAborted, "Client disconnected"));
            }
            return Ok(());
        }

        // Decode the client message
        if let Ok(client_message) = ClientMessage::decode(&buffer[..bytes_read]) {
            match client_message.message {
                // Handle EchoMessage
                Some(client_message::Message::EchoMessage(echo_message)) => {
                    info!("Received EchoMessage: {}", echo_message.content);
                    // Create a ServerMessage with EchoMessage
                    let server_message = ServerMessage {
                        message: Some(server_message::Message::EchoMessage(echo_message)),
                    };
                    // Encode the ServerMessage
                    let payload = server_message.encode_to_vec();
                    self.stream.write_all(&payload)?; // Send the response
                    self.stream.flush()?; // Flush the stream
                }
                // Handle AddRequest
                Some(client_message::Message::AddRequest(add_request)) => {
                    info!("Received AddRequest: {:?}", add_request);
                    // Process the AddRequest and send back the result
                    let result = add_request.a + add_request.b;
                    let response = AddResponse { result };
                    // Create a ServerMessage with AddResponse
                    let server_message = ServerMessage {
                        message: Some(server_message::Message::AddResponse(response)),
                    };
                    // Encode the ServerMessage
                    let buf = server_message.encode_to_vec();
                    self.stream.write_all(&buf)?; // Send the response
                    self.stream.flush()?; // Flush the stream
                }
                None => {
                    error!("Received message with no content");
                }
            }
        } else {
            error!("Failed to decode message");
        }

        Ok(())
    }
}

// Define the Server struct
#[derive(Debug)]
pub struct Server {
    listener: TcpListener, // TCP listener for incoming connections
    is_running: Arc<AtomicBool>, // Atomic flag to indicate if the server is running
    client_count: Arc<Mutex<usize>>, // Reference counter for active clients
}

// Initialize a static HashMap to store server instances
lazy_static! {
    static ref SERVERS: Arc<Mutex<HashMap<String, Arc<Server>>>> = Arc::new(Mutex::new(HashMap::new()));
}

// Implement methods for the Server struct
impl Server {
    /// Creates a new server instance
    pub fn new(addr: &str) -> io::Result<Arc<Self>> {
        let mut servers_lock = SERVERS.lock().unwrap(); // Lock the HashMap

        // Debugging: Print the contents of the HashMap
        info!("Current server instances: {:?}", *servers_lock);

        // Check if a server instance already exists for the given address
        if let Some(server) = servers_lock.get(addr) {
            warn!("Server instance for address {} already exists.", addr); 
            // Increment the client count
            {
                let mut count = server.client_count.lock().unwrap();
                *count += 1;
            }
            return Ok(Arc::clone(server));
        }

        // Bind the TCP listener to the address
        match TcpListener::bind(addr) {
            Ok(listener) => {
                let is_running = Arc::new(AtomicBool::new(false)); // Initialize the running flag
                let client_count = Arc::new(Mutex::new(1)); // Initialize the client count
                let server = Arc::new(Server {
                    listener,
                    is_running,
                    client_count,
                });
                servers_lock.insert(addr.to_string(), Arc::clone(&server)); // Store the server instance
                Ok(server)
            }
            Err(ref e) if e.kind() == ErrorKind::AddrInUse => {
                eprintln!("Address {} is already in use.", addr);
                Err(io::Error::new(e.kind(), e.to_string()))
            }
            Err(e) => {
                eprintln!("Failed to bind to address {}: {}", addr, e);
                Err(e)
            }
        }
    }

    /// Runs the server, listening for incoming connections and handling them
    pub fn run(&self) -> io::Result<()> {
        self.is_running.store(true, Ordering::SeqCst); // Set the server as running
        info!("Server is running on {}", self.listener.local_addr()?);

        // Set the listener to non-blocking mode
        self.listener.set_nonblocking(true)?;

        while self.is_running.load(Ordering::SeqCst) {
            match self.listener.accept() {
                Ok((stream, addr)) => {
                    info!("New client connected: {}", addr);
        
                    // Clone the Arc to share the is_running flag with the new thread
                    let is_running = Arc::clone(&self.is_running);
        
                    // Spawn a new thread to handle the client connection
                    thread::spawn(move || {
                        let mut client = Client::new(stream);
                        while is_running.load(Ordering::SeqCst) {
                            if let Err(e) = client.handle() {
                                error!("Error handling client: {}", e);
                                break;
                            }
                        }
                    });
                }
                Err(ref e) if e.kind() == ErrorKind::WouldBlock => {
                    // No incoming connections, sleep briefly to reduce CPU usage
                    thread::sleep(Duration::from_millis(100));
                }
                Err(e) => {
                    error!("Error accepting connection: {}", e);
                }
            }
        }

        info!("Server stopped.");
        Ok(())
    }

    /// Stops the server by setting the `is_running` flag to `false` and removing it from the HashMap
    pub fn stop(&self) {
        let mut count = self.client_count.lock().unwrap();
        if *count == 1 {
            if self.is_running.load(Ordering::SeqCst) {
                self.is_running.store(false, Ordering::SeqCst);
                info!("Shutdown signal sent.");

                // Remove the server instance from the HashMap
                let mut servers_lock: std::sync::MutexGuard<'_, HashMap<String, Arc<Server>>> = SERVERS.lock().unwrap();
                let addr = self.listener.local_addr().unwrap().to_string();
                servers_lock.remove(&addr);
            } else {
                warn!("Server was already stopped or not running.");
            }
        } else {
            // Decrement the client count
            {
                *count -= 1;
                info!("Client disconnected. Current client count: {}", *count);
            }
            info!("Server still has {} active clients.", *count);
        }
    }
}