# Solution

This Md File details the modifications implemented in the new version of the Server code and Client code and explains the rationale behind each change with Testing Modification to test full functionality.

## Modifications in Server.rs File

### 1. Imports

*   Several new modules from `crate::message` were imported: `ServerMessage`, `AddResponse`, `ClientMessage`, `client_message`, and `server_message`. This addition was necessary to support new message types beyond the original `EchoMessage`, enabling the server to handle different requests and responses.

### 2. Client Struct

*   The `Client` struct remains unchanged, containing a `stream` field (`TcpStream`) for managing the client's TCP connection.

### 3. `Client::handle` Method

*   **Error Handling:**
    *   Enhanced error handling for client disconnections. Instead of just logging the disconnection, the method now returns a specific `io::Error` with `ErrorKind::ConnectionAborted` and the message "Client disconnected." This allows the calling function to handle disconnections gracefully and potentially perform cleanup or other actions.
*   **Message Handling:**
    *   The method now handles multiple message types using a `match` statement on the decoded `ClientMessage`.
    *   **`EchoMessage` Handling:** When an `EchoMessage` is received, it is now encapsulated within a `ServerMessage` before being sent back to the client. This provides a consistent message structure for all server responses.
    *   **`AddRequest` Handling:** The method now includes logic to process `AddRequest` messages. It extracts the operands, performs the addition, creates an `AddResponse` containing the result, and sends it back to the client within a `ServerMessage`. This implements the new addition functionality.
    *   **Handling Empty Messages:** A case for `None` (no message content) was added to the `match` statement. This ensures that the server handles malformed or empty messages gracefully by logging an error, preventing potential crashes.

### 4. Server Struct

*   A `client_count` field (`Arc<Mutex<usize>>`) was added to the `Server` struct. This field is used to track the number of active clients connected to a specific server instance. Using `Arc<Mutex<usize>>` ensures thread-safe access and modification of the client count from multiple threads.

### 5. Server Initialization (`Server::new`)

*   **Server Instance Management:** A static `HashMap` called `SERVERS` (using `lazy_static!`) was introduced to store multiple `Server` instances, keyed by their listening addresses. This allows the server to manage multiple listening sockets on different ports.
*   **Server Reuse:** The `new` method now checks if a server instance already exists for the given address in the `SERVERS` HashMap. If a server exists, the method increments the `client_count` of the existing server and returns a clone of the `Arc<Server>`. This prevents the creation of duplicate servers on the same address and allows multiple clients to connect to the same server instance.
*   **Improved Error Handling:** More explicit error handling for `ErrorKind::AddrInUse` was added. This provides a more informative error message when attempting to bind to an address that is already in use.

### 6. `Server::run` Method

*   **Concurrent Client Handling:** The most significant change is the introduction of threading. The `run` method now spawns a new thread for each incoming client connection using `thread::spawn`. This allows the server to handle multiple clients concurrently without blocking the main server loop.
*   **Shared `is_running` Flag:** The `is_running` flag (`Arc<AtomicBool>`) is cloned using `Arc::clone` before being moved into the new thread. This allows the client handling threads to access and check the server's running state, ensuring they terminate when the server is stopped.
*   **Refined Error Handling:** The error handling within the main loop was refined to specifically handle `ErrorKind::WouldBlock`. This error indicates that there are no new incoming connections at the moment. The server now sleeps briefly in this case to reduce CPU usage.

### 7. `Server::stop` Method

*   **Client Count Management:** The `stop` method now uses the `client_count` to manage server shutdown.
*   **Conditional Server Removal:** The server instance is only removed from the `SERVERS` HashMap when the `client_count` reaches zero, indicating that all clients have disconnected. This prevents the server from being removed prematurely if there are still active clients.
*   **Improved Logging:** More detailed logging messages were added to provide better insight into client disconnections and the server's state.

### Summary of Modifications

These modifications have significantly improved the server's functionality, scalability, and robustness:

*   **Support for Multiple Message Types:** The server can now handle different types of requests and responses.
*   **Concurrent Client Handling:** The use of threads allows the server to handle multiple clients simultaneously.
*   **Thread Safety:** `Arc` and `Mutex` ensure thread-safe access to shared data.
*   **Efficient Resource Management:** The `SERVERS` HashMap and `client_count` provide efficient management of server instances and client connections.
*   **Improved Error Handling:** More specific and informative error handling improves the server's stability.

This revised version is more robust, scalable, and capable of handling a wider range of client interactions.

## Modifications in Client.rs File

### Client Structure

The `Client` struct holds the following fields:

*   `ip`: `String` - Stores the server's IP address.
*   `port`: `u32` - Stores the server's port number.
*   `timeout`: `Duration` - Represents the connection timeout duration.
*   `stream`: `Option<TcpStream>` - Holds an optional `TcpStream` for the client's connection to the server.

### Client Methods

### 1. `new(ip: &str, port: u32, timeout_ms: u64) -> Self`

*   Creates a new `Client` instance.
*   Takes the server's IP (`ip`), port (`port`), and timeout in milliseconds (`timeout_ms`) as arguments.
*   Converts the `ip` parameter to a `String`.
*   Sets the `timeout` field using `Duration::from_millis`.

### 2. `connect(&mut self) -> io::Result<()>`

*   Connects the client to the server.
*   Logs a message indicating the connection attempt.
*   Constructs the server's address by formatting the `ip` and `port`.
*   Resolves the address using `to_socket_addrs` and collects the results into a `Vec<SocketAddr>`.
*   Returns an `io::Error` with `ErrorKind::InvalidInput` if the address resolution fails (e.g., invalid IP or port).
*   Attempts a connection with a timeout using `TcpStream::connect_timeout`.
*   Stores the obtained `TcpStream` in the `stream` field upon successful connection.
*   Logs a message indicating a successful connection.

### 3. `disconnect(&mut self) -> io::Result<()>`

*   Disconnects the client from the server if there's an active connection.
*   Retrieves the `TcpStream` from the `stream` field using `take`, which removes the value and leaves `None` in its place.
*   Shuts down the connection using `stream.shutdown(std::net::Shutdown::Both)` if a stream exists. This ensures both reading and writing are stopped.
*   Logs a message indicating a successful disconnection.

### 4. `send(&mut self, message: client_message::Message) -> io::Result<()>`

*   Sends a message to the server.
*   Checks for an active connection by verifying the presence of a `TcpStream` in the `stream` field.
*   Encodes the provided `client_message::Message` into a buffer (`Vec<u8>`) using the `encode` method from the `prost` crate.
*   Writes the encoded buffer to the server's `TcpStream` using `write_all` and flushes the stream using `flush` to ensure the data is sent immediately.
*   Logs a message indicating the message has been sent.
*   Returns an `io::Error` with `ErrorKind::NotConnected` if there is no active connection.

### 5. `receive(&mut self) -> io::Result<ServerMessage>`

*   Receives a message from the server.
*   Checks for an active connection.
*   Creates a buffer (`Vec<u8>`) of size 1024 bytes to hold the received data.
*   Reads data from the `TcpStream` into the buffer using `read`.
*   Returns an `io::Error` with `ErrorKind::ConnectionAborted` and an informative message if zero bytes are read, indicating a server disconnection.
*   Logs the number of bytes received.
*   Attempts to decode the received data using `ServerMessage::decode`.
    *   On successful decoding:
        *   Checks if the `message` field within the `ServerMessage` is present.
        *   If the `message` is present, a `match` statement is used to handle different `server_message::Message` types:
            *   `AddResponse`: Logs the result of the addition.
            *   `EchoMessage`: Logs the echo message content.
            *   Logs an error if the message type is unexpected or missing.
        *   Returns the decoded `ServerMessage`.
    *   On decoding failure, logs the specific error details and returns an `io::Error` with `ErrorKind::InvalidData`.
*   Returns an `io::Error` with `ErrorKind::NotConnected` and a message indicating no stream is available if there is no active connection.

### Summary of Modifications

The new client code maintains the core functionality (connecting, sending, receiving, disconnecting) but adds:

*   **Improved Error Handling:** More specific error kinds and messages are used.
*   **Structured Message Handling:** The `receive` function now handles different types of server responses (`AddResponse` and `EchoMessage`) using a `match` statement, improving code clarity and extensibility.
*   **Logging:** More informative logging is added throughout the methods.

These changes make the client code more robust, easier to debug, and capable of handling a wider range of server interactions.

## Modifications for Test Cases

The following changes were implemented in the test files:

### 1. Added `env_logger` for Logging

*   A new line `use env_logger;` was added to the import statements.
*   The line `let _ = env_logger::builder().is_test(true).try_init();` was added to the beginning of each test function.

**Explanation:** This change enables logging during tests. `env_logger` is a logging implementation for Rust. The `is_test(true)` part configures the logger for test environments, and `try_init()` attempts to initialize the logger. This allows for more detailed output during test runs, aiding in debugging and understanding test behavior.

### 2. Modified Server Creation Function

*   The `create_server` function was modified to accept a server address (`addr: &str`) as an argument.
*   The function now uses `Server::new(addr).expect("Failed to create server")` to create the server instance.

**Explanation:** This change makes the server creation more flexible. Previously, the server address was hardcoded. Now, different addresses can be used for different tests, which is especially useful for testing scenarios involving multiple servers or specific port configurations.

## Test Cases Created Explanations

### `test_multiple_clients_with_different_message_types`

This test specifically addresses the scenario of multiple clients interacting with multiple servers using different message types (both `EchoMessage` and `AddRequest`).

**Steps:**

1.  **Initialize Logging:** `env_logger` is initialized for test logging.

2.  **Create Multiple Servers:** Two server instances (`server1` and `server2`) are created, each listening on a different address ("localhost:2050" and "localhost:2010", respectively). This sets up a scenario with multiple independent servers.

3.  **Spawn Server Threads:** Separate threads (`handle1` and `handle2`) are spawned to run each server concurrently.

4.  **Create Multiple Clients:** Three client instances are created. One client connects to `server1` (port 2050), and the other two connect to `server2` (port 2010).

5.  **Prepare Messages:** Two sets of messages are prepared:
    *   `echo_messages`: A vector of strings for `EchoMessage` tests.
    *   `add_requests`: A vector of tuples `(a, b)` for `AddRequest` tests.

6.  **Test Echo Messages:**
    *   The test iterates through the `echo_messages`.
    *   For each message content:
        *   An `EchoMessage` is created.
        *   The message is sent to *each* client.
        *   The response is received from *each* client.
        *   The received echo content is asserted to match the sent content.

7.  **Test Add Requests:**
    *   The test iterates through the `add_requests`.
    *   For each `(a, b)` tuple:
        *   An `AddRequest` is created.
        *   The message is sent to *each* client.
        *   The response is received from *each* client.
        *   The received sum is asserted to match the expected sum (`a + b`).

8.  **Disconnect Clients:** All clients are disconnected.

9.  **Stop Servers and Join Threads:** Both servers are stopped, and the server threads are joined to ensure proper cleanup.

**Purpose:**

This test verifies the following:

*   **Multiple Server Handling:** The client can correctly interact with multiple servers simultaneously.
*   **Message Type Handling:** The client can send and receive different message types (`EchoMessage` and `AddRequest`) correctly.
*   **Concurrency:** Multiple clients can interact with the servers without interfering with each other.
*   **Server Independence:** The two servers operate independently, and clients connected to one server do not affect clients connected to the other.

This comprehensive test is crucial for ensuring the robustness and correctness of the client-server interaction in more complex scenarios.
